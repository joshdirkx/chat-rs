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
use crate::grpc_handlers::create_user::handle_create_user;
use crate::grpc_handlers::get_user::handle_get_user;
use crate::proto::messaging_server::MessagingServer;
use crate::proto_structs::user::User;
use crate::websocket_structs::incoming_websocket_message::IncomingWebsocketMessage;

mod db;
mod grpc_handlers;
mod proto_structs;
mod websocket_structs;

mod proto {
    tonic::include_proto!("messaging");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("messaging_descriptor");
}

#[derive(Debug, Default)]
struct MessagingService {}

#[tonic::async_trait]
impl Messaging for MessagingService {
    async fn create_user(
        &self,
        request: tonic::Request<proto::CreateUserRequest>,
    ) -> Result<tonic::Response<proto::CreateUserResponse>, tonic::Status> {
        handle_create_user(request).await
    }

    async fn get_user(
        &self,
        request: tonic::Request<proto::GetUserRequest>,
    ) -> Result<tonic::Response<proto::GetUserResponse>, tonic::Status> {
        handle_get_user(request).await
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
