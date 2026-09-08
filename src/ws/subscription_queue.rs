//! Bounded, replaceable pending messages for one WebSocket subscription.
use super::{rpc_status::RpcStatus, sub_demux::BusMsg};
use crate::runtime::{SubscriptionOverflowPolicy, ws_subscribe_queue_capacity};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionStats {
    pub id: u64,
    pub filter: String,
    pub policy: SubscriptionOverflowPolicy,
    pub capacity: usize,
    pub pending: usize,
    pub received: u64,
    pub dequeued: u64,
    pub dropped: u64,
}
struct State {
    messages: VecDeque<BusMsg>,
    closed: bool,
    error: Option<RpcStatus>,
    received: u64,
    dequeued: u64,
    dropped: u64,
}
pub struct SubscriptionQueue {
    id: u64,
    filter: String,
    policy: SubscriptionOverflowPolicy,
    capacity: usize,
    state: Mutex<State>,
    ready: Notify,
    total_dropped: Arc<AtomicU64>,
}
impl SubscriptionQueue {
    pub(super) fn new(
        id: u64,
        filter: String,
        depth: i32,
        policy: SubscriptionOverflowPolicy,
        total_dropped: Arc<AtomicU64>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            filter,
            policy,
            capacity: if policy == SubscriptionOverflowPolicy::Latest {
                1
            } else {
                ws_subscribe_queue_capacity(depth)
            },
            state: Mutex::new(State {
                messages: VecDeque::new(),
                closed: false,
                error: None,
                received: 0,
                dequeued: 0,
                dropped: 0,
            }),
            ready: Notify::new(),
            total_dropped,
        })
    }
    pub(super) fn push(&self, message: BusMsg) {
        let mut state = self.state.lock().unwrap();
        if state.closed {
            return;
        }
        state.received += 1;
        if state.messages.len() == self.capacity {
            state.dropped += 1;
            self.total_dropped.fetch_add(1, Ordering::Relaxed);
            if self.policy == SubscriptionOverflowPolicy::DropNewest {
                return;
            }
            state.messages.pop_front();
        }
        state.messages.push_back(message);
        drop(state);
        self.ready.notify_one();
    }
    pub(super) fn fail(&self, error: RpcStatus) {
        let mut state = self.state.lock().unwrap();
        state.error = Some(error);
        state.closed = true;
        state.messages.clear();
        drop(state);
        self.ready.notify_one();
    }
    pub(super) fn is_closed(&self) -> bool {
        self.state.lock().unwrap().closed
    }
    pub fn stats(&self) -> SubscriptionStats {
        let state = self.state.lock().unwrap();
        SubscriptionStats {
            id: self.id,
            filter: self.filter.clone(),
            policy: self.policy,
            capacity: self.capacity,
            pending: state.messages.len(),
            received: state.received,
            dequeued: state.dequeued,
            dropped: state.dropped,
        }
    }
    pub(super) async fn wait_ready(&self) -> Result<bool, RpcStatus> {
        loop {
            let notified = self.ready.notified();
            {
                let state = self.state.lock().unwrap();
                if let Some(error) = &state.error {
                    return Err(error.clone());
                }
                if state.closed {
                    return Ok(false);
                }
                if !state.messages.is_empty() {
                    return Ok(true);
                }
            }
            notified.await;
        }
    }
    pub(super) fn pop(&self) -> Option<BusMsg> {
        let mut state = self.state.lock().unwrap();
        let message = state.messages.pop_front();
        if message.is_some() {
            state.dequeued += 1;
        }
        message
    }
}
pub struct SubscriptionReceiver {
    pub(super) queue: Arc<SubscriptionQueue>,
}
impl SubscriptionReceiver {
    pub async fn recv(&mut self) -> Option<Result<BusMsg, RpcStatus>> {
        match self.queue.wait_ready().await {
            Ok(true) => self.queue.pop().map(Ok),
            Ok(false) => None,
            Err(err) => {
                self.queue.state.lock().unwrap().error.take();
                Some(Err(err))
            }
        }
    }
}
impl Drop for SubscriptionReceiver {
    fn drop(&mut self) {
        let mut state = self.queue.state.lock().unwrap();
        state.closed = true;
        state.messages.clear();
        drop(state);
        self.queue.ready.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn message(n: u8) -> BusMsg {
        BusMsg {
            topic: "sensor".into(),
            payload: Arc::from([n]),
        }
    }

    #[test]
    fn overflow_retains_expected_messages_and_counts_drops() {
        for (policy, expected) in [
            (SubscriptionOverflowPolicy::DropNewest, vec![0, 1, 2]),
            (SubscriptionOverflowPolicy::DropOldest, vec![7, 8, 9]),
            (SubscriptionOverflowPolicy::Latest, vec![9]),
        ] {
            let total = Arc::new(AtomicU64::new(0));
            let queue = SubscriptionQueue::new(1, "sensor".into(), 3, policy, total.clone());
            for n in 0..10 {
                queue.push(message(n));
            }
            let stats = queue.stats();
            assert_eq!(stats.pending, expected.len());
            assert_eq!(stats.received, 10);
            assert_eq!(stats.dropped, 10 - expected.len() as u64);
            assert_eq!(total.load(Ordering::Relaxed), stats.dropped);
            let actual: Vec<_> = std::iter::from_fn(|| queue.pop())
                .map(|m| m.payload[0])
                .collect();
            assert_eq!(actual, expected);
            assert_eq!(queue.stats().dequeued, expected.len() as u64);
            assert_eq!(queue.stats().pending, 0);
        }
    }

    #[test]
    fn keep_last_retains_newest_n_in_order_including_depth_one() {
        for (depth, expected) in [(3, vec![7, 8, 9]), (1, vec![9])] {
            let demux = super::super::sub_demux::SubDemux::new("tcp://127.0.0.1:1");
            let receiver = demux.open_subscribe("sensor".into(), depth).unwrap();
            let queue = &receiver.queue;
            for n in 0..10 {
                queue.push(message(n));
            }
            assert_eq!(queue.stats().policy, SubscriptionOverflowPolicy::DropOldest);
            assert_eq!(queue.stats().dropped, 10 - depth as u64);
            let actual: Vec<_> = std::iter::from_fn(|| queue.pop())
                .map(|m| m.payload[0])
                .collect();
            assert_eq!(actual, expected);
        }
    }

    #[tokio::test]
    async fn readiness_does_not_freeze_a_stale_payload_while_writer_is_busy() {
        let queue = SubscriptionQueue::new(
            1,
            "sensor".into(),
            1,
            SubscriptionOverflowPolicy::default(),
            Arc::default(),
        );
        queue.push(message(1));
        assert!(queue.wait_ready().await.unwrap());
        // A readiness token is waiting in the socket writer; newer samples replace it.
        queue.push(message(2));
        queue.push(message(3));
        assert_eq!(queue.pop().unwrap().payload[0], 3);
        assert_eq!(queue.stats().dropped, 2);
    }

    #[tokio::test]
    async fn dropping_receiver_clears_pending_and_wakes_writer() {
        let total = Arc::new(AtomicU64::new(0));
        let queue = SubscriptionQueue::new(
            1,
            "sensor".into(),
            1,
            SubscriptionOverflowPolicy::default(),
            total.clone(),
        );
        let receiver = SubscriptionReceiver {
            queue: queue.clone(),
        };
        queue.push(message(1));
        queue.push(message(2));
        drop(receiver);
        assert!(!queue.wait_ready().await.unwrap());
        assert!(queue.pop().is_none());
        queue.push(message(3));
        assert_eq!(queue.stats().received, 2);
        assert_eq!(total.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn receiver_wakes_on_new_data_and_on_failure_even_when_full() {
        let queue = SubscriptionQueue::new(
            1,
            "sensor".into(),
            1,
            SubscriptionOverflowPolicy::DropNewest,
            Arc::default(),
        );
        let mut receiver = SubscriptionReceiver {
            queue: queue.clone(),
        };
        let writer = queue.clone();
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            writer.push(message(7));
        });
        let message = tokio::time::timeout(std::time::Duration::from_secs(1), receiver.recv())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(message.payload[0], 7);
        queue.push(super::tests::message(8));
        queue.fail(RpcStatus::unavailable("disconnected"));
        assert!(receiver.recv().await.unwrap().is_err());
        assert!(receiver.recv().await.is_none());
        assert!(queue.pop().is_none());
    }
}
