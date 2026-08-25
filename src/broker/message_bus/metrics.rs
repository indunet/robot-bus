//! Lightweight per-topic counters for the message bus proxy.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Counters for a single topic.
#[derive(Debug, Default)]
pub struct TopicCounters {
    pub total_msgs: AtomicU64,
    pub total_bytes: AtomicU64,
    /// Unix epoch milliseconds of last forwarded message.
    pub last_seen_unix_ms: AtomicU64,
}

/// Snapshot of one topic (owned values for JSON / rate math).
#[derive(Clone, Debug)]
pub struct TopicSnapshot {
    pub name: String,
    pub total_msgs: u64,
    pub total_bytes: u64,
    pub last_seen_unix_ms: u64,
}

/// Aggregate snapshot of the message bus.
#[derive(Clone, Debug, Default)]
pub struct MessageMetricsSnapshot {
    pub total_msgs: u64,
    pub total_bytes: u64,
    pub topics: Vec<TopicSnapshot>,
}

/// Shared message-bus metrics updated on the proxy hot path.
#[derive(Debug, Default)]
pub struct MessageMetrics {
    total_msgs: AtomicU64,
    total_bytes: AtomicU64,
    topics: Mutex<HashMap<String, Arc<TopicCounters>>>,
}

impl MessageMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Record one captured pub/sub message (`[topic][payload…]`).
    ///
    /// Runs on the capture side-thread only — not on the libzmq forward path.
    pub fn record(&self, topic: &str, frame_bytes: u64) {
        self.total_msgs.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(frame_bytes, Ordering::Relaxed);

        let counters = {
            let mut map = self.topics.lock().unwrap_or_else(|e| e.into_inner());
            // Avoid allocating a new String when the topic is already known.
            if let Some(existing) = map.get(topic) {
                existing.clone()
            } else {
                let c = Arc::new(TopicCounters::default());
                map.insert(topic.to_string(), c.clone());
                c
            }
        };
        counters.total_msgs.fetch_add(1, Ordering::Relaxed);
        counters
            .total_bytes
            .fetch_add(frame_bytes, Ordering::Relaxed);
        // Capture side-thread only. Update every message so sub-1 Hz last-seen stays honest.
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        counters.last_seen_unix_ms.store(now_ms, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MessageMetricsSnapshot {
        let map = self.topics.lock().unwrap_or_else(|e| e.into_inner());
        let mut topics: Vec<TopicSnapshot> = map
            .iter()
            .map(|(name, c)| TopicSnapshot {
                name: name.clone(),
                total_msgs: c.total_msgs.load(Ordering::Relaxed),
                total_bytes: c.total_bytes.load(Ordering::Relaxed),
                last_seen_unix_ms: c.last_seen_unix_ms.load(Ordering::Relaxed),
            })
            .collect();
        topics.sort_by(|a, b| a.name.cmp(&b.name));
        MessageMetricsSnapshot {
            total_msgs: self.total_msgs.load(Ordering::Relaxed),
            total_bytes: self.total_bytes.load(Ordering::Relaxed),
            topics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_snapshot() {
        let m = MessageMetrics::new();
        m.record("/a", 10);
        m.record("/a", 5);
        m.record("/b", 3);
        let s = m.snapshot();
        assert_eq!(s.total_msgs, 3);
        assert_eq!(s.total_bytes, 18);
        assert_eq!(s.topics.len(), 2);
        let a = s.topics.iter().find(|t| t.name == "/a").unwrap();
        assert_eq!(a.total_msgs, 2);
        assert_eq!(a.total_bytes, 15);
    }
}
