use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

pub(super) struct TopicEvent {
    pub(super) id: u64,
    pub(super) filter: Arc<str>,
    pub(super) payload: Vec<u8>,
}

struct Pending {
    filter: Arc<str>,
    capacity: usize,
    messages: VecDeque<Vec<u8>>,
}

/// The broker bounds its own queue. Keep the SDK's callback backlog bounded too,
/// without blocking the connection task while the application is not spinning.
#[derive(Default)]
pub(super) struct TopicInbox {
    queues: HashMap<u64, Pending>,
    ready: VecDeque<u64>,
}

impl TopicInbox {
    pub(super) fn add(&mut self, id: u64, filter: &str, capacity: usize) {
        self.queues.insert(
            id,
            Pending {
                filter: filter.into(),
                capacity,
                messages: VecDeque::new(),
            },
        );
    }

    pub(super) fn remove(&mut self, id: u64) {
        self.queues.remove(&id);
        self.ready.retain(|queued| *queued != id);
    }

    pub(super) fn push(&mut self, id: u64, payload: Vec<u8>) {
        let Some(queue) = self.queues.get_mut(&id) else {
            return;
        };
        if queue.messages.is_empty() {
            self.ready.push_back(id);
        }
        if queue.messages.len() == queue.capacity {
            queue.messages.pop_front();
        }
        queue.messages.push_back(payload);
    }

    pub(super) fn pop(&mut self) -> Option<TopicEvent> {
        let id = self.ready.pop_front()?;
        let queue = self.queues.get_mut(&id).expect("ready topic queue");
        let payload = queue.messages.pop_front().expect("ready topic payload");
        if !queue.messages.is_empty() {
            self.ready.push_back(id);
        }
        Some(TopicEvent {
            id,
            filter: Arc::clone(&queue.filter),
            payload,
        })
    }
}
