use std::sync::Arc;

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::web::web_state;

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeRequest {
    pub user_id: String,
    pub chat_guid: String,
}

#[derive(Debug, Serialize)]
pub struct BridgeResponse {
    pub room_id: String,
    pub chat_guid: String,
}

#[handler]
pub async fn list_rooms(res: &mut Response) {
    let state = web_state();
    
    match state.db_manager.get_all_portals() {
        Ok(portals) => {
            let rooms: Vec<BridgeResponse> = portals
                .into_iter()
                .filter_map(|p| {
                    p.mxid.map(|mxid| BridgeResponse {
                        room_id: mxid,
                        chat_guid: p.guid,
                    })
                })
                .collect();
            
            res.render(Json(rooms));
        }
        Err(e) => {
            error!("Failed to list rooms: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to list rooms"
            })));
        }
    }
}

#[handler]
pub async fn create_bridge(req: &mut Request, res: &mut Response) {
    let state = web_state();
    
    let bridge_req: BridgeRequest = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to parse bridge request: {}", e);
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body"
            })));
            return;
        }
    };

    debug!("Creating bridge for user {} and chat {}", bridge_req.user_id, bridge_req.chat_guid);

    match state.bridge.create_portal_room(&bridge_req.chat_guid, &bridge_req.user_id).await {
        Ok(room_id) => {
            info!("Created bridge room {} for chat {}", room_id, bridge_req.chat_guid);
            res.render(Json(BridgeResponse {
                room_id,
                chat_guid: bridge_req.chat_guid,
            }));
        }
        Err(e) => {
            error!("Failed to create bridge: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": format!("Failed to create bridge: {}", e)
            })));
        }
    }
}

#[handler]
pub async fn get_bridge_info(req: &mut Request, res: &mut Response) {
    let state = web_state();
    let chat_guid = req.param::<String>("id").unwrap_or_default();

    debug!("Getting bridge info for chat {}", chat_guid);

    match state.db_manager.get_portal_by_guid(&chat_guid) {
        Ok(Some(portal)) => {
            if let Some(mxid) = portal.mxid {
                res.render(Json(BridgeResponse {
                    room_id: mxid,
                    chat_guid: portal.guid,
                }));
            } else {
                res.status_code(StatusCode::NOT_FOUND);
                res.render(Json(serde_json::json!({
                    "error": "Portal exists but has no Matrix room"
                })));
            }
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Portal not found"
            })));
        }
        Err(e) => {
            error!("Failed to get bridge info: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to get bridge info"
            })));
        }
    }
}

#[handler]
pub async fn delete_bridge(req: &mut Request, res: &mut Response) {
    let state = web_state();
    let chat_guid = req.param::<String>("id").unwrap_or_default();

    debug!("Deleting bridge for chat {}", chat_guid);

    match state.bridge.delete_portal(&chat_guid).await {
        Ok(_) => {
            info!("Deleted bridge for chat {}", chat_guid);
            res.render(Json(serde_json::json!({
                "success": true
            })));
        }
        Err(e) => {
            error!("Failed to delete bridge: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": format!("Failed to delete bridge: {}", e)
            })));
        }
    }
}
