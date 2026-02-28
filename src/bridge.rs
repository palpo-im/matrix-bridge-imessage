use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use crate::db::DatabaseManager;
use crate::imessage::IMessageClient;
use crate::matrix::MatrixAppservice;

pub mod logic;
pub mod message_flow;
pub mod presence_handler;
pub mod provisioning;
pub mod queue;
pub mod user_sync;

#[derive(Clone)]
pub struct BridgeCore {
    matrix_client: Arc<MatrixAppservice>,
    imessage_client: Arc<IMessageClient>,
    db_manager: Arc<DatabaseManager>,
}

impl BridgeCore {
    pub fn new(
        matrix_client: Arc<MatrixAppservice>,
        imessage_client: Arc<IMessageClient>,
        db_manager: Arc<DatabaseManager>,
    ) -> Self {
        Self {
            matrix_client,
            imessage_client,
            db_manager,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("starting bridge core");
        
        // Start iMessage client
        self.imessage_client.start().await?;
        
        info!("bridge core started successfully");
        
        // Keep the bridge running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    pub fn matrix_client(&self) -> &MatrixAppservice {
        &self.matrix_client
    }

    pub fn imessage_client(&self) -> &IMessageClient {
        &self.imessage_client
    }

    pub fn db_manager(&self) -> &DatabaseManager {
        &self.db_manager
    }
}
