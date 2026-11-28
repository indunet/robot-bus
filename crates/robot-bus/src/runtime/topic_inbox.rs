//! KeepLast queues between native topic sockets and application callbacks.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak, mpsc};
use std::thread::{self, JoinHandle};

use crate::errors::{BusError, Result};
use crate::runtime::callback_group::SubscriptionCallback;
use crate::runtime::topic_callbacks::for_each_matching_callback;
use crate::runtime::wake::{Waker, WakingSender, wake_pair};
use crate::runtime::worker_pool::{SubmitError, WeakWorkerPool, WorkerPool, run_callback};
use crate::zmq::{self, Socket};

use super::retained_topic::{RetainedFilter, is_internal_topic};

pub(crate) struct ReadyTopics {
    queue: Mutex<VecDeque<Arc<NativeSubscription>>>,
    direct: Option<DirectDispatch>,
}

struct DirectDispatch {
    pool: WeakWorkerPool,
    running: Arc<AtomicBool>,
    // Closing the gate waits for any in-progress submission, never for user callbacks.
    enabled: Mutex<bool>,
}

pub(crate) struct DirectDispatchGuard<'a>(Option<&'a Mutex<bool>>);

impl Drop for DirectDispatchGuard<'_> {
    fn drop(&mut self) {
        if let Some(enabled) = self.0 {
            *enabled.lock().unwrap() = false;
        }
    }
}

impl ReadyTopics {
    pub(crate) fn new(pool: Option<&WorkerPool>, running: &Arc<AtomicBool>) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            direct: pool.map(|pool| DirectDispatch {
                pool: pool.downgrade(),
                running: Arc::clone(running),
                enabled: Mutex::new(false),
            }),
        }
    }

    /// Only a continuously spinning executor admits topic jobs from reader threads.
    pub(crate) fn enable_direct_dispatch(&self) -> DirectDispatchGuard<'_> {
        let enabled = self.direct.as_ref().map(|direct| &direct.enabled);
        if let Some(enabled) = enabled {
            *enabled.lock().unwrap() = true;
        }
        DirectDispatchGuard(enabled)
    }

    /// Schedule a ticket and return whether the executor needs a wake.
    fn schedule(&self, sub: &Arc<NativeSubscription>) -> bool {
        let mut queue = self.queue.lock().unwrap();
        let was_empty = queue.is_empty();
        // Give already waiting tickets first use of newly available pool slots.
        if was_empty
            && self
                .direct
                .as_ref()
                .is_some_and(|direct| direct.submit(sub))
        {
            return false;
        }
        // Manual spin, or a worker/group queue with no room. The executor will
        // retry the ticket while its payload stays in the KeepLast queue.
        queue.push_back(Arc::clone(sub));
        was_empty
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    pub(crate) fn dispatch(&self, pool: Option<&WorkerPool>) -> bool {
        // Only run one ready item per subscription before polling other entities.
        let count = self.queue.lock().unwrap().len();
        let mut did_work = false;
        for _ in 0..count {
            let mut queue = self.queue.lock().unwrap();
            let Some(sub) = queue.pop_front() else {
                break;
            };
            if let Some(pool) = pool {
                if sub.submit(pool).is_err() {
                    // Keep the bounded inbox with its newest contents until the pool has room.
                    queue.push_back(sub);
                    continue;
                }
            } else {
                drop(queue);
                sub.run(false);
            }
            did_work = true;
        }
        did_work
    }
}

impl DirectDispatch {
    fn submit(&self, sub: &Arc<NativeSubscription>) -> bool {
        let enabled = self.enabled.lock().unwrap();
        *enabled
            && self.running.load(Ordering::Acquire)
            && self
                .pool
                .upgrade()
                .is_some_and(|pool| sub.submit(&pool).is_ok())
    }
}

struct Pending {
    samples: VecDeque<(Arc<Vec<u8>>, usize)>,
    outstanding: usize,
    running: usize,
    active: bool,
}

pub(crate) struct NativeSubscription {
    callback: SubscriptionCallback,
    capacity: usize,
    parallelism: usize,
    pending: Mutex<Pending>,
    ready: Weak<ReadyTopics>,
    waker: Waker,
}

impl NativeSubscription {
    pub(crate) fn new(
        callback: SubscriptionCallback,
        capacity: usize,
        parallelism: usize,
        ready: &Arc<ReadyTopics>,
        waker: Waker,
    ) -> Self {
        Self {
            callback,
            capacity,
            parallelism,
            pending: Mutex::new(Pending {
                samples: VecDeque::new(),
                outstanding: 0,
                running: 0,
                active: true,
            }),
            ready: Arc::downgrade(ready),
            waker,
        }
    }

    pub(crate) fn deactivate(&self) {
        let mut pending = self.pending.lock().unwrap();
        pending.active = false;
        pending.samples.clear();
        if let Some(ready) = self.ready.upgrade() {
            ready
                .queue
                .lock()
                .unwrap()
                .retain(|sub| !std::ptr::eq(sub.as_ref(), self));
        }
    }

    fn push(self: &Arc<Self>, payload: Arc<Vec<u8>>, offset: usize) {
        let mut pending = self.pending.lock().unwrap();
        if !pending.active {
            return;
        }
        if pending.samples.len() == self.capacity {
            pending.samples.pop_front();
        }
        pending.samples.push_back((payload, offset));
        let wake = self.schedule(&mut pending);
        drop(pending);
        if wake {
            self.waker.wake();
        }
    }

    fn schedule(self: &Arc<Self>, pending: &mut Pending) -> bool {
        // Each ticket accounts for one callback until it finishes. A queued ticket
        // takes its payload when it actually starts, after any oldest-item drops.
        if pending.samples.len() > pending.outstanding - pending.running
            && pending.outstanding < self.parallelism
        {
            if let Some(ready) = self.ready.upgrade() {
                pending.outstanding += 1;
                return ready.schedule(self);
            }
        }
        false
    }

    fn submit(self: &Arc<Self>, pool: &WorkerPool) -> std::result::Result<(), SubmitError> {
        let job = Arc::clone(self);
        self.callback
            .group
            .try_run(Some(pool), move || job.run(true))
    }

    fn run(self: &Arc<Self>, wake: bool) {
        let (payload, more) = {
            let mut pending = self.pending.lock().unwrap();
            pending.running += 1;
            let item = pending.samples.pop_front();
            (item, self.schedule(&mut pending))
        };
        if more && wake {
            self.waker.wake();
        }
        if let Some((payload, offset)) = payload {
            let _ = run_callback(|| (self.callback.callback)(&payload[offset..]));
        }
        let mut pending = self.pending.lock().unwrap();
        pending.running -= 1;
        pending.outstanding -= 1;
        let more = self.schedule(&mut pending);
        drop(pending);
        if more && wake {
            self.waker.wake();
        }
    }
}

struct ReaderSubscription {
    queue: Arc<NativeSubscription>,
    retained: Option<RetainedFilter>,
}

#[derive(Default)]
struct Filters {
    callbacks: HashMap<String, Vec<SubscriptionCallback>>,
    subscriptions: HashMap<u64, ReaderSubscription>,
}

enum Command {
    Add(
        String,
        Arc<NativeSubscription>,
        Option<RetainedFilter>,
        mpsc::SyncSender<Result<()>>,
    ),
    Remove(u64, mpsc::SyncSender<Result<()>>),
    SetHwm(i32, mpsc::SyncSender<Result<()>>),
    Stop,
}

pub(crate) struct TopicReader {
    pub(crate) endpoint: String,
    commands: WakingSender<Command>,
    thread: Option<JoinHandle<()>>,
}

impl TopicReader {
    pub(crate) fn start(context: &zmq::Context, socket: Socket, endpoint: String) -> Result<Self> {
        let (wake, wake_rx) = wake_pair(context);
        let wake_rx = wake_rx
            .ok_or_else(|| BusError::Protocol("topic reader wake socket unavailable".into()))?;
        let (tx, rx) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("rbus-topic-reader".into())
            .spawn(move || {
                let mut filters = Filters::default();
                loop {
                    let mut items = [socket.as_poll_item(zmq::POLLIN), wake_rx.poll_item()];
                    match zmq::poll(&mut items, -1) {
                        Ok(_) => {}
                        Err(zmq::Error::EINTR) => continue,
                        Err(err) => {
                            log::warn!("topic reader stopped: {err}");
                            break;
                        }
                    }
                    if items[1].is_readable() {
                        wake_rx.clear();
                        while let Ok(command) = rx.try_recv() {
                            match command {
                                Command::Add(topic, sub, retained, done) => {
                                    let _ = done.send(filters.add(&socket, topic, sub, retained));
                                }
                                Command::Remove(id, done) => {
                                    let _ = done.send(filters.remove(&socket, id));
                                }
                                Command::SetHwm(hwm, done) => {
                                    let _ = done.send(socket.set_rcvhwm(hwm).map_err(Into::into));
                                }
                                Command::Stop => return,
                            }
                        }
                    }
                    if items[0].is_readable() {
                        // Bound the batch so subscribe/unsubscribe and shutdown cannot starve.
                        for _ in 0..64 {
                            let Ok(frames) = socket.recv_multipart(zmq::DONTWAIT) else {
                                break;
                            };
                            filters.deliver(frames);
                        }
                    }
                }
            })
            .map_err(|err| BusError::Protocol(format!("start topic reader: {err}")))?;
        Ok(Self {
            endpoint,
            commands: WakingSender::new(tx, wake),
            thread: Some(thread),
        })
    }

    fn request(&self, make: impl FnOnce(mpsc::SyncSender<Result<()>>) -> Command) -> Result<()> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.commands.send(make(tx)).map_err(|_| BusError::Closed)?;
        rx.recv().map_err(|_| BusError::Closed)?
    }

    pub(crate) fn add(
        &self,
        topic: String,
        sub: Arc<NativeSubscription>,
        retained: Option<RetainedFilter>,
    ) -> Result<()> {
        self.request(|done| Command::Add(topic, sub, retained, done))
    }

    pub(crate) fn remove(&self, id: u64) -> Result<()> {
        self.request(|done| Command::Remove(id, done))
    }

    pub(crate) fn set_hwm(&self, hwm: i32) -> Result<()> {
        self.request(|done| Command::SetHwm(hwm, done))
    }
}

impl Drop for TopicReader {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Filters {
    fn add(
        &mut self,
        socket: &Socket,
        topic: String,
        queue: Arc<NativeSubscription>,
        retained: Option<RetainedFilter>,
    ) -> Result<()> {
        let id = queue.callback.id;
        if let Some(filter) = &retained {
            socket.set_subscribe(filter.live.as_bytes())?;
            if let Err(err) = socket.set_subscribe(filter.replay.as_bytes()) {
                let _ = socket.set_unsubscribe(filter.live.as_bytes());
                return Err(err.into());
            }
        } else {
            if !self.callbacks.contains_key(&topic) {
                socket.set_subscribe(topic.as_bytes())?;
            }
            self.callbacks
                .entry(topic)
                .or_default()
                .push(queue.callback.clone());
        }
        self.subscriptions
            .insert(id, ReaderSubscription { queue, retained });
        Ok(())
    }

    fn remove(&mut self, socket: &Socket, id: u64) -> Result<()> {
        let Some(sub) = self.subscriptions.remove(&id) else {
            return Ok(());
        };
        if let Some(filter) = sub.retained {
            let live = socket.set_unsubscribe(filter.live.as_bytes());
            let replay = socket.set_unsubscribe(filter.replay.as_bytes());
            live.and(replay)?;
        } else {
            self.callbacks.retain(|topic, callbacks| {
                callbacks.retain(|entry| entry.id != id);
                if callbacks.is_empty() {
                    let _ = socket.set_unsubscribe(topic.as_bytes());
                }
                !callbacks.is_empty()
            });
        }
        Ok(())
    }

    fn deliver(&mut self, mut frames: Vec<Vec<u8>>) {
        if frames.len() < 2 {
            return;
        }
        let payload = Arc::new(std::mem::take(&mut frames[1]));
        let topic = String::from_utf8_lossy(&frames[0]);
        if is_internal_topic(&topic) {
            for sub in self.subscriptions.values_mut() {
                if let Some(filter) = &mut sub.retained {
                    if let Some(offset) = filter.accept(&topic, &payload) {
                        sub.queue.push(Arc::clone(&payload), offset);
                    }
                }
            }
            return;
        }
        for_each_matching_callback(&topic, &self.callbacks, |entry| {
            if let Some(sub) = self.subscriptions.get(&entry.id) {
                sub.queue.push(Arc::clone(&payload), 0);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::runtime::CallbackGroup;

    const WAIT: Duration = Duration::from_secs(2);

    fn subscription(
        ready: &Arc<ReadyTopics>,
        depth: usize,
    ) -> (Arc<NativeSubscription>, mpsc::Receiver<u8>) {
        let (tx, rx) = mpsc::channel();
        let sub = Arc::new(NativeSubscription::new(
            SubscriptionCallback {
                id: 0,
                callback: Arc::new(move |payload| tx.send(payload[0]).unwrap()),
                group: CallbackGroup::mutually_exclusive(),
            },
            depth,
            1,
            ready,
            Waker::disabled(),
        ));
        (sub, rx)
    }

    #[test]
    fn only_continuous_spin_directly_admits_new_topic_jobs() {
        let pool = WorkerPool::new(1);
        let running = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(ReadyTopics::new(Some(&pool), &running));
        let (manual, manual_rx) = subscription(&ready, 1);
        manual.push(Arc::new(vec![1]), 0);
        assert!(!ready.is_empty());
        assert!(manual_rx.try_recv().is_err());
        assert!(ready.dispatch(Some(&pool)));
        assert_eq!(manual_rx.recv_timeout(WAIT).unwrap(), 1);

        let (continuous, continuous_rx) = subscription(&ready, 1);
        let (shutdown, shutdown_rx) = subscription(&ready, 1);
        running.store(true, Ordering::Release);
        {
            let _guard = ready.enable_direct_dispatch();
            continuous.push(Arc::new(vec![2]), 0);
            assert!(ready.is_empty());
            assert_eq!(continuous_rx.recv_timeout(WAIT).unwrap(), 2);

            running.store(false, Ordering::Release);
            shutdown.push(Arc::new(vec![3]), 0);
            assert!(!ready.is_empty());
            assert!(shutdown_rx.try_recv().is_err());
        }

        // spin_some also sets the running flag, but the direct gate stays closed.
        running.store(true, Ordering::Release);
        let (stepped, stepped_rx) = subscription(&ready, 1);
        stepped.push(Arc::new(vec![4]), 0);
        assert_eq!(ready.queue.lock().unwrap().len(), 2);
        assert!(stepped_rx.try_recv().is_err());
        assert!(ready.dispatch(Some(&pool)));
        assert_eq!(shutdown_rx.recv_timeout(WAIT).unwrap(), 3);
        assert_eq!(stepped_rx.recv_timeout(WAIT).unwrap(), 4);
    }

    #[test]
    fn full_worker_queue_retries_the_ticket_with_the_latest_samples() {
        let pool = WorkerPool::with_queue_capacity(1, 1);
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        pool.try_submit(move || {
            entered_tx.send(()).unwrap();
            release_rx.recv_timeout(WAIT).unwrap();
        })
        .unwrap();
        entered_rx.recv_timeout(WAIT).unwrap();
        let (drained_tx, drained_rx) = mpsc::channel();
        pool.try_submit(move || drained_tx.send(()).unwrap())
            .unwrap();

        let running = Arc::new(AtomicBool::new(true));
        let ready = Arc::new(ReadyTopics::new(Some(&pool), &running));
        let _guard = ready.enable_direct_dispatch();
        let (sub, rx) = subscription(&ready, 2);
        for sample in 1..=10 {
            sub.push(Arc::new(vec![sample]), 0);
        }
        assert_eq!(ready.queue.lock().unwrap().len(), 1);
        assert!(!ready.dispatch(Some(&pool)));
        assert!(rx.try_recv().is_err());

        release_tx.send(()).unwrap();
        drained_rx.recv_timeout(WAIT).unwrap();
        let (new_sub, new_rx) = subscription(&ready, 1);
        new_sub.push(Arc::new(vec![11]), 0);
        assert!(new_rx.try_recv().is_err());
        assert_eq!(ready.queue.lock().unwrap().len(), 2);
        assert!(ready.dispatch(Some(&pool)));
        assert_eq!(rx.recv_timeout(WAIT).unwrap(), 9);
        // Keep polling while the two subscriptions share the single worker.
        let deadline = std::time::Instant::now() + WAIT;
        let mut latest = None;
        let mut newer = None;
        while latest.is_none() || newer.is_none() {
            ready.dispatch(Some(&pool));
            latest = latest.or_else(|| rx.try_recv().ok());
            newer = newer.or_else(|| new_rx.try_recv().ok());
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(latest, Some(10));
        assert_eq!(newer, Some(11));
        assert!(ready.is_empty());
    }
}
