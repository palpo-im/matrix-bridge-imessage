use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::Instant;

pub struct AsyncTimedCache<T> {
    data: RwLock<Option<(T, Instant)>>,
    ttl: Duration,
}

impl<T: Clone> AsyncTimedCache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: RwLock::new(None),
            ttl,
        }
    }

    pub async fn get(&self) -> Option<T> {
        let data = self.data.read().await;
        if let Some((value, timestamp)) = data.as_ref() {
            if timestamp.elapsed() < self.ttl {
                return Some(value.clone());
            }
        }
        None
    }

    pub async fn set(&self, value: T) {
        let mut data = self.data.write().await;
        *data = Some((value, Instant::now()));
    }

    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        *data = None;
    }
}
