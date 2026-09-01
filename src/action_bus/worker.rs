//! DEALER worker for the action bus backend.

use std::sync::Arc;
use std::time::{Duration, Instant};

use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

use crate::errors::{BusError, Result};
use crate::transports;
use crate::zmq_helpers::{apply_action_options_with, poll_readable, HighWaterMark};

pub type ActionGoalHandler = Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync>;

pub struct ActionWorker {
    pub action_name: String,
    endpoint: String,
    identity: Vec<u8>,
    heartbeat_interval: Duration,
    last_heartbeat: Instant,
    socket: Option<Socket>,
    handler: ActionGoalHandler,
}

impl ActionWorker {
    pub fn new(
        action_name: impl Into<String>,
        handler: ActionGoalHandler,
        endpoint: Option<&str>,
        identity: Option<&str>,
        heartbeat_interval_ms: u64,
    ) -> Result<Self> {
        Self::with_hwm(
            action_name,
            handler,
            endpoint,
            identity,
            heartbeat_interval_ms,
            HighWaterMark::ACTION,
        )
    }

    pub fn with_hwm(
        action_name: impl Into<String>,
        handler: ActionGoalHandler,
        endpoint: Option<&str>,
        identity: Option<&str>,
        heartbeat_interval_ms: u64,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        Self::with_context_hwm(
            &Context::new(),
            action_name,
            handler,
            endpoint,
            identity,
            heartbeat_interval_ms,
            hwm,
        )
    }

    /// Create a worker using a shared ZeroMQ context (required for inproc).
    pub fn with_context_hwm(
        context: &Context,
        action_name: impl Into<String>,
        handler: ActionGoalHandler,
        endpoint: Option<&str>,
        identity: Option<&str>,
        heartbeat_interval_ms: u64,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        let action_name = action_name.into();
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => transports::action_backend_endpoint("localhost", "tcp")
                .map_err(|e| crate::errors::BusError::Protocol(e))?,
        };
        let identity = identity
            .map(str::to_string)
            .unwrap_or_else(|| format!("worker-{}", &Uuid::new_v4().simple().to_string()[..8]));
        let identity_bytes = identity.into_bytes();
        let heartbeat_interval = Duration::from_millis(heartbeat_interval_ms);

        let socket = context.socket(SocketType::DEALER)?;
        apply_action_options_with(&socket, hwm)?;
        socket.set_identity(&identity_bytes)?;
        socket.connect(&endpoint)?;
        let mut worker = Self {
            action_name: action_name.clone(),
            endpoint,
            identity: identity_bytes,
            heartbeat_interval,
            last_heartbeat: Instant::now(),
            socket: Some(socket),
            handler,
        };
        worker.send_control(b"READY")?;
        worker.last_heartbeat = Instant::now();
        log::info!(
            "action worker {:?} registered for {action_name} on {}",
            String::from_utf8_lossy(&worker.identity),
            worker.endpoint
        );
        Ok(worker)
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        let Some(socket) = &self.socket else {
            return Err(BusError::Closed);
        };
        Ok(HighWaterMark::from_socket(socket)?)
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        let Some(socket) = &self.socket else {
            return Err(BusError::Closed);
        };
        hwm.apply(socket)?;
        Ok(())
    }

    fn send_control(&self, command: &[u8]) -> Result<()> {
        if let Some(socket) = &self.socket {
            socket.send_multipart([command, self.action_name.as_bytes()], 0)?;
        }
        Ok(())
    }

    fn maybe_send_heartbeat(&mut self) {
        if self.last_heartbeat.elapsed() >= self.heartbeat_interval {
            if let Some(socket) = &self.socket {
                // Must not block serve_once / worker join if the backend HWM is full.
                let _ = socket.send_multipart(
                    [b"HEARTBEAT".as_slice(), self.action_name.as_bytes()],
                    zmq::DONTWAIT,
                );
            }
            self.last_heartbeat = Instant::now();
        }
    }

    fn reply(&self, client_id: &[u8], goal_id: &[u8], kind: &[u8], body: &[u8]) -> Result<()> {
        if let Some(socket) = &self.socket {
            socket.send_multipart(
                [client_id, self.action_name.as_bytes(), goal_id, kind, body],
                0,
            )?;
        }
        Ok(())
    }

    /// Handle one message if available. Returns `false` on poll timeout.
    pub fn serve_once(&mut self, timeout_ms: i64) -> Result<bool> {
        self.maybe_send_heartbeat();
        let socket = match &self.socket {
            Some(sock) => sock,
            None => return Ok(false),
        };
        if !poll_readable(socket, timeout_ms)? {
            return Ok(false);
        }

        let frames = match socket.recv_multipart(0) {
            Ok(frames) => frames,
            Err(_) => return Ok(false),
        };
        if frames.len() != 5 {
            log::warn!("ignored frame with count {}", frames.len());
            return Ok(true);
        }
        let client_id = &frames[0];
        let action = String::from_utf8_lossy(&frames[1]);
        let goal_id = &frames[2];
        let kind = &frames[3];
        let body = &frames[4];
        if action != self.action_name {
            log::warn!("ignored message for action {action:?}");
            return Ok(true);
        }
        let kind_str = String::from_utf8_lossy(kind);
        if kind_str == "CANCEL" {
            // Sequential serve_once cannot interrupt an in-flight GOAL on this
            // socket. Node dispatch (ActionRegistration) handles CANCEL via
            // ActionGoalContext. Do not treat CANCEL as a new goal payload.
            return Ok(true);
        }
        if kind_str == "GOAL" {
            let replies = (self.handler)(body);
            for (phase, chunk) in replies {
                let _ = self.reply(client_id, goal_id, phase.as_bytes(), &chunk);
            }
        } else {
            log::warn!("ignored kind {kind_str:?}");
        }
        Ok(true)
    }

    pub fn serve(&mut self) {
        while self.socket.is_some() {
            if self.serve_once(500).unwrap_or(false) {
                continue;
            }
        }
    }

    pub fn close(&mut self) {
        if let Some(socket) = &self.socket {
            let _ = socket.send_multipart(
                [b"DISCONNECT".as_slice(), self.action_name.as_bytes()],
                zmq::DONTWAIT,
            );
        }
        self.socket = None;
    }
}

impl Drop for ActionWorker {
    fn drop(&mut self) {
        self.close();
    }
}
