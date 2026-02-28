use std::sync::Arc;

use crate::imessage::Message;
use crate::matrix::MatrixAppservice;

#[derive(Debug, Clone)]
pub struct IMessageInboundMessage {
    pub message: Message,
    pub channel_id: String,
}

#[derive(Debug, Clone)]
pub struct OutboundMatrixMessage {
    pub room_id: String,
    pub sender_id: String,
    pub content: String,
    pub reply_to: Option<String>,
    pub edit_of: Option<String>,
}

pub struct MessageFlow {
    matrix_client: Arc<MatrixAppservice>,
}

impl MessageFlow {
    pub fn new(matrix_client: Arc<MatrixAppservice>) -> Self {
        Self { matrix_client }
    }

    pub async fn send_to_matrix(&self, message: OutboundMatrixMessage) -> anyhow::Result<String> {
        self.matrix_client
            .send_message(
                &message.room_id,
                &message.sender_id,
                &message.content,
                message.reply_to.as_deref(),
                message.edit_of.as_deref(),
            )
            .await
    }
}
