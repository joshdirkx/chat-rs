extern crate derive_builder;

use lazy_static::lazy_static;
use serde::Serialize;
use sqlx_postgres::{PgPool, PgPoolOptions};
use tonic::transport::Server;

use proto::messaging_server::Messaging;

use crate::proto::messaging_server::MessagingServer;

lazy_static!(
    static ref DATABASE_POOL: PgPool = {
        let database_url = "postgres://postgres:password@localhost/messaging";
        
        PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&database_url)
            .expect("Failed to create pool")
    };
);
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
    async fn send_message(
        &self,
        request: tonic::Request<proto::SendMessageRequest>,
    ) -> Result<tonic::Response<proto::SendMessageResponse>, tonic::Status> {
        let message_request = request.get_ref();

        let sender_uuid = &message_request.sender.as_ref().unwrap().user_id;
        let recipient_uuid = &message_request.recipient.as_ref().unwrap().user_id;

        let message_response = proto::SendMessageResponse {
            message_id: 1,
        };

        Ok(tonic::Response::new(message_response))
    }

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
        let user_response = proto::GetUserResponse {
            user_id: 1,
            first_name: "Josh".to_string(),
            last_name: "Dirkx".to_string(),
        };

        Ok(tonic::Response::new(user_response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    let messaging_service = MessagingService::default();

    let reflector = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    let messaging_server = MessagingServer::new(messaging_service);

    Server::builder()
        .add_service(reflector)
        .add_service(messaging_server)
        .serve(addr)
        .await?;

    Ok(())
}
