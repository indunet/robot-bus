//! DEALER worker for the service bus backend.

use std::sync::Arc;
use std::time::{Duration, Instant};

use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

use crate::errors::{BusError, Result};
use crate::transports;
use crate::zmq_helpers::{apply_rpc_options_with, poll_readable, HighWaterMark};

pub type ServiceHandler = Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>;

pub struct ServiceWorker {
    pub service_name: String,
    endpoint: String,
    identity: Vec<u8>,
    heartbeat_interval: Duration,
    last_heartbeat: Instant,
    context: Context,
    socket: Option<Socket>,
    handler: ServiceHandler,
}

impl ServiceWorker {
    pub fn new(
        service_name: impl Into<String>,
        handler: ServiceHandler,
        endpoint: Option<&str>,
        identity: Option<&str>,
        heartbeat_interval_ms: u64,
    ) -> Result<Self> {
        Self::with_hwm(
            service_name,
            handler,
            endpoint,
            identity,
            heartbeat_interval_ms,
            HighWaterMark::RPC,
        )
    }

    pub fn with_hwm(
        service_name: impl Into<String>,
        handler: ServiceHandler,
        endpoint: Option<&str>,
        identity: Option<&str>,
        heartbeat_interval_ms: u64,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        let service_name = service_name.into();
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => transports::service_backend_endpoint("localhost", "tcp")
                .map_err(|e| crate::errors::BusError::Protocol(e))?,
        };
        let identity = identity
            .map(str::to_string)
            .unwrap_or_else(|| format!("worker-{}", &Uuid::new_v4().simple().to_string()[..8]));
        let identity_bytes = identity.into_bytes();
        let heartbeat_interval = Duration::from_millis(heartbeat_interval_ms);

        let context = Context::new();
        let socket = context.socket(SocketType::DEALER)?;
        apply_rpc_options_with(&socket, hwm)?;
        socket.set_identity(&identity_bytes)?;
        socket.connect(&endpoint)?;
        let mut worker = Self {
            service_name: service_name.clone(),
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
            "service worker {:?} registered for {service_name} on {}",
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
            socket.send_multipart([command, self.service_name.as_bytes()], 0)?;
        }
        Ok(())
    }

    fn maybe_send_heartbeat(&mut self) {
        if self.last_heartbeat.elapsed() >= self.heartbeat_interval {
            let _ = self.send_control(b"HEARTBEAT");
            self.last_heartbeat = Instant::now();
        }
    }

    /// Handle one request if available. Returns `false` on poll timeout.
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
        if frames.len() != 4 {
            log::warn!("ignored frame with count {}", frames.len());
            return Ok(true);
        }
        let client_id = &frames[0];
        let svc = String::from_utf8_lossy(&frames[1]);
        let req_id = &frames[2];
        let body = &frames[3];
        if svc != self.service_name {
            log::warn!("ignored request for service {svc:?}");
            return Ok(true);
        }

        let reply_body = (self.handler)(body);

        if let Some(socket) = &self.socket {
            let _ = socket.send_multipart(
                [client_id, frames[1].as_slice(), req_id, reply_body.as_slice()],
                0,
            );
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

impl Drop for ServiceWorker {
    fn drop(&mut self) {
        self.close();
    }
}
