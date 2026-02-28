#![forbid(unsafe_code)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_comparisons)]

use std::sync::Arc;

use anyhow::Result;
use tracing::{error, info};

mod admin;
mod bridge;
mod cache;
mod cli;
mod config;
mod db;
mod imessage;
mod matrix;
mod media;
mod parsers;
mod utils;
mod web;

use config::Config;
use web::WebServer;

#[tokio::main]
async fn main() -> Result<()> {
    utils::logging::init_tracing();

    let config = Arc::new(Config::load()?);
    info!("matrix-imessage bridge starting up");

    let db_manager = Arc::new(db::DatabaseManager::new(&config.database).await?);
    db_manager.migrate().await?;

    let matrix_client = Arc::new(matrix::MatrixAppservice::new(config.clone()).await?);
    let imessage_client = Arc::new(imessage::IMessageClient::new(config.as_ref()).await?);

    let bridge = Arc::new(bridge::BridgeCore::new(
        matrix_client.clone(),
        imessage_client.clone(),
        db_manager.clone(),
    ));

    imessage_client.set_bridge(bridge.clone()).await;

    let web_server = WebServer::new(
        config.clone(),
        matrix_client.clone(),
        db_manager.clone(),
        bridge.clone(),
    )
    .await?;

    let web_handle = tokio::spawn(async move {
        if let Err(e) = web_server.start().await {
            error!("web server error: {}", e);
        }
    });

    let bridge_handle = tokio::spawn(async move {
        if let Err(e) = bridge.start().await {
            error!("bridge error: {}", e);
        }
    });

    tokio::pin!(web_handle);
    tokio::pin!(bridge_handle);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("received Ctrl+C, beginning shutdown");
        },
        _ = &mut web_handle => {
            info!("web server task exited, beginning shutdown");
        },
        _ = &mut bridge_handle => {
            info!("bridge task exited, beginning shutdown");
        },
    }

    web_handle.abort();
    bridge_handle.abort();

    if let Err(err) = imessage_client.stop().await {
        error!("imessage shutdown error: {}", err);
    }

    info!("matrix-imessage bridge shutting down");
    Ok(())
}
