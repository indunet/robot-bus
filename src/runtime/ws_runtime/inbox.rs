use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Instant;

use crate::errors::{BusError, Result};

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
struct TopicInbox {
    queues: HashMap<u64, Pending>,
    ready: VecDeque<u64>,
    /// Threads blocked in [`SharedInbox::pop_or_wait`]. Guarded by the same mutex
    /// as the queues, so a producer only pays for a wake-up when someone waits.
    waiters: usize,
}

impl TopicInbox {
    fn add(&mut self, id: u64, filter: &str, capacity: usize) {
        self.queues.insert(
            id,
            Pending {
                filter: filter.into(),
                capacity,
                messages: VecDeque::new(),
            },
        );
    }

    fn remove(&mut self, id: u64) {
        self.queues.remove(&id);
        self.ready.retain(|queued| *queued != id);
    }

    fn push(&mut self, id: u64, payload: Vec<u8>) {
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

    fn pop(&mut self) -> Option<TopicEvent> {
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

/// Topic inbox shared between the connection task (producer) and the spinning
/// thread (consumer).
///
/// The consumer blocks on a condition variable instead of sleeping and polling,
/// so a message that arrives is dispatched immediately rather than after up to a
/// poll interval.
#[derive(Default)]
pub(super) struct SharedInbox {
    queue: Mutex<TopicInbox>,
    arrived: Condvar,
}

fn poisoned() -> BusError {
    BusError::Protocol("ws inbox mutex poisoned".into())
}

impl SharedInbox {
    fn lock(&self) -> Result<MutexGuard<'_, TopicInbox>> {
        self.queue.lock().map_err(|_| poisoned())
    }

    pub(super) fn add(&self, id: u64, filter: &str, capacity: usize) -> Result<()> {
        self.lock()?.add(id, filter, capacity);
        Ok(())
    }

    pub(super) fn remove(&self, id: u64) -> Result<()> {
        self.lock()?.remove(id);
        Ok(())
    }

    /// Enqueue a payload and wake the spinning thread if it is waiting.
    pub(super) fn push(&self, id: u64, payload: Vec<u8>) {
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(id, payload);
        let waiting = queue.waiters > 0;
        drop(queue);
        if waiting {
            self.arrived.notify_all();
        }
    }

    /// Wake a waiting consumer so it re-checks `running`.
    pub(super) fn wake(&self) {
        // Taking the lock orders this wake-up after a consumer's `running` check
        // and before its wait, so the wake-up cannot be lost in between.
        if let Ok(queue) = self.queue.lock() {
            drop(queue);
        }
        self.arrived.notify_all();
    }

    /// Next queued event, blocking until one arrives, `deadline` passes, or
    /// `running` becomes false.
    pub(super) fn pop_or_wait(
        &self,
        deadline: Instant,
        running: &AtomicBool,
    ) -> Result<Option<TopicEvent>> {
        let mut queue = self.lock()?;
        loop {
            if let Some(event) = queue.pop() {
                return Ok(Some(event));
            }
            if !running.load(Ordering::Acquire) {
                return Ok(None);
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }
            queue.waiters += 1;
            let waited = self.arrived.wait_timeout(queue, deadline - now);
            let (mut guard, _) = waited.map_err(|_| poisoned())?;
            guard.waiters -= 1;
            queue = guard;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn inbox_with_topic() -> Arc<SharedInbox> {
        let inbox = Arc::new(SharedInbox::default());
        inbox.add(1, "t", 8).unwrap();
        inbox
    }

    #[test]
    fn queued_event_is_returned_without_waiting() {
        let inbox = inbox_with_topic();
        inbox.push(1, b"a".to_vec());
        let running = AtomicBool::new(true);
        let start = Instant::now();
        let event = inbox
            .pop_or_wait(start + Duration::from_secs(5), &running)
            .unwrap()
            .expect("event");
        assert_eq!(event.payload, b"a");
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn waiting_consumer_wakes_when_a_message_arrives() {
        let inbox = inbox_with_topic();
        let running = Arc::new(AtomicBool::new(true));
        let producer = {
            let inbox = Arc::clone(&inbox);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(50));
                inbox.push(1, b"late".to_vec());
            })
        };
        let start = Instant::now();
        let event = inbox
            .pop_or_wait(start + Duration::from_secs(5), &running)
            .unwrap()
            .expect("event");
        assert_eq!(event.payload, b"late");
        // Woken by the push, not by the 5 s deadline.
        assert!(start.elapsed() < Duration::from_secs(2));
        producer.join().unwrap();
    }

    #[test]
    fn deadline_ends_the_wait_when_nothing_arrives() {
        let inbox = inbox_with_topic();
        let running = AtomicBool::new(true);
        let start = Instant::now();
        let event = inbox
            .pop_or_wait(start + Duration::from_millis(40), &running)
            .unwrap();
        assert!(event.is_none());
        assert!(start.elapsed() >= Duration::from_millis(40));
    }

    #[test]
    fn clearing_running_and_waking_ends_the_wait_early() {
        let inbox = inbox_with_topic();
        let running = Arc::new(AtomicBool::new(true));
        let stopper = {
            let inbox = Arc::clone(&inbox);
            let running = Arc::clone(&running);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(50));
                running.store(false, Ordering::Release);
                inbox.wake();
            })
        };
        let start = Instant::now();
        let event = inbox
            .pop_or_wait(start + Duration::from_secs(5), &running)
            .unwrap();
        assert!(event.is_none());
        assert!(start.elapsed() < Duration::from_secs(2));
        stopper.join().unwrap();
    }

    #[test]
    fn overflow_keeps_only_the_newest_messages() {
        let inbox = Arc::new(SharedInbox::default());
        inbox.add(1, "t", 2).unwrap();
        for i in 0..4u8 {
            inbox.push(1, vec![i]);
        }
        let running = AtomicBool::new(true);
        let now = Instant::now();
        let first = inbox.pop_or_wait(now, &running).unwrap().unwrap();
        let second = inbox.pop_or_wait(now, &running).unwrap().unwrap();
        assert_eq!((first.payload, second.payload), (vec![2], vec![3]));
        assert!(inbox.pop_or_wait(now, &running).unwrap().is_none());
    }
}
