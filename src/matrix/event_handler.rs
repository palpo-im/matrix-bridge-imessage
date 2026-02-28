use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, warn};

use crate::bridge::BridgeCore;

use super::MatrixEvent;

#[async_trait]
pub trait MatrixEventHandler: Send + Sync {
    async fn handle_event(&mut self, event: MatrixEvent) -> Result<()>;
}

pub struct MatrixEventProcessor {
    handler: Arc<dyn MatrixEventHandler>,
    age_limit_ms: Option<u64>,
}

impl MatrixEventProcessor {
    pub fn new(handler: Arc<dyn MatrixEventHandler>) -> Self {
        Self {
            handler,
            age_limit_ms: None,
        }
    }

    pub fn with_age_limit(handler: Arc<dyn MatrixEventHandler>, age_limit_ms: u64) -> Self {
        Self {
            handler,
            age_limit_ms: Some(age_limit_ms),
        }
    }

    pub async fn process_event(&self, event: MatrixEvent) -> Result<()> {
        // Check event age
        if let (Some(age_limit), Some(timestamp_str)) = (self.age_limit_ms, &event.timestamp) {
            if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                let now = chrono::Utc::now().timestamp_millis();
                let age = (now - timestamp) as u64;
                if age > age_limit {
                    debug!("skipping event {} due to age {}ms > {}ms", 
                           event.event_id.as_deref().unwrap_or("unknown"), age, age_limit);
                    return Ok(());
                }
            }
        }

        self.handler.handle_event(event).await
    }
}

pub struct MatrixEventHandlerImpl {
    matrix_client: Arc<super::MatrixAppservice>,
    bridge: Option<Arc<BridgeCore>>,
}

impl MatrixEventHandlerImpl {
    pub fn new(matrix_client: Arc<super::MatrixAppservice>) -> Self {
        Self {
            matrix_client,
            bridge: None,
        }
    }

    pub fn set_bridge(&mut self, bridge: Arc<BridgeCore>) {
        self.bridge = Some(bridge);
    }
}

#[async_trait]
impl MatrixEventHandler for MatrixEventHandlerImpl {
    async fn handle_event(&mut self, event: MatrixEvent) -> Result<()> {
        debug!("handling matrix event: {:?}", event.event_type);

        // Skip events from our own users
        if self.matrix_client.is_namespaced_user(&event.sender) {
            debug!("skipping event from bridge user: {}", event.sender);
            return Ok(());
        }

        let Some(bridge) = &self.bridge else {
            warn!("bridge not set, skipping event");
            return Ok(());
        };

        match event.event_type.as_str() {
            "m.room.message" => {
                self.handle_message_event(bridge, event).await?;
            }
            "m.room.redaction" => {
                self.handle_redaction_event(bridge, event).await?;
            }
            "m.reaction" => {
                self.handle_reaction_event(bridge, event).await?;
            }
            "m.room.member" => {
                self.handle_member_event(bridge, event).await?;
            }
            "m.typing" => {
                self.handle_typing_event(bridge, event).await?;
            }
            "m.read_receipt" => {
                self.handle_read_receipt_event(bridge, event).await?;
            }
            _ => {
                debug!("unhandled event type: {}", event.event_type);
            }
        }

        Ok(())
    }
}

impl MatrixEventHandlerImpl {
    async fn handle_message_event(&self, _bridge: &BridgeCore, event: MatrixEvent) -> Result<()> {
        debug!("handling message event in room: {}", event.room_id);
        // TODO: Implement message bridging
        Ok(())
    }

    async fn handle_redaction_event(&self, _bridge: &BridgeCore, event: MatrixEvent) -> Result<()> {
        debug!("handling redaction event in room: {}", event.room_id);
        // TODO: Implement redaction bridging
        Ok(())
    }

    async fn handle_reaction_event(&self, _bridge: &BridgeCore, event: MatrixEvent) -> Result<()> {
        debug!("handling reaction event in room: {}", event.room_id);
        // TODO: Implement reaction bridging
        Ok(())
    }

    async fn handle_member_event(&self, _bridge: &BridgeCore, event: MatrixEvent) -> Result<()> {
        debug!("handling member event in room: {}", event.room_id);
        // TODO: Implement member event handling
        Ok(())
    }

    async fn handle_typing_event(&self, _bridge: &BridgeCore, event: MatrixEvent) -> Result<()> {
        debug!("handling typing event in room: {}", event.room_id);
        // TODO: Implement typing notification bridging
        Ok(())
    }

    async fn handle_read_receipt_event(&self, _bridge: &BridgeCore, event: MatrixEvent) -> Result<()> {
        debug!("handling read receipt event in room: {}", event.room_id);
        // TODO: Implement read receipt bridging
        Ok(())
    }
}
