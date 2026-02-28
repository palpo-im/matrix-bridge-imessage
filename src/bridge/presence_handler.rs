use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

pub struct PresenceHandler {
    presence_cache: RwLock<HashMap<String, MatrixPresenceState>>,
}

#[derive(Debug, Clone)]
pub struct MatrixPresenceState {
    pub user_id: String,
    pub presence: String,
    pub last_active: Option<u64>,
}

impl PresenceHandler {
    pub fn new() -> Self {
        Self {
            presence_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn update_presence(&self, user_id: &str, presence: &str) {
        let mut cache = self.presence_cache.write();
        cache.insert(
            user_id.to_string(),
            MatrixPresenceState {
                user_id: user_id.to_string(),
                presence: presence.to_string(),
                last_active: Some(chrono::Utc::now().timestamp() as u64),
            },
        );
    }

    pub fn get_presence(&self, user_id: &str) -> Option<MatrixPresenceState> {
        let cache = self.presence_cache.read();
        cache.get(user_id).cloned()
    }
}

impl Default for PresenceHandler {
    fn default() -> Self {
        Self::new()
    }
}
