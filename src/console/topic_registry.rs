//! Sticky topic → protobuf type-name registry (control plane only).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Shared map of topic path → canonical protobuf type name (e.g. `sensor_msgs.msg.v1.Imu`).
#[derive(Debug, Default)]
pub struct TopicTypeRegistry {
    topics: Mutex<HashMap<String, String>>,
}

impl TopicTypeRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Register or overwrite. Returns the previous type name when it differed.
    pub fn register(&self, topic: &str, type_name: &str) -> Option<String> {
        let mut map = self.topics.lock().unwrap_or_else(|e| e.into_inner());
        match map.insert(topic.to_string(), type_name.to_string()) {
            Some(prev) if prev != type_name => {
                log::warn!(
                    "topic type register overwrite: {topic} {prev} -> {type_name}"
                );
                Some(prev)
            }
            Some(prev) => Some(prev),
            None => None,
        }
    }

    pub fn get(&self, topic: &str) -> Option<String> {
        self.topics
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(topic)
            .cloned()
    }

    /// Snapshot of all registered `(topic, type_name)` pairs, sorted by topic.
    pub fn snapshot(&self) -> Vec<(String, String)> {
        let map = self.topics.lock().unwrap_or_else(|e| e.into_inner());
        let mut out: Vec<_> = map
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_get_and_overwrite() {
        let r = TopicTypeRegistry::new();
        assert!(r.register("/a", "pkg.A").is_none());
        assert_eq!(r.get("/a").as_deref(), Some("pkg.A"));
        let prev = r.register("/a", "pkg.B");
        assert_eq!(prev.as_deref(), Some("pkg.A"));
        assert_eq!(r.get("/a").as_deref(), Some("pkg.B"));
    }
}
