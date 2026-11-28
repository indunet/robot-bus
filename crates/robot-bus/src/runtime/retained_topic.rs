//! Publisher-local snapshots carried on private topic filters through the normal proxy.

use std::collections::HashSet;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use uuid::Uuid;

use crate::errors::{BusError, Result};
use crate::zmq::{self, Socket, SocketType};
use crate::zmq_helpers::{HighWaterMark, apply_publisher_options_with};

use super::wake::{WakingSender, wake_pair};

const PREFIX: &str = "\0rbus-state-v1/";
const STAMP: usize = 24;

pub(crate) fn is_internal_topic(topic: &str) -> bool {
    topic.starts_with(PREFIX)
}

#[cfg(feature = "broker")]
pub(crate) fn internal_source_topic(topic: &str) -> Option<&str> {
    let (_, rest) = topic.strip_prefix(PREFIX)?.split_once('/')?;
    rest.split_once('\0').map(|(source, _)| source)
}

fn validate_topic(topic: &str) -> Result<()> {
    if topic.is_empty() || topic.contains('\0') {
        return Err(BusError::Protocol(
            "transient_local needs a nonempty topic name without NUL".into(),
        ));
    }
    Ok(())
}

pub(crate) struct RetainedFilter {
    pub(crate) live: String,
    pub(crate) replay: String,
    revision: Option<([u8; 16], u64)>,
}

impl RetainedFilter {
    pub(crate) fn new(topic: &str) -> Result<Self> {
        validate_topic(topic)?;
        Ok(Self {
            live: format!("{PREFIX}live/{topic}\0"),
            replay: format!("{PREFIX}replay/{topic}\0{}", Uuid::new_v4().simple()),
            revision: None,
        })
    }

    pub(crate) fn accept(&mut self, topic: &str, payload: &[u8]) -> Option<usize> {
        let is_replay = topic == self.replay;
        if !is_replay && topic != self.live {
            return None;
        }
        let publisher = payload.get(..16)?.try_into().ok()?;
        let sequence = u64::from_le_bytes(payload.get(16..STAMP)?.try_into().ok()?);
        if let Some((last_publisher, last_seq)) = self.revision {
            // A replay of the same revision can re-establish state after reconnect.
            if publisher == last_publisher
                && (sequence < last_seq || (sequence == last_seq && !is_replay))
            {
                return None;
            }
        }
        self.revision = Some((publisher, sequence));
        Some(STAMP)
    }
}

enum Command {
    Publish(Vec<u8>, mpsc::SyncSender<Result<()>>),
    GetHwm(mpsc::SyncSender<Result<HighWaterMark>>),
    SetHwm(HighWaterMark, mpsc::SyncSender<Result<()>>),
    SetSendTimeout(i32, mpsc::SyncSender<Result<()>>),
    Stop,
}

pub(crate) struct RetainedPublisher {
    commands: WakingSender<Command>,
    thread: Option<JoinHandle<()>>,
}

impl RetainedPublisher {
    pub(crate) fn new(
        context: &zmq::Context,
        endpoint: &str,
        topic: &str,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        validate_topic(topic)?;
        let socket = context.socket(SocketType::XPUB)?;
        apply_publisher_options_with(&socket, hwm)?;
        socket.connect(endpoint)?;
        let (wake, wake_rx) = wake_pair(context);
        let wake_rx = wake_rx
            .ok_or_else(|| BusError::Protocol("state publisher wake socket unavailable".into()))?;
        let (tx, rx) = mpsc::channel();
        let mut state = PublisherState {
            topic: topic.to_owned(),
            live: format!("{PREFIX}live/{topic}\0"),
            replay_prefix: format!("{PREFIX}replay/{topic}\0"),
            id: Uuid::new_v4(),
            sequence: 0,
            current: Vec::new(),
            live_requested: false,
            waiting_for_live: HashSet::new(),
        };
        let thread = thread::Builder::new()
            .name("rbus-state-publisher".into())
            .spawn(move || {
                loop {
                    let mut items = [socket.as_poll_item(zmq::POLLIN), wake_rx.poll_item()];
                    match zmq::poll(&mut items, -1) {
                        Ok(_) => {}
                        Err(zmq::Error::EINTR) => continue,
                        Err(err) => {
                            log::warn!("state publisher stopped: {err}");
                            return;
                        }
                    }
                    if items[0].is_readable() {
                        for _ in 0..64 {
                            let Ok(event) = socket.recv_bytes(zmq::DONTWAIT) else {
                                break;
                            };
                            state.on_subscription(&socket, &event);
                        }
                    }
                    if items[1].is_readable() {
                        wake_rx.clear();
                        while let Ok(command) = rx.try_recv() {
                            match command {
                                Command::Publish(mut data, done) => {
                                    let result = state.publish(&socket, &mut data);
                                    let _ = done.send(result);
                                }
                                Command::GetHwm(done) => {
                                    let _ = done.send(
                                        HighWaterMark::from_socket(&socket).map_err(Into::into),
                                    );
                                }
                                Command::SetHwm(hwm, done) => {
                                    let _ = done.send(hwm.apply(&socket).map_err(Into::into));
                                }
                                Command::SetSendTimeout(ms, done) => {
                                    let _ = done.send(socket.set_sndtimeo(ms).map_err(Into::into));
                                }
                                Command::Stop => return,
                            }
                        }
                    }
                }
            })
            .map_err(|err| BusError::Protocol(format!("start state publisher: {err}")))?;
        Ok(Self {
            commands: WakingSender::new(tx, wake),
            thread: Some(thread),
        })
    }

    fn request<T>(&self, make: impl FnOnce(mpsc::SyncSender<Result<T>>) -> Command) -> Result<T> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.commands.send(make(tx)).map_err(|_| BusError::Closed)?;
        rx.recv().map_err(|_| BusError::Closed)?
    }

    pub(crate) fn publish(&self, payload: &[u8]) -> Result<()> {
        let mut data = Vec::with_capacity(STAMP + payload.len());
        data.resize(STAMP, 0);
        data.extend_from_slice(payload);
        self.request(|done| Command::Publish(data, done))
    }

    pub(crate) fn high_water_mark(&self) -> Result<HighWaterMark> {
        self.request(Command::GetHwm)
    }

    pub(crate) fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        self.request(|done| Command::SetHwm(hwm, done))
    }

    pub(crate) fn set_send_timeout_ms(&self, ms: i32) -> Result<()> {
        self.request(|done| Command::SetSendTimeout(ms, done))
    }
}

impl Drop for RetainedPublisher {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct PublisherState {
    topic: String,
    live: String,
    replay_prefix: String,
    id: Uuid,
    sequence: u64,
    current: Vec<u8>,
    live_requested: bool,
    waiting_for_live: HashSet<Vec<u8>>,
}

impl PublisherState {
    fn publish(&mut self, socket: &Socket, data: &mut Vec<u8>) -> Result<()> {
        self.sequence += 1;
        data[..16].copy_from_slice(self.id.as_bytes());
        data[16..STAMP].copy_from_slice(&self.sequence.to_le_bytes());
        // The same socket serializes snapshots with live delivery.
        socket.send_multipart([self.topic.as_bytes(), &data[STAMP..]], 0)?;
        socket.send_multipart([self.live.as_bytes(), data.as_slice()], 0)?;
        self.current = std::mem::take(data);
        Ok(())
    }

    fn on_subscription(&mut self, socket: &Socket, event: &[u8]) {
        let Some((&kind, filter)) = event.split_first() else {
            return;
        };
        if filter == self.live.as_bytes() {
            self.live_requested = kind == 1;
            if self.live_requested {
                for reply in std::mem::take(&mut self.waiting_for_live) {
                    self.send_snapshot(socket, &reply);
                }
            }
            return;
        }
        let Some(id) = filter.strip_prefix(self.replay_prefix.as_bytes()) else {
            return;
        };
        if id.len() != 32 || !id.iter().all(u8::is_ascii_hexdigit) {
            return;
        }
        match kind {
            0 => {
                self.waiting_for_live.remove(filter);
            }
            1 if self.live_requested => self.send_snapshot(socket, filter),
            // Federated brokers can deliver the two filters in either order.
            1 => {
                self.waiting_for_live.insert(filter.to_vec());
            }
            _ => {}
        }
    }

    fn send_snapshot(&self, socket: &Socket, filter: &[u8]) {
        if self.current.is_empty() {
            return;
        }
        if let Err(err) = socket.send_multipart([filter, self.current.as_slice()], 0) {
            log::debug!("state replay on {} failed: {err}", self.topic);
        }
    }
}
