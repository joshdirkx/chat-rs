extern crate derive_builder;

use futures_util::SinkExt;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tonic::transport::Server;

use proto::messaging_server::Messaging;

use crate::db::DATABASE_POOL;
use crate::proto::messaging_server::MessagingServer;
use crate::websocket_message::IncomingWebsocketMessage;

mod db;
mod websocket_message;

mod proto {
    tonic::include_proto!("messaging");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("messaging_descriptor");
}

#[derive(Debug, Default)]
struct MessagingService {}

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Option<i32>,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Message {
    pub sender: User,
    pub recipient: User,
    pub contents: String,
}

#[tonic::async_trait]
impl Messaging for MessagingService {
    async fn create_user(
        &self,
        request: tonic::Request<proto::CreateUserRequest>,
    ) -> Result<tonic::Response<proto::CreateUserResponse>, tonic::Status> {
        let user_create_request = request.get_ref();

        let user_params = User {
            id: None,
            first_name: user_create_request.first_name.to_string(),
            last_name: user_create_request.last_name.to_string(),
        };

        let query = "
            INSERT INTO users (first_name, last_name)
            VALUES ($1, $2)
            RETURNING id, first_name, last_name
        ";

        let row = sqlx::query_as::<_, User>(query)
            .bind(&user_params.first_name)
            .bind(&user_params.last_name)
            .fetch_one(&*DATABASE_POOL)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let user_response = proto::CreateUserResponse {
            user_id: row.id.unwrap(),
            first_name: row.first_name,
            last_name: row.last_name,
        };

        Ok(tonic::Response::new(user_response))
    }

    async fn get_user(
        &self,
        request: tonic::Request<proto::GetUserRequest>,
    ) -> Result<tonic::Response<proto::GetUserResponse>, tonic::Status> {
        let user_create_request = request.get_ref();

        let query = format!(
            "SELECT id, first_name, last_name FROM users WHERE id = {}",
            user_create_request.user_id
        );

        let row = sqlx::query_as::<_, User>(&query)
            .fetch_one(&*DATABASE_POOL)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let user_response = proto::GetUserResponse {
            user_id: row.id.unwrap(),
            first_name: row.first_name,
            last_name: row.last_name,
        };

        Ok(tonic::Response::new(user_response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reflector = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    let messaging_server = MessagingServer::new(MessagingService::default());

    let grpc_server = Server::builder()
        .add_service(reflector)
        .add_service(messaging_server)
        .serve("[::1]:50051".parse()?);

    let websocket_server = async {
        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
        println!("WebSocket Server listening on: 127.0.0.1:8080");

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_connection(stream));
        }
    };

    tokio::select! {
        _ = grpc_server => println!("gRPC server exited"),
        _ = websocket_server => println!("WebSocket server exited"),
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("New WebSocket connection");

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_text().unwrap();

                    let parsed_message: Result<IncomingWebsocketMessage, Error> =
                        serde_json::from_str(text);

                    match parsed_message {
                        Ok(parsed) => {
                            println!("Received message: {:?}", parsed);
                            let query = "
                                INSERT INTO messages (sender_id, recipient_id, message_contents)
                                VALUES ($1, $2, $3)
                                RETURNING id, sender_id, recipient_id, message_contents
                            ";

                            let row = sqlx::query_as::<_, User>(query)
                                .bind(&parsed.sender_id)
                                .bind(&parsed.recipient_id)
                                .bind(&parsed.message_contents)
                                .fetch_one(&*DATABASE_POOL)
                                .await
                                .map_err(|e| tonic::Status::internal(e.to_string()));
                        }
                        Err(e) => {
                            println!("Failed to parse message as JSON: {:?}", e);
                        }
                    }
                }
                // Echo the message back
                write.send(msg).await.expect("Failed to send message");
            }
            Err(e) => {
                eprintln!("Error processing message: {}", e);
                break;
            }
        }
    }
}
