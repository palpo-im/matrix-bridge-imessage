use salvo::prelude::*;
use serde::{Deserialize, Serialize};

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

fn not_implemented(res: &mut Response) {
    res.status_code(StatusCode::NOT_IMPLEMENTED);
    res.render(Json(serde_json::json!({
        "error": "Provisioning endpoints are not implemented in this build"
    })));
}

#[handler]
pub async fn list_rooms(res: &mut Response) {
    res.render(Json(Vec::<BridgeResponse>::new()));
}

#[handler]
pub async fn create_bridge(_req: &mut Request, res: &mut Response) {
    not_implemented(res);
}

#[handler]
pub async fn get_bridge_info(_req: &mut Request, res: &mut Response) {
    not_implemented(res);
}

#[handler]
pub async fn delete_bridge(_req: &mut Request, res: &mut Response) {
    not_implemented(res);
}
