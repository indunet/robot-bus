//! Unified poll loop for subscriptions and worker callbacks.
//!
//! Mirrors a simplified ROS 2 executor:
//! - register callbacks (`subscribe` / `register_service` / …)
//! - drive them with [`BusRuntime::spin`], [`BusRuntime::spin_once`], or
//!   [`BusRuntime::spin_some`]
//! - stop with [`BusRuntime::shutdown`] (from any thread)
//!
//! Default mode is single-threaded: callbacks run on the spin/I/O thread.
//! [`BusRuntime::with_executor`] offloads long service/action handlers to a
//! bounded worker pool (subscription callbacks stay on the I/O thread).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use uuid::Uuid;
use zmq::{Context, SocketType};

use crate::errors::{BusError, Result};
use crate::runtime::dispatch::{
    dispatch_registration, flush_outbound, flush_reply_queue, tick_heartbeats,
};
use crate::runtime::queues::{ActionMessageCallback, OutboundCommand, ReplyMessage};
use crate::runtime::registrations::{
    ActionClientRegistration, ActionGoalHandler, ActionRegistration, MessageCallback,
    Registration, ServiceHandler, ServiceRegistration, SubRegistration,
};
use crate::runtime::worker_pool::WorkerPool;
use crate::transports::{
    action_backend_endpoint, action_frontend_endpoint, message_xpub_endpoint,
    service_backend_endpoint,
};
use crate::zmq_helpers::{apply_subscriber_options, wait_for_connection};

const DEFAULT_POLL_TIMEOUT_MS: i64 = 250;
const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 2500;

/// Token that can interrupt [`BusRuntime::spin`] / background [`BusRuntime::start`]
/// from another thread (ROS 2–style cancel).
#[derive(Clone)]
pub struct ShutdownHandle {
    running: Arc<AtomicBool>,
}

impl ShutdownHandle {
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Release);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}

pub struct BusRuntime {
    context: Context,
    heartbeat_interval_ms: u64,
    service_registrations: Vec<ServiceRegistration>,
    action_registrations: Vec<ActionRegistration>,
    action_client_registration: Option<ActionClientRegistration>,
    topic_callbacks: HashMap<String, Vec<MessageCallback>>,
    socket_registrations: Vec<Registration>,
    outbound_tx: Sender<OutboundCommand>,
    outbound_rx: Option<Receiver<OutboundCommand>>,
    reply_tx: Sender<ReplyMessage>,
    reply_rx: Option<Receiver<ReplyMessage>>,
    worker_pool: Option<WorkerPool>,
    running: Arc<AtomicBool>,
    started: bool,
    closed: bool,
    thread: Option<JoinHandle<()>>,
}

impl BusRuntime {
    /// Single-threaded executor: all callbacks run on the spin/I/O thread.
    pub fn new() -> Self {
        Self::with_options(DEFAULT_HEARTBEAT_INTERVAL_MS, None)
    }

    /// Offload service/action handlers to at most `max_workers` threads.
    /// Subscription and action-client callbacks still run on the I/O thread.
    /// If the pool is saturated, handlers fall back to inline execution.
    pub fn with_executor(max_workers: usize) -> Self {
        Self::with_options(
            DEFAULT_HEARTBEAT_INTERVAL_MS,
            Some(WorkerPool::new(max_workers)),
        )
    }

    fn with_options(heartbeat_interval_ms: u64, worker_pool: Option<WorkerPool>) -> Self {
        let (outbound_tx, outbound_rx) = mpsc::channel();
        let (reply_tx, reply_rx) = mpsc::channel();
        Self {
            context: Context::new(),
            heartbeat_interval_ms,
            service_registrations: Vec::new(),
            action_registrations: Vec::new(),
            action_client_registration: None,
            topic_callbacks: HashMap::new(),
            socket_registrations: Vec::new(),
            outbound_tx,
            outbound_rx: Some(outbound_rx),
            reply_tx,
            reply_rx: Some(reply_rx),
            worker_pool,
            running: Arc::new(AtomicBool::new(false)),
            started: false,
            closed: false,
            thread: None,
        }
    }

    /// Cloneable handle to stop [`spin`](Self::spin) / [`start`](Self::start).
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle {
            running: self.running.clone(),
        }
    }

    /// Signal the executor to leave `spin` / background `start`.
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Release);
    }

    pub fn connect_subscriber(&mut self, endpoint: Option<&str>) -> Result<()> {
        self.ensure_open()?;
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => message_xpub_endpoint("localhost", "tcp").map_err(BusError::Protocol)?,
        };
        let socket = self.context.socket(SocketType::SUB)?;
        apply_subscriber_options(&socket)?;
        socket.connect(&endpoint)?;
        wait_for_connection();
        let reg = SubRegistration {
            socket,
            endpoint: endpoint.clone(),
        };
        self.socket_registrations.push(Registration::Sub(reg));
        log::info!("subscriber connected to {endpoint}");
        Ok(())
    }

    pub fn subscribe(&mut self, topic: &str, callback: MessageCallback) -> Result<()> {
        self.ensure_open()?;
        if !self.has_sub_registration() {
            return Err(BusError::Protocol(
                "connect_subscriber() before subscribe()".into(),
            ));
        }
        let topic_bytes = topic.as_bytes();
        for reg in self.socket_registrations.iter_mut() {
            if let Registration::Sub(sub) = reg {
                sub.socket.set_subscribe(topic_bytes)?;
            }
        }
        self.topic_callbacks
            .entry(topic.to_string())
            .or_default()
            .push(callback);
        Ok(())
    }

    pub fn connect_action_client(&mut self, endpoint: Option<&str>) -> Result<()> {
        self.ensure_open()?;
        if self.action_client_registration.is_some() || self.has_action_client_registration() {
            return Err(BusError::Protocol(
                "action client already connected".into(),
            ));
        }
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => action_frontend_endpoint("localhost", "tcp").map_err(BusError::Protocol)?,
        };
        let reg = ActionClientRegistration::create(&self.context, &endpoint)?;
        log::info!("action client connected to {endpoint}");
        self.action_client_registration = Some(reg);
        self.sync_action_client_registration();
        Ok(())
    }

    pub fn send_goal(
        &self,
        action_name: &str,
        body: &[u8],
        callback: ActionMessageCallback,
        goal_id: Option<&str>,
    ) -> Result<String> {
        self.ensure_open()?;
        if self.action_client_registration.is_none() && !self.has_action_client_registration() {
            return Err(BusError::Protocol(
                "connect_action_client() before send_goal()".into(),
            ));
        }
        let gid = goal_id
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
        self.outbound_tx
            .send(OutboundCommand::SendGoal {
                action_name: action_name.to_string(),
                goal_id: gid.clone(),
                body: body.to_vec(),
                callback,
            })
            .map_err(|err| BusError::Protocol(err.to_string()))?;
        Ok(gid)
    }

    pub fn cancel_goal(&self, action_name: &str, goal_id: &str, body: &[u8]) -> Result<()> {
        self.ensure_open()?;
        if self.action_client_registration.is_none() && !self.has_action_client_registration() {
            return Err(BusError::Protocol(
                "connect_action_client() before cancel_goal()".into(),
            ));
        }
        self.outbound_tx
            .send(OutboundCommand::CancelGoal {
                action_name: action_name.to_string(),
                goal_id: goal_id.to_string(),
                body: body.to_vec(),
            })
            .map_err(|err| BusError::Protocol(err.to_string()))
    }

    pub fn register_service(
        &mut self,
        service_name: &str,
        handler: ServiceHandler,
        backend_endpoint: Option<&str>,
        identity: Option<&str>,
    ) -> Result<()> {
        self.ensure_open()?;
        let endpoint = match backend_endpoint {
            Some(ep) => ep.to_string(),
            None => service_backend_endpoint("localhost", "tcp").map_err(BusError::Protocol)?,
        };
        let reg = ServiceRegistration::create(
            &self.context,
            service_name,
            handler,
            &endpoint,
            identity,
            self.heartbeat_interval_ms,
        )?;
        log::info!(
            "service worker {:?} registered for {service_name} on {endpoint}",
            String::from_utf8_lossy(&reg.identity)
        );
        self.service_registrations.push(reg);
        self.sync_worker_registrations();
        Ok(())
    }

    pub fn register_action(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        backend_endpoint: Option<&str>,
        identity: Option<&str>,
    ) -> Result<()> {
        self.ensure_open()?;
        let endpoint = match backend_endpoint {
            Some(ep) => ep.to_string(),
            None => action_backend_endpoint("localhost", "tcp").map_err(BusError::Protocol)?,
        };
        let reg = ActionRegistration::create(
            &self.context,
            action_name,
            handler,
            &endpoint,
            identity,
            self.heartbeat_interval_ms,
        )?;
        log::info!(
            "action worker {:?} registered for {action_name} on {endpoint}",
            String::from_utf8_lossy(&reg.identity)
        );
        self.action_registrations.push(reg);
        self.sync_worker_registrations();
        Ok(())
    }

    /// One executor step (ROS 2 `spin_once`): wait up to `timeout`, then
    /// dispatch every currently readable registration.
    ///
    /// Returns `true` if at least one socket was readable.
    pub fn spin_once(&mut self, timeout: Option<Duration>) -> Result<bool> {
        self.ensure_open()?;
        if self.started {
            return Err(BusError::Protocol(
                "spin_once() cannot run while start() is active".into(),
            ));
        }
        self.prepare_for_spin()?;
        let outbound_rx = self
            .outbound_rx
            .as_ref()
            .ok_or_else(|| BusError::Protocol("runtime already started".into()))?;
        let reply_rx = self
            .reply_rx
            .as_ref()
            .ok_or_else(|| BusError::Protocol("runtime already started".into()))?;
        Ok(spin_once_inner(
            &mut self.socket_registrations,
            &self.topic_callbacks,
            outbound_rx,
            reply_rx,
            &self.reply_tx,
            self.worker_pool.as_ref(),
            timeout_to_ms(timeout),
        ))
    }

    /// Wait up to `timeout` for work, then drain ready callbacks (ROS 2 `spin_some`).
    ///
    /// After the first successful poll, further iterations use a zero timeout so
    /// only already-queued messages are processed before returning.
    pub fn spin_some(&mut self, timeout: Option<Duration>) -> Result<()> {
        self.ensure_open()?;
        if self.started {
            return Err(BusError::Protocol(
                "spin_some() cannot run while start() is active".into(),
            ));
        }
        self.prepare_for_spin()?;
        self.running.store(true, Ordering::Release);
        let deadline = timeout.map(|t| Instant::now() + t);
        let mut draining = false;
        loop {
            if !self.running.load(Ordering::Acquire) {
                break;
            }
            let poll_ms = if draining {
                0
            } else {
                match deadline {
                    Some(end) => {
                        let remaining = end.saturating_duration_since(Instant::now());
                        if remaining.is_zero() {
                            break;
                        }
                        remaining.as_millis().min(i64::MAX as u128) as i64
                    }
                    None => DEFAULT_POLL_TIMEOUT_MS,
                }
            };
            let outbound_rx = self
                .outbound_rx
                .as_ref()
                .ok_or_else(|| BusError::Protocol("runtime already started".into()))?;
            let reply_rx = self
                .reply_rx
                .as_ref()
                .ok_or_else(|| BusError::Protocol("runtime already started".into()))?;
            let had_work = spin_once_inner(
                &mut self.socket_registrations,
                &self.topic_callbacks,
                outbound_rx,
                reply_rx,
                &self.reply_tx,
                self.worker_pool.as_ref(),
                poll_ms,
            );
            if had_work {
                draining = true;
                continue;
            }
            break;
        }
        Ok(())
    }

    /// Block on the executor until [`shutdown`](Self::shutdown) (ROS 2 `spin`).
    pub fn spin(&mut self) -> Result<()> {
        self.ensure_open()?;
        if self.started {
            return Err(BusError::Protocol(
                "spin() cannot run while start() is active".into(),
            ));
        }
        self.prepare_for_spin()?;
        self.running.store(true, Ordering::Release);
        let outbound_rx = self
            .outbound_rx
            .as_ref()
            .ok_or_else(|| BusError::Protocol("runtime already started".into()))?;
        let reply_rx = self
            .reply_rx
            .as_ref()
            .ok_or_else(|| BusError::Protocol("runtime already started".into()))?;
        while self.running.load(Ordering::Acquire) {
            spin_once_inner(
                &mut self.socket_registrations,
                &self.topic_callbacks,
                outbound_rx,
                reply_rx,
                &self.reply_tx,
                self.worker_pool.as_ref(),
                DEFAULT_POLL_TIMEOUT_MS,
            );
        }
        Ok(())
    }

    /// Run the executor on a background thread until [`shutdown`](Self::shutdown)
    /// or [`stop`](Self::stop).
    pub fn start(&mut self) -> Result<()> {
        self.ensure_open()?;
        if self.started {
            return Ok(());
        }
        self.prepare_for_spin()?;
        self.running.store(true, Ordering::Release);
        self.started = true;
        let running = self.running.clone();
        let mut registrations = std::mem::take(&mut self.socket_registrations);
        let topic_callbacks = self.topic_callbacks.clone();
        let outbound_rx = self
            .outbound_rx
            .take()
            .ok_or_else(|| BusError::Protocol("runtime already started".into()))?;
        let reply_rx = self
            .reply_rx
            .take()
            .ok_or_else(|| BusError::Protocol("runtime already started".into()))?;
        let reply_tx = self.reply_tx.clone();
        let worker_pool = self.worker_pool.clone();
        let handle = thread::spawn(move || {
            while running.load(Ordering::Acquire) {
                spin_once_inner(
                    &mut registrations,
                    &topic_callbacks,
                    &outbound_rx,
                    &reply_rx,
                    &reply_tx,
                    worker_pool.as_ref(),
                    DEFAULT_POLL_TIMEOUT_MS,
                );
            }
        });
        self.thread = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) {
        if self.closed {
            return;
        }
        self.running.store(false, Ordering::Release);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
        self.disconnect_workers();
        self.close_sockets();
        self.started = false;
        self.closed = true;
    }

    pub fn wait(&mut self) {
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
            self.thread = None;
        }
    }

    fn prepare_for_spin(&mut self) -> Result<()> {
        if self.socket_registrations.is_empty() {
            self.sync_all_registrations();
        }
        if self.socket_registrations.is_empty() {
            return Err(BusError::Protocol(
                "nothing registered; connect_subscriber or register worker first".into(),
            ));
        }
        Ok(())
    }

    fn ensure_open(&self) -> Result<()> {
        if self.closed {
            return Err(BusError::Closed);
        }
        Ok(())
    }

    fn has_sub_registration(&self) -> bool {
        self.socket_registrations
            .iter()
            .any(|reg| matches!(reg, Registration::Sub(_)))
    }

    fn has_action_client_registration(&self) -> bool {
        self.socket_registrations
            .iter()
            .any(|reg| matches!(reg, Registration::ActionClient(_)))
    }

    fn sync_action_client_registration(&mut self) {
        if let Some(reg) = self.action_client_registration.take() {
            self.socket_registrations
                .push(Registration::ActionClient(reg));
        }
    }

    fn sync_worker_registrations(&mut self) {
        while let Some(reg) = self.service_registrations.pop() {
            self.socket_registrations
                .push(Registration::Service(reg));
        }
        while let Some(reg) = self.action_registrations.pop() {
            self.socket_registrations.push(Registration::Action(reg));
        }
    }

    fn sync_all_registrations(&mut self) {
        self.sync_worker_registrations();
        self.sync_action_client_registration();
    }

    fn disconnect_workers(&mut self) {
        for reg in &self.socket_registrations {
            match reg {
                Registration::Service(worker) => worker.disconnect(),
                Registration::Action(worker) => worker.disconnect(),
                _ => {}
            }
        }
    }

    fn close_sockets(&mut self) {
        for reg in &self.socket_registrations {
            let _ = reg.socket().set_linger(0);
        }
        self.socket_registrations.clear();
        self.service_registrations.clear();
        self.action_registrations.clear();
        self.action_client_registration = None;
    }
}

impl Default for BusRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BusRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}

fn timeout_to_ms(timeout: Option<Duration>) -> i64 {
    match timeout {
        None => DEFAULT_POLL_TIMEOUT_MS,
        Some(duration) if duration.is_zero() => 0,
        Some(duration) => duration.as_millis().min(i64::MAX as u128) as i64,
    }
}

/// One poll + dispatch iteration. Returns whether any socket was readable.
fn spin_once_inner(
    registrations: &mut [Registration],
    topic_callbacks: &HashMap<String, Vec<MessageCallback>>,
    outbound_rx: &Receiver<OutboundCommand>,
    reply_rx: &Receiver<ReplyMessage>,
    reply_tx: &Sender<ReplyMessage>,
    worker_pool: Option<&WorkerPool>,
    poll_timeout_ms: i64,
) -> bool {
    let now = Instant::now();
    tick_heartbeats(registrations, now);
    for reg in registrations.iter_mut() {
        if let Registration::ActionClient(client) = reg {
            flush_outbound(client, outbound_rx);
        }
    }
    flush_reply_queue(registrations, reply_rx);

    let mut poll_items: Vec<zmq::PollItem> = registrations
        .iter()
        .map(|reg| reg.socket().as_poll_item(zmq::POLLIN))
        .collect();
    if zmq::poll(&mut poll_items, poll_timeout_ms).unwrap_or(0) == 0 {
        return false;
    }

    let readable: Vec<usize> = poll_items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.is_readable())
        .map(|(index, _)| index)
        .collect();
    for index in readable {
        dispatch_registration(
            &mut registrations[index],
            topic_callbacks,
            reply_tx,
            worker_pool,
        );
    }
    true
}
