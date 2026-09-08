//! Shared ZMQ SUB demux for the message gateway (one reader thread, many watchers).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::rpc_status::RpcStatus;
use super::subscription_queue::{SubscriptionQueue, SubscriptionReceiver, SubscriptionStats};
use serde::Serialize;

use crate::message_bus::Subscriber;
use crate::runtime::SubscriptionOverflowPolicy;

const POLL_TIMEOUT: Duration = Duration::from_millis(200);

/// One bus message fanned out to WS subscribe streams.
#[derive(Clone)]
pub struct BusMsg {
    pub topic: String,
    pub payload: Arc<[u8]>,
}

type WatcherTx = Arc<SubscriptionQueue>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionSnapshot {
    pub total_dropped: u64,
    pub subscriptions: Vec<SubscriptionStats>,
}

struct Watcher {
    id: u64,
    tx: WatcherTx,
}

struct DemuxState {
    /// Filter string → active watchers (same filter string as ZMQ SUB subscribe).
    filters: HashMap<String, Vec<Watcher>>,
}

enum Control {
    Add {
        filter: String,
        id: u64,
        tx: WatcherTx,
    },
    /// Reader exits when received (demux dropped).
    Shutdown,
}

/// Shared SUB demux handle (cloneable).
#[derive(Clone)]
pub struct SubDemux {
    inner: Arc<SubDemuxInner>,
}

struct SubDemuxInner {
    next_id: AtomicU64,
    total_dropped: Arc<AtomicU64>,
    control_tx: Mutex<Option<std::sync::mpsc::Sender<Control>>>,
    state: Arc<Mutex<DemuxState>>,
    xpub: String,
}

impl SubDemux {
    pub fn new(message_xpub: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(SubDemuxInner {
                next_id: AtomicU64::new(1),
                total_dropped: Arc::new(AtomicU64::new(0)),
                control_tx: Mutex::new(None),
                state: Arc::new(Mutex::new(DemuxState {
                    filters: HashMap::new(),
                })),
                xpub: message_xpub.into(),
            }),
        }
    }

    fn ensure_started(&self) -> Result<(), RpcStatus> {
        let mut guard = self
            .inner
            .control_tx
            .lock()
            .map_err(|_| RpcStatus::internal("sub demux control mutex poisoned"))?;
        if guard.is_some() {
            return Ok(());
        }
        let (tx, rx) = std::sync::mpsc::channel::<Control>();
        let state = Arc::clone(&self.inner.state);
        let xpub = self.inner.xpub.clone();
        thread::Builder::new()
            .name("ws-zmq-sub-demux".into())
            .spawn(move || demux_loop(xpub, state, rx))
            .map_err(|err| RpcStatus::internal(format!("spawn sub demux: {err}")))?;
        *guard = Some(tx);
        Ok(())
    }

    pub fn open_subscribe(
        &self,
        topic: String,
        qos_depth: i32,
    ) -> Result<SubscriptionReceiver, RpcStatus> {
        self.open_subscribe_with_policy(topic, qos_depth, SubscriptionOverflowPolicy::DropNewest)
    }

    pub fn snapshot(&self) -> SubscriptionSnapshot {
        let state = self.inner.state.lock().unwrap();
        let mut subscriptions: Vec<_> = state
            .filters
            .values()
            .flatten()
            .filter(|w| !w.tx.is_closed())
            .map(|w| w.tx.stats())
            .collect();
        subscriptions.sort_by_key(|s| s.id);
        SubscriptionSnapshot {
            total_dropped: self.inner.total_dropped.load(Ordering::Relaxed),
            subscriptions,
        }
    }

    pub fn open_subscribe_with_policy(
        &self,
        topic: String,
        qos_depth: i32,
        policy: SubscriptionOverflowPolicy,
    ) -> Result<SubscriptionReceiver, RpcStatus> {
        self.ensure_started()?;
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let tx = SubscriptionQueue::new(
            id,
            topic.clone(),
            qos_depth,
            policy,
            Arc::clone(&self.inner.total_dropped),
        );
        let rx = SubscriptionReceiver {
            queue: Arc::clone(&tx),
        };
        let control = {
            let guard = self
                .inner
                .control_tx
                .lock()
                .map_err(|_| RpcStatus::internal("sub demux control mutex poisoned"))?;
            guard
                .as_ref()
                .ok_or_else(|| RpcStatus::internal("sub demux not started"))?
                .clone()
        };
        control
            .send(Control::Add {
                filter: topic,
                id,
                tx,
            })
            .map_err(|_| RpcStatus::internal("sub demux control channel closed"))?;
        Ok(rx)
    }
}

impl Drop for SubDemuxInner {
    fn drop(&mut self) {
        if let Ok(guard) = self.control_tx.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(Control::Shutdown);
            }
        }
    }
}

fn filter_matches(filter: &str, topic: &str) -> bool {
    // ZeroMQ SUB prefix match: empty filter matches all; otherwise topic must start with filter.
    filter.is_empty() || topic.starts_with(filter)
}

fn demux_loop(
    xpub: String,
    state: Arc<Mutex<DemuxState>>,
    control_rx: std::sync::mpsc::Receiver<Control>,
) {
    let sub = match Subscriber::new(Some(&xpub)) {
        Ok(sub) => sub,
        Err(err) => {
            log::error!("sub demux failed to connect: {err}");
            while let Ok(cmd) = control_rx.recv() {
                match cmd {
                    Control::Add { tx, .. } => tx.fail(RpcStatus::unavailable(err.to_string())),
                    Control::Shutdown => return,
                }
            }
            return;
        }
    };

    let mut last_sweep = std::time::Instant::now();
    loop {
        if last_sweep.elapsed() >= POLL_TIMEOUT {
            sweep_closed(&sub, &state);
            last_sweep = std::time::Instant::now();
        }
        while let Ok(cmd) = control_rx.try_recv() {
            match cmd {
                Control::Shutdown => return,
                Control::Add { filter, id, tx } => {
                    let mut guard = match state.lock() {
                        Ok(g) => g,
                        Err(_) => return,
                    };
                    let entry = guard.filters.entry(filter.clone()).or_default();
                    let first = entry.is_empty();
                    entry.push(Watcher { id, tx });
                    drop(guard);
                    if first {
                        if let Err(err) = sub.subscribe(&filter) {
                            log::warn!("sub demux subscribe {filter:?} failed: {err}");
                        }
                    }
                }
            }
        }

        match sub.receive(Some(POLL_TIMEOUT)) {
            Ok((msg_topic, payload)) => {
                let payload: Arc<[u8]> = Arc::from(payload);
                let mut dead: Vec<(String, u64)> = Vec::new();
                let Ok(guard) = state.lock() else {
                    return;
                };
                for (filter, watchers) in guard.filters.iter() {
                    if !filter_matches(filter, &msg_topic) {
                        continue;
                    }
                    for w in watchers {
                        let msg = BusMsg {
                            topic: msg_topic.clone(),
                            payload: Arc::clone(&payload),
                        };
                        if w.tx.is_closed() {
                            dead.push((filter.clone(), w.id));
                        } else {
                            w.tx.push(msg);
                        }
                    }
                }
                drop(guard);
                if !dead.is_empty() {
                    remove_watchers(&sub, &state, &dead);
                }
            }
            Err(crate::errors::BusError::Timeout(_)) => {
                sweep_closed(&sub, &state);
            }
            Err(err) => {
                log::error!("sub demux receive error: {err}");
                let Ok(mut guard) = state.lock() else {
                    return;
                };
                for watchers in guard.filters.values() {
                    for w in watchers {
                        w.tx.fail(RpcStatus::internal(err.to_string()));
                    }
                }
                guard.filters.clear();
                return;
            }
        }
    }
}

fn remove_watchers(sub: &Subscriber, state: &Mutex<DemuxState>, dead: &[(String, u64)]) {
    let Ok(mut guard) = state.lock() else {
        return;
    };
    let mut empty_filters = Vec::new();
    for (filter, id) in dead {
        let Some(watchers) = guard.filters.get_mut(filter) else {
            continue;
        };
        watchers.retain(|w| w.id != *id);
        if watchers.is_empty() {
            empty_filters.push(filter.clone());
        }
    }
    for filter in &empty_filters {
        guard.filters.remove(filter);
        if let Err(err) = sub.unsubscribe(filter) {
            log::warn!("sub demux unsubscribe {filter:?} failed: {err}");
        }
    }
}

fn sweep_closed(sub: &Subscriber, state: &Mutex<DemuxState>) {
    let Ok(mut guard) = state.lock() else {
        return;
    };
    let mut empty_filters = Vec::new();
    for (filter, watchers) in guard.filters.iter_mut() {
        watchers.retain(|w| !w.tx.is_closed());
        if watchers.is_empty() {
            empty_filters.push(filter.clone());
        }
    }
    for filter in empty_filters {
        guard.filters.remove(&filter);
        if let Err(err) = sub.unsubscribe(&filter) {
            log::warn!("sub demux unsubscribe {filter:?} failed: {err}");
        }
    }
}
