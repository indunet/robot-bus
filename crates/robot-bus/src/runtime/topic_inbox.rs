//! KeepLast queues between native topic sockets and application callbacks.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, Weak, mpsc};
use std::thread::{self, JoinHandle};

use crate::errors::{BusError, Result};
use crate::runtime::callback_group::SubscriptionCallback;
use crate::runtime::topic_callbacks::for_each_matching_callback;
use crate::runtime::wake::{Waker, WakingSender, wake_pair};
use crate::runtime::worker_pool::{WorkerPool, run_callback};
use crate::zmq::{self, Socket};

use super::retained_topic::{RetainedFilter, is_internal_topic};

#[derive(Default)]
pub(crate) struct ReadyTopics {
    queue: Mutex<VecDeque<Arc<NativeSubscription>>>,
}

impl ReadyTopics {
    pub(crate) fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    pub(crate) fn dispatch(&self, pool: Option<&WorkerPool>) -> bool {
        // Only run one ready item per subscription before polling other entities.
        let count = self.queue.lock().unwrap().len();
        let mut did_work = false;
        for _ in 0..count {
            let Some(sub) = self.queue.lock().unwrap().pop_front() else {
                break;
            };
            if let Some(pool) = pool {
                let job = Arc::clone(&sub);
                if sub
                    .callback
                    .group
                    .try_run(Some(pool), move || job.run(true))
                    .is_err()
                {
                    // Keep the bounded inbox with its newest contents until the pool has room.
                    self.queue.lock().unwrap().push_back(sub);
                    continue;
                }
            } else {
                sub.run(false);
            }
            did_work = true;
        }
        did_work
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
        self.notify(&mut pending, true);
    }

    fn notify(self: &Arc<Self>, pending: &mut Pending, wake: bool) {
        // Each ticket accounts for one callback until it finishes. A queued ticket
        // takes its payload when it actually starts, after any oldest-item drops.
        if pending.samples.len() > pending.outstanding - pending.running
            && pending.outstanding < self.parallelism
        {
            if let Some(ready) = self.ready.upgrade() {
                pending.outstanding += 1;
                let was_empty = {
                    let mut queue = ready.queue.lock().unwrap();
                    let was_empty = queue.is_empty();
                    queue.push_back(Arc::clone(self));
                    was_empty
                };
                if was_empty && wake {
                    self.waker.wake();
                }
            }
        }
    }

    fn run(self: &Arc<Self>, wake: bool) {
        let payload = {
            let mut pending = self.pending.lock().unwrap();
            pending.running += 1;
            let item = pending.samples.pop_front();
            self.notify(&mut pending, wake);
            item
        };
        if let Some((payload, offset)) = payload {
            let _ = run_callback(|| (self.callback.callback)(&payload[offset..]));
        }
        let mut pending = self.pending.lock().unwrap();
        pending.running -= 1;
        pending.outstanding -= 1;
        self.notify(&mut pending, wake);
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
