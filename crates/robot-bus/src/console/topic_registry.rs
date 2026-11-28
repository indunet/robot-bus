//! Sticky topic → protobuf type-name registry (control plane only).

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

/// What [`TopicTypeRegistry::register`] did with this name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterOutcome {
    /// First registration of this topic.
    Created,
    /// This name is already the kept type, or this conflict was already recorded.
    Unchanged,
    /// A new conflicting name was recorded. The first type is kept.
    Conflict { kept: String },
}

#[derive(Debug)]
struct Entry {
    type_name: String,
    conflicts: BTreeSet<String>,
}

/// Shared map of topic path → canonical protobuf type name (e.g. `sensor_msgs.msg.v1.Imu`).
///
/// The first name wins. Later different names are recorded and do not replace it.
#[derive(Debug, Default)]
pub struct TopicTypeRegistry {
    topics: Mutex<HashMap<String, Entry>>,
}

impl TopicTypeRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Keep the first type. A different name is a conflict and is recorded once.
    pub fn register(&self, topic: &str, type_name: &str) -> RegisterOutcome {
        let mut map = self.topics.lock().unwrap_or_else(|e| e.into_inner());
        match map.get_mut(topic) {
            None => {
                map.insert(
                    topic.to_string(),
                    Entry {
                        type_name: type_name.to_string(),
                        conflicts: BTreeSet::new(),
                    },
                );
                RegisterOutcome::Created
            }
            Some(entry) if entry.type_name == type_name => RegisterOutcome::Unchanged,
            Some(entry) => {
                if entry.conflicts.insert(type_name.to_string()) {
                    log::warn!(
                        "topic type conflict: {topic} keeping {} ignoring {type_name}",
                        entry.type_name
                    );
                    RegisterOutcome::Conflict {
                        kept: entry.type_name.clone(),
                    }
                } else {
                    RegisterOutcome::Unchanged
                }
            }
        }
    }

    pub fn get(&self, topic: &str) -> Option<String> {
        self.topics
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(topic)
            .map(|entry| entry.type_name.clone())
    }

    /// Conflicting names recorded for `topic`, in sorted order. Empty if none.
    pub fn conflicts(&self, topic: &str) -> Vec<String> {
        self.topics
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(topic)
            .map(|entry| entry.conflicts.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Snapshot of the kept `(topic, type_name)` pairs, sorted by topic.
    pub fn snapshot(&self) -> Vec<(String, String)> {
        let map = self.topics.lock().unwrap_or_else(|e| e.into_inner());
        let mut out: Vec<_> = map
            .iter()
            .map(|(topic, entry)| (topic.clone(), entry.type_name.clone()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Topics that have at least one conflicting name. Names are sorted.
    pub fn conflict_snapshot(&self) -> HashMap<String, Vec<String>> {
        let map = self.topics.lock().unwrap_or_else(|e| e.into_inner());
        map.iter()
            .filter(|(_, entry)| !entry.conflicts.is_empty())
            .map(|(topic, entry)| (topic.clone(), entry.conflicts.iter().cloned().collect()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_keeps_the_first_type_and_dedups_conflicts() {
        let r = TopicTypeRegistry::new();
        assert_eq!(r.register("/a", "pkg.A"), RegisterOutcome::Created);
        assert_eq!(r.get("/a").as_deref(), Some("pkg.A"));
        assert_eq!(r.register("/a", "pkg.A"), RegisterOutcome::Unchanged);
        assert_eq!(
            r.register("/a", "pkg.B"),
            RegisterOutcome::Conflict {
                kept: "pkg.A".into()
            }
        );
        assert_eq!(r.get("/a").as_deref(), Some("pkg.A"));
        assert_eq!(r.register("/a", "pkg.B"), RegisterOutcome::Unchanged);
        assert_eq!(r.conflicts("/a"), vec!["pkg.B".to_string()]);
    }
}
