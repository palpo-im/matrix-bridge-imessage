use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::{debug, error, info, warn};

use crate::db::models::{Backfill as DBBackfill, Message as DBMessage};
use crate::imessage::Message;
use crate::matrix::MatrixClient;

const BACKFILL_BATCH_SIZE: usize = 100;
const BACKFILL_QUEUE_SIZE: usize = 50;
const BACKFILL_DELAY_MS: u64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillConfig {
    pub enable: bool,
    pub initial_limit: usize,
    pub catchup_limit: usize,
    pub unread_hours_threshold: i64,
    pub queue_size: usize,
    pub batch_size: usize,
    pub delay_ms: u64,
}

impl Default for BackfillConfig {
    fn default() -> Self {
        Self {
            enable: true,
            initial_limit: 100,
            catchup_limit: 50,
            unread_hours_threshold: 720,
            queue_size: BACKFILL_QUEUE_SIZE,
            batch_size: BACKFILL_BATCH_SIZE,
            delay_ms: BACKFILL_DELAY_MS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackfillTask {
    pub portal_guid: String,
    pub backfill_id: String,
    pub messages: Vec<Message>,
    pub is_initial: bool,
    pub priority: BackfillPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackfillPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

pub struct BackfillQueue {
    tasks: Mutex<VecDeque<BackfillTask>>,
    notify_tx: Sender<()>,
    notify_rx: Mutex<Option<Receiver<()>>>,
    config: BackfillConfig,
}

impl BackfillQueue {
    pub fn new(config: BackfillConfig) -> Self {
        let (notify_tx, notify_rx) = mpsc::channel(1);
        
        Self {
            tasks: Mutex::new(VecDeque::with_capacity(config.queue_size)),
            notify_tx,
            notify_rx: Mutex::new(Some(notify_rx)),
            config,
        }
    }

    pub fn push(&self, task: BackfillTask) -> Result<(), String> {
        let mut tasks = self.tasks.lock();
        
        if tasks.len() >= self.config.queue_size {
            return Err("Backfill queue is full".to_string());
        }
        
        let pos = tasks.iter().position(|t| t.priority < task.priority).unwrap_or(tasks.len());
        tasks.insert(pos, task);
        
        let _ = self.notify_tx.try_send(());
        
        Ok(())
    }

    pub fn pop(&self) -> Option<BackfillTask> {
        self.tasks.lock().pop_front()
    }

    pub fn len(&self) -> usize {
        self.tasks.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.lock().is_empty()
    }

    pub fn clear(&self) {
        self.tasks.lock().clear();
    }

    pub fn take_receiver(&self) -> Option<Receiver<()>> {
        self.notify_rx.lock().take()
    }
}

pub struct BackfillManager {
    queue: Arc<BackfillQueue>,
    matrix: Arc<MatrixClient>,
    config: BackfillConfig,
}

impl BackfillManager {
    pub fn new(matrix: Arc<MatrixClient>, config: BackfillConfig) -> Self {
        Self {
            queue: Arc::new(BackfillQueue::new(config.clone())),
            matrix,
            config,
        }
    }

    pub fn queue(&self) -> Arc<BackfillQueue> {
        self.queue.clone()
    }

    pub async fn start_processing(&self) {
        if !self.config.enable {
            info!("Backfill is disabled");
            return;
        }

        info!("Starting backfill processing loop");

        let mut notify_rx = self.queue.take_receiver();
        
        loop {
            while let Some(task) = self.queue.pop() {
                if let Err(e) = self.process_task(task).await {
                    error!("Failed to process backfill task: {}", e);
                }
            }

            if let Some(ref mut rx) = notify_rx {
                match rx.recv().await {
                    Some(_) => continue,
                    None => break,
                }
            } else {
                break;
            }
        }

        info!("Backfill processing loop stopped");
    }

    async fn process_task(&self, task: BackfillTask) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Processing backfill task {} for portal {} ({} messages)",
            task.backfill_id,
            task.portal_guid,
            task.messages.len()
        );

        let portal_mxid = match self.matrix.get_room_id(&task.portal_guid).await? {
            Some(mxid) => mxid,
            None => {
                warn!("Portal {} has no Matrix room, skipping backfill", task.portal_guid);
                return Ok(());
            }
        };

        let batches = self.create_batches(&task.messages);
        
        for (batch_idx, batch) in batches.into_iter().enumerate() {
            debug!(
                "Processing batch {}/{} for backfill {}",
                batch_idx + 1,
                task.messages.len() / self.config.batch_size + 1,
                task.backfill_id
            );

            if let Err(e) = self.process_batch(&portal_mxid, batch, &task).await {
                error!("Failed to process batch {}: {}", batch_idx, e);
                
                DBBackfill::update_status(&task.backfill_id, false, Some(&e.to_string()))?;
                
                return Err(e);
            }

            if self.config.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.delay_ms)).await;
            }
        }

        DBBackfill::update_status(&task.backfill_id, true, None)?;
        
        info!("Completed backfill task {}", task.backfill_id);
        
        Ok(())
    }

    fn create_batches(&self, messages: &[Message]) -> Vec<Vec<Message>> {
        messages
            .chunks(self.config.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    async fn process_batch(
        &self,
        portal_mxid: &str,
        batch: Vec<Message>,
        task: &BackfillTask,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut events = Vec::new();
        
        for msg in batch {
            if DBMessage::get_by_guid(&task.portal_guid, &msg.guid, 0)?.is_some() {
                debug!("Skipping duplicate message {}", msg.guid);
                continue;
            }

            let event = self.matrix.convert_message_to_event(&msg).await?;
            events.push(event);
        }

        if events.is_empty() {
            debug!("No events to send in this batch");
            return Ok(());
        }

        let is_read = self.determine_read_status(&task.messages, task.is_initial);

        self.matrix.send_batch_events(portal_mxid, events, is_read).await?;

        Ok(())
    }

    fn determine_read_status(&self, messages: &[Message], is_initial: bool) -> bool {
        if is_initial {
            let threshold = chrono::Utc::now() - chrono::Duration::hours(self.config.unread_hours_threshold);
            
            messages.iter().all(|msg| {
                msg.is_from_me || msg.is_read || msg.timestamp < threshold
            })
        } else {
            true
        }
    }

    pub async fn schedule_initial_backfill(
        &self,
        portal_guid: &str,
        messages: Vec<Message>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let backfill_id = format!(
            "initial-{}-{}",
            portal_guid,
            chrono::Utc::now().timestamp_millis()
        );

        let task = BackfillTask {
            portal_guid: portal_guid.to_string(),
            backfill_id: backfill_id.clone(),
            messages,
            is_initial: true,
            priority: BackfillPriority::High,
        };

        DBBackfill::create(&backfill_id, portal_guid, task.messages.len(), true)?;

        self.queue.push(task)?;

        info!("Scheduled initial backfill {} for portal {}", backfill_id, portal_guid);
        
        Ok(backfill_id)
    }

    pub async fn schedule_catchup_backfill(
        &self,
        portal_guid: &str,
        messages: Vec<Message>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let backfill_id = format!(
            "catchup-{}-{}",
            portal_guid,
            chrono::Utc::now().timestamp_millis()
        );

        let task = BackfillTask {
            portal_guid: portal_guid.to_string(),
            backfill_id: backfill_id.clone(),
            messages,
            is_initial: false,
            priority: BackfillPriority::Normal,
        };

        DBBackfill::create(&backfill_id, portal_guid, task.messages.len(), false)?;

        self.queue.push(task)?;

        info!("Scheduled catchup backfill {} for portal {}", backfill_id, portal_guid);
        
        Ok(backfill_id)
    }

    pub fn get_queue_status(&self) -> BackfillQueueStatus {
        let tasks = self.queue.len();
        
        BackfillQueueStatus {
            queued_tasks: tasks,
            is_processing: tasks > 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackfillQueueStatus {
    pub queued_tasks: usize,
    pub is_processing: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backfill_queue_priority() {
        let config = BackfillConfig::default();
        let queue = BackfillQueue::new(config);

        let low_task = BackfillTask {
            portal_guid: "portal1".to_string(),
            backfill_id: "low".to_string(),
            messages: vec![],
            is_initial: false,
            priority: BackfillPriority::Low,
        };

        let high_task = BackfillTask {
            portal_guid: "portal2".to_string(),
            backfill_id: "high".to_string(),
            messages: vec![],
            is_initial: false,
            priority: BackfillPriority::High,
        };

        queue.push(low_task.clone()).unwrap();
        queue.push(high_task.clone()).unwrap();

        let first = queue.pop().unwrap();
        assert_eq!(first.priority, BackfillPriority::High);
    }

    #[test]
    fn test_backfill_config_default() {
        let config = BackfillConfig::default();
        assert!(config.enable);
        assert_eq!(config.initial_limit, 100);
        assert_eq!(config.batch_size, 100);
    }
}
