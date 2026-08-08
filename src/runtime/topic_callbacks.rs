//! Shared topic → subscription callback matching for executors.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::runtime::callback_group::SubscriptionCallback;

/// Invoke `f` for each unique callback matching `topic` (exact or non-empty prefix).
pub fn for_each_matching_callback(
    topic: &str,
    topic_callbacks: &HashMap<String, Vec<SubscriptionCallback>>,
    mut f: impl FnMut(&SubscriptionCallback),
) {
    let mut seen = HashSet::new();

    if let Some(exact) = topic_callbacks.get(topic) {
        for entry in exact {
            let ptr = Arc::as_ptr(&entry.callback) as *const ();
            if seen.insert(ptr) {
                f(entry);
            }
        }
    }

    for (pattern, callbacks) in topic_callbacks {
        if pattern.as_str() == topic {
            continue;
        }
        if pattern.is_empty() || !topic.starts_with(pattern.as_str()) {
            continue;
        }
        for entry in callbacks {
            let ptr = Arc::as_ptr(&entry.callback) as *const ();
            if seen.insert(ptr) {
                f(entry);
            }
        }
    }
}
