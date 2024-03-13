use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IncomingWebsocketMessage {
    pub(crate) sender_id: i32,
    pub(crate) recipient_id: i32,
    pub(crate) message_contents: String,
}