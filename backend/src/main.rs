extern crate derive_builder;

use tonic::transport::Server;
use uuid::Uuid;

use proto::messaging_server::Messaging;

use crate::proto::messaging_server::MessagingServer;

mod proto {
    tonic::include_proto!("messaging");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("messaging_descriptor");
}

#[derive(Debug, Default)]
struct MessagingService {}

#[tonic::async_trait]
impl Messaging for MessagingService {
    async fn send_message(
        &self,
        request: tonic::Request<proto::SendMessageRequest>,
    ) -> Result<tonic::Response<proto::SendMessageResponse>, tonic::Status> {
        let message_request = request.get_ref();

        let sender_uuid = &message_request.sender.as_ref().unwrap().user_uuid;
        let recipient_uuid = &message_request.recipient.as_ref().unwrap().user_uuid;

        let message_response = proto::SendMessageResponse {
            message_uuid: String::from(Uuid::new_v4()),
        };

        Ok(tonic::Response::new(message_response))
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
