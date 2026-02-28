use std::sync::Arc;

use anyhow::Result;

use crate::bridge::BridgeCore;

use super::MatrixAppservice;

pub struct MatrixCommandHandler {
    enable_self_service_bridging: bool,
    bridge: Option<Arc<BridgeCore>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatrixCommandPermission {
    Admin,
    User,
    None,
}

#[derive(Debug, Clone)]
pub enum MatrixCommandOutcome {
    Success(String),
    Error(String),
    NoCommand,
}

impl MatrixCommandHandler {
    pub fn new(enable_self_service_bridging: bool, bridge: Option<Arc<BridgeCore>>) -> Self {
        Self {
            enable_self_service_bridging,
            bridge,
        }
    }

    pub fn set_bridge(&mut self, bridge: Arc<BridgeCore>) {
        self.bridge = Some(bridge);
    }

    pub async fn handle_command(
        &self,
        _matrix: &MatrixAppservice,
        _room_id: &str,
        _sender: &str,
        _body: &str,
    ) -> Result<MatrixCommandOutcome> {
        // TODO: Implement command handling
        Ok(MatrixCommandOutcome::NoCommand)
    }

    pub fn check_permission(
        &self,
        _matrix: &MatrixAppservice,
        _sender: &str,
    ) -> MatrixCommandPermission {
        // TODO: Implement permission checking
        MatrixCommandPermission::None
    }
}
