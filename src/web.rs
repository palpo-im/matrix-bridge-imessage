use std::sync::Arc;

use anyhow::Result;

use crate::bridge::BridgeCore;
use crate::db::DatabaseManager;
use crate::matrix::MatrixAppservice;
use crate::config::Config;

pub struct WebServer {
    config: Arc<Config>,
    matrix_client: Arc<MatrixAppservice>,
    db_manager: Arc<DatabaseManager>,
    bridge: Arc<BridgeCore>,
}

impl WebServer {
    pub async fn new(
        config: Arc<Config>,
        matrix_client: Arc<MatrixAppservice>,
        db_manager: Arc<DatabaseManager>,
        bridge: Arc<BridgeCore>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            matrix_client,
            db_manager,
            bridge,
        })
    }

    pub async fn start(&self) -> Result<()> {
        tracing::info!("starting web server on {}:{}", 
                       self.config.bridge.bind_address, 
                       self.config.bridge.port);
        
        // TODO: Implement actual web server with Salvo
        
        Ok(())
    }
}
