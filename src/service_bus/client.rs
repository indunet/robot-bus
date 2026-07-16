//! REQ client for the service bus frontend.

use std::time::Duration;

use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

use crate::errors::{parse_error_body, BusError, Result};
use crate::transports;
use crate::zmq_helpers::{apply_rpc_options_with, poll_readable, HighWaterMark};

pub struct ServiceClient {
    endpoint: String,
    context: Context,
    socket: Socket,
}

impl ServiceClient {
    pub fn new(endpoint: Option<&str>) -> Result<Self> {
        Self::with_hwm(endpoint, HighWaterMark::RPC)
    }

    pub fn with_hwm(endpoint: Option<&str>, hwm: HighWaterMark) -> Result<Self> {
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => transports::service_frontend_endpoint("localhost", "tcp")
                .map_err(|e| BusError::Protocol(e))?,
        };
        let context = Context::new();
        let socket = context.socket(SocketType::REQ)?;
        apply_rpc_options_with(&socket, hwm)?;
        socket.connect(&endpoint)?;
        log::info!("service client connected to {endpoint}");
        Ok(Self {
            endpoint,
            context,
            socket,
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        Ok(HighWaterMark::from_socket(&self.socket)?)
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        hwm.apply(&self.socket)?;
        Ok(())
    }

    pub fn call(
        &self,
        service_name: &str,
        body: &[u8],
        request_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>> {
        let req_id = request_id
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
        self.socket.send_multipart(
            [
                service_name.as_bytes(),
                req_id.as_bytes(),
                body,
            ],
            0,
        )?;

        if let Some(duration) = timeout {
            let ms = duration.as_millis().min(i64::MAX as u128) as i64;
            if !poll_readable(&self.socket, ms)? {
                return Err(BusError::Timeout(format!(
                    "service '{service_name}' timed out after {}s",
                    duration.as_secs_f64()
                )));
            }
        }

        let frames = self.socket.recv_multipart(0)?;
        if frames.len() != 3 {
            return Err(BusError::Protocol(format!(
                "expected 3 reply frames, got {}",
                frames.len()
            )));
        }
        let reply_svc = String::from_utf8_lossy(&frames[0]);
        let reply_id = String::from_utf8_lossy(&frames[1]);
        if reply_svc != service_name {
            return Err(BusError::Protocol(format!(
                "service name mismatch: {reply_svc:?}"
            )));
        }
        if reply_id != req_id {
            return Err(BusError::Protocol(format!(
                "request id mismatch: {reply_id:?}"
            )));
        }
        if let Some(err) = parse_error_body(&frames[2]) {
            return Err(err);
        }
        Ok(frames[2].clone())
    }
}

impl Drop for ServiceClient {
    fn drop(&mut self) {
        let _ = self.socket.set_linger(0);
    }
}
