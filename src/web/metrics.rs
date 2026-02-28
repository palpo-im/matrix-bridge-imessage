use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use salvo::prelude::*;

static MATRIX_MESSAGES_RECEIVED: AtomicU64 = AtomicU64::new(0);
static MATRIX_MESSAGES_SUCCESS: AtomicU64 = AtomicU64::new(0);
static MATRIX_MESSAGES_FAILED: AtomicU64 = AtomicU64::new(0);
static IMESSAGE_MESSAGES_RECEIVED: AtomicU64 = AtomicU64::new(0);
static IMESSAGE_MESSAGES_SUCCESS: AtomicU64 = AtomicU64::new(0);
static IMESSAGE_MESSAGES_FAILED: AtomicU64 = AtomicU64::new(0);
static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
static BACKFILL_QUEUE_SIZE: AtomicU64 = AtomicU64::new(0);
static MESSAGES_LATENCY_MS: AtomicU64 = AtomicU64::new(0);
static MESSAGES_LATENCY_COUNT: AtomicU64 = AtomicU64::new(0);
static ACTIVE_PORTALS: AtomicU64 = AtomicU64::new(0);
static BRIDGED_ROOMS: AtomicU64 = AtomicU64::new(0);
static ERROR_COUNT: AtomicU64 = AtomicU64::new(0);
static EDITS_PROCESSED: AtomicU64 = AtomicU64::new(0);
static DELETES_PROCESSED: AtomicU64 = AtomicU64::new(0);
static ATTACHMENTS_UPLOADED: AtomicU64 = AtomicU64::new(0);
static TAPBACKS_PROCESSED: AtomicU64 = AtomicU64::new(0);
static BACKFILL_TASKS_COMPLETED: AtomicU64 = AtomicU64::new(0);

pub struct Metrics {
    started_at: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }

    pub fn matrix_message_received() {
        MATRIX_MESSAGES_RECEIVED.fetch_add(1, Ordering::Relaxed);
    }

    pub fn matrix_message_success() {
        MATRIX_MESSAGES_SUCCESS.fetch_add(1, Ordering::Relaxed);
    }

    pub fn matrix_message_failed() {
        MATRIX_MESSAGES_FAILED.fetch_add(1, Ordering::Relaxed);
    }

    pub fn imessage_message_received() {
        IMESSAGE_MESSAGES_RECEIVED.fetch_add(1, Ordering::Relaxed);
    }

    pub fn imessage_message_success() {
        IMESSAGE_MESSAGES_SUCCESS.fetch_add(1, Ordering::Relaxed);
    }

    pub fn imessage_message_failed() {
        IMESSAGE_MESSAGES_FAILED.fetch_add(1, Ordering::Relaxed);
    }

    pub fn cache_hit() {
        CACHE_HITS.fetch_add(1, Ordering::Relaxed);
    }

    pub fn cache_miss() {
        CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_backfill_queue_size(size: u64) {
        BACKFILL_QUEUE_SIZE.store(size, Ordering::Relaxed);
    }

    pub fn record_latency(latency_ms: u64) {
        MESSAGES_LATENCY_MS.fetch_add(latency_ms, Ordering::Relaxed);
        MESSAGES_LATENCY_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_active_portals(count: u64) {
        ACTIVE_PORTALS.store(count, Ordering::Relaxed);
    }

    pub fn set_bridged_rooms(count: u64) {
        BRIDGED_ROOMS.store(count, Ordering::Relaxed);
    }

    pub fn error_occurred() {
        ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    pub fn edit_processed() {
        EDITS_PROCESSED.fetch_add(1, Ordering::Relaxed);
    }

    pub fn delete_processed() {
        DELETES_PROCESSED.fetch_add(1, Ordering::Relaxed);
    }

    pub fn attachment_uploaded() {
        ATTACHMENTS_UPLOADED.fetch_add(1, Ordering::Relaxed);
    }

    pub fn tapback_processed() {
        TAPBACKS_PROCESSED.fetch_add(1, Ordering::Relaxed);
    }

    pub fn backfill_task_completed() {
        BACKFILL_TASKS_COMPLETED.fetch_add(1, Ordering::Relaxed);
    }

    pub fn reset() {
        MATRIX_MESSAGES_RECEIVED.store(0, Ordering::Relaxed);
        MATRIX_MESSAGES_SUCCESS.store(0, Ordering::Relaxed);
        MATRIX_MESSAGES_FAILED.store(0, Ordering::Relaxed);
        IMESSAGE_MESSAGES_RECEIVED.store(0, Ordering::Relaxed);
        IMESSAGE_MESSAGES_SUCCESS.store(0, Ordering::Relaxed);
        IMESSAGE_MESSAGES_FAILED.store(0, Ordering::Relaxed);
        CACHE_HITS.store(0, Ordering::Relaxed);
        CACHE_MISSES.store(0, Ordering::Relaxed);
        MESSAGES_LATENCY_MS.store(0, Ordering::Relaxed);
        MESSAGES_LATENCY_COUNT.store(0, Ordering::Relaxed);
        ERROR_COUNT.store(0, Ordering::Relaxed);
        EDITS_PROCESSED.store(0, Ordering::Relaxed);
        DELETES_PROCESSED.store(0, Ordering::Relaxed);
        ATTACHMENTS_UPLOADED.store(0, Ordering::Relaxed);
        TAPBACKS_PROCESSED.store(0, Ordering::Relaxed);
        BACKFILL_TASKS_COMPLETED.store(0, Ordering::Relaxed);
    }

    pub fn snapshot() -> MetricsSnapshot {
        let avg_latency = {
            let total_ms = MESSAGES_LATENCY_MS.load(Ordering::Relaxed);
            let count = MESSAGES_LATENCY_COUNT.load(Ordering::Relaxed);
            if count > 0 {
                total_ms / count
            } else {
                0
            }
        };

        MetricsSnapshot {
            matrix_messages_received: MATRIX_MESSAGES_RECEIVED.load(Ordering::Relaxed),
            matrix_messages_success: MATRIX_MESSAGES_SUCCESS.load(Ordering::Relaxed),
            matrix_messages_failed: MATRIX_MESSAGES_FAILED.load(Ordering::Relaxed),
            imessage_messages_received: IMESSAGE_MESSAGES_RECEIVED.load(Ordering::Relaxed),
            imessage_messages_success: IMESSAGE_MESSAGES_SUCCESS.load(Ordering::Relaxed),
            imessage_messages_failed: IMESSAGE_MESSAGES_FAILED.load(Ordering::Relaxed),
            cache_hits: CACHE_HITS.load(Ordering::Relaxed),
            cache_misses: CACHE_MISSES.load(Ordering::Relaxed),
            backfill_queue_size: BACKFILL_QUEUE_SIZE.load(Ordering::Relaxed),
            avg_latency_ms: avg_latency,
            active_portals: ACTIVE_PORTALS.load(Ordering::Relaxed),
            bridged_rooms: BRIDGED_ROOMS.load(Ordering::Relaxed),
            error_count: ERROR_COUNT.load(Ordering::Relaxed),
            edits_processed: EDITS_PROCESSED.load(Ordering::Relaxed),
            deletes_processed: DELETES_PROCESSED.load(Ordering::Relaxed),
            attachments_uploaded: ATTACHMENTS_UPLOADED.load(Ordering::Relaxed),
            tapbacks_processed: TAPBACKS_PROCESSED.load(Ordering::Relaxed),
            backfill_tasks_completed: BACKFILL_TASKS_COMPLETED.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct MetricsSnapshot {
    pub matrix_messages_received: u64,
    pub matrix_messages_success: u64,
    pub matrix_messages_failed: u64,
    pub imessage_messages_received: u64,
    pub imessage_messages_success: u64,
    pub imessage_messages_failed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub backfill_queue_size: u64,
    pub avg_latency_ms: u64,
    pub active_portals: u64,
    pub bridged_rooms: u64,
    pub error_count: u64,
    pub edits_processed: u64,
    pub deletes_processed: u64,
    pub attachments_uploaded: u64,
    pub tapbacks_processed: u64,
    pub backfill_tasks_completed: u64,
}

#[handler]
pub async fn metrics_endpoint(res: &mut Response) {
    let snapshot = Metrics::snapshot();
    res.render(Json(snapshot));
}
