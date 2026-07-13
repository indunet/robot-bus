//! Unified poll loop for subscriptions and worker callbacks.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

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
use crate::transports::{
    action_backend_endpoint, action_frontend_endpoint, message_xpub_endpoint,
    service_backend_endpoint,
};
use crate::zmq_helpers::{apply_subscriber_options, wait_for_connection};

const POLL_TIMEOUT_MS: i64 = 250;

pub struct BusRuntime {
    context: Context,
    heartbeat_interval_ms: u64,
    sub_registrations: Vec<SubRegistration>,
    service_registrations: Vec<ServiceRegistration>,
    action_registrations: Vec<ActionRegistration>,
    action_client_registration: Option<ActionClientRegistration>,
    topic_callbacks: HashMap<String, Vec<MessageCallback>>,
    socket_registrations: Vec<Registration>,
    outbound_tx: Sender<OutboundCommand>,
    outbound_rx: Option<Receiver<OutboundCommand>>,
    reply_tx: Sender<ReplyMessage>,
    reply_rx: Option<Receiver<ReplyMessage>>,
    use_thread_pool: bool,
    running: Arc<AtomicBool>,
    started: bool,
    closed: bool,
    thread: Option<JoinHandle<()>>,
}

impl BusRuntime {
    pub fn new() -> Self {
        Self::with_options(2500, false)
    }

    pub fn with_executor(max_workers: usize) -> Self {
        let _ = max_workers;
        Self::with_options(2500, true)
    }

    fn with_options(heartbeat_interval_ms: u64, use_thread_pool: bool) -> Self {
        let (outbound_tx, outbound_rx) = mpsc::channel();
        let (reply_tx, reply_rx) = mpsc::channel();
        Self {
            context: Context::new(),
            heartbeat_interval_ms,
            sub_registrations: Vec::new(),
            service_registrations: Vec::new(),
            action_registrations: Vec::new(),
            action_client_registration: None,
            topic_callbacks: HashMap::new(),
            socket_registrations: Vec::new(),
            outbound_tx,
            outbound_rx: Some(outbound_rx),
            reply_tx,
            reply_rx: Some(reply_rx),
            use_thread_pool,
            running: Arc::new(AtomicBool::new(false)),
            started: false,
            closed: false,
            thread: None,
        }
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

    pub fn start(&mut self) -> Result<()> {
        self.ensure_open()?;
        if self.started {
            return Ok(());
        }
        if self.socket_registrations.is_empty() {
            self.sync_all_registrations();
        }
        if self.socket_registrations.is_empty() {
            return Err(BusError::Protocol(
                "nothing registered; connect_subscriber or register worker first".into(),
            ));
        }
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
        let use_thread_pool = self.use_thread_pool;
        let handle = thread::spawn(move || {
            io_loop(
                &mut registrations,
                &topic_callbacks,
                &outbound_rx,
                &reply_rx,
                &reply_tx,
                use_thread_pool,
                &running,
            );
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

    pub fn spin(&mut self) -> Result<()> {
        self.ensure_open()?;
        if self.started {
            return Err(BusError::Protocol(
                "spin() cannot run while start() is active".into(),
            ));
        }
        if self.socket_registrations.is_empty() {
            self.sync_all_registrations();
        }
        if self.socket_registrations.is_empty() {
            return Err(BusError::Protocol(
                "nothing registered; connect_subscriber or register worker first".into(),
            ));
        }
        self.running.store(true, Ordering::Release);
        let running = self.running.clone();
        let outbound_rx = self
            .outbound_rx
            .as_ref()
            .ok_or_else(|| BusError::Protocol("runtime already started".into()))?;
        let reply_rx = self
            .reply_rx
            .as_ref()
            .ok_or_else(|| BusError::Protocol("runtime already started".into()))?;
        io_loop(
            &mut self.socket_registrations,
            &self.topic_callbacks,
            outbound_rx,
            reply_rx,
            &self.reply_tx,
            self.use_thread_pool,
            &running,
        );
        self.running.store(false, Ordering::Release);
        Ok(())
    }

    pub fn wait(&mut self) {
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
            self.thread = None;
        }
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

    fn sub_registrations_mut(&mut self) -> impl Iterator<Item = &mut SubRegistration> {
        self.socket_registrations.iter_mut().filter_map(|reg| {
            if let Registration::Sub(sub) = reg {
                Some(sub)
            } else {
                None
            }
        })
    }

    fn sync_action_client_registration(&mut self) {
        if let Some(reg) = self.action_client_registration.take() {
            self.socket_registrations.push(Registration::ActionClient(reg));
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
        for reg in self.sub_registrations.drain(..) {
            self.socket_registrations.push(Registration::Sub(reg));
        }
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
        self.sub_registrations.clear();
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

fn io_loop(
    registrations: &mut [Registration],
    topic_callbacks: &HashMap<String, Vec<MessageCallback>>,
    outbound_rx: &Receiver<OutboundCommand>,
    reply_rx: &Receiver<ReplyMessage>,
    reply_tx: &Sender<ReplyMessage>,
    use_thread_pool: bool,
    running: &AtomicBool,
) {
    while running.load(Ordering::Acquire) {
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
        if zmq::poll(&mut poll_items, POLL_TIMEOUT_MS).unwrap_or(0) == 0 {
            continue;
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
                use_thread_pool,
            );
        }
    }
}
