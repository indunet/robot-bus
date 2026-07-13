//! DEALER worker for the action bus backend.

use std::sync::Arc;
use std::time::{Duration, Instant};

use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

use crate::errors::Result;
use crate::transports;
use crate::zmq_helpers::{apply_action_options, poll_readable};

pub type ActionGoalHandler =
    Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync>;

pub struct ActionWorker {
    pub action_name: String,
    endpoint: String,
    identity: Vec<u8>,
    heartbeat_interval: Duration,
    last_heartbeat: Instant,
    context: Context,
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

        let context = Context::new();
        let socket = context.socket(SocketType::DEALER)?;
        apply_action_options(&socket)?;
        socket.set_identity(&identity_bytes)?;
        socket.connect(&endpoint)?;
        let mut worker = Self {
            action_name: action_name.clone(),
            endpoint,
            identity: identity_bytes,
            heartbeat_interval,
            last_heartbeat: Instant::now(),
            context,
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

    fn send_control(&self, command: &[u8]) -> Result<()> {
        if let Some(socket) = &self.socket {
            socket.send_multipart([command, self.action_name.as_bytes()], 0)?;
        }
        Ok(())
    }

    fn maybe_send_heartbeat(&mut self) {
        if self.last_heartbeat.elapsed() >= self.heartbeat_interval {
            let _ = self.send_control(b"HEARTBEAT");
            self.last_heartbeat = Instant::now();
        }
    }

    fn reply(
        &self,
        client_id: &[u8],
        goal_id: &[u8],
        kind: &[u8],
        body: &[u8],
    ) -> Result<()> {
        if let Some(socket) = &self.socket {
            socket.send_multipart(
                [
                    client_id,
                    self.action_name.as_bytes(),
                    goal_id,
                    kind,
                    body,
                ],
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
        if kind_str == "CANCEL" || kind_str == "GOAL" {
            let payload = if kind_str == "GOAL" {
                body.as_slice()
            } else {
                kind.as_slice()
            };
            let replies = (self.handler)(client_id, goal_id, payload);
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
        let _ = self.send_control(b"DISCONNECT");
        self.socket = None;
    }
}

impl Drop for ActionWorker {
    fn drop(&mut self) {
        self.close();
    }
}
