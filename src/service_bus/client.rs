//! REQ client for the service bus frontend.

use std::cell::RefCell;
use std::time::Duration;

use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

use crate::errors::{BusError, Result, parse_error_body};
use crate::transports;
use crate::zmq_helpers::{HighWaterMark, apply_rpc_options_with, poll_readable};

pub struct ServiceClient {
    context: Context,
    endpoint: String,
    hwm: RefCell<HighWaterMark>,
    socket: RefCell<Socket>,
}

impl ServiceClient {
    pub fn new(endpoint: Option<&str>) -> Result<Self> {
        Self::with_hwm(endpoint, HighWaterMark::RPC)
    }

    pub fn with_hwm(endpoint: Option<&str>, hwm: HighWaterMark) -> Result<Self> {
        Self::with_context_hwm(&Context::new(), endpoint, hwm)
    }

    /// Create a client using a shared ZeroMQ context (required for inproc).
    pub fn with_context_hwm(
        context: &Context,
        endpoint: Option<&str>,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => transports::service_frontend_endpoint("localhost", "tcp")
                .map_err(|e| BusError::Protocol(e))?,
        };
        let socket = Self::connect_socket(context, &endpoint, hwm)?;
        log::info!("service client connected to {endpoint}");
        Ok(Self {
            context: context.clone(),
            endpoint,
            hwm: RefCell::new(hwm),
            socket: RefCell::new(socket),
        })
    }

    fn connect_socket(context: &Context, endpoint: &str, hwm: HighWaterMark) -> Result<Socket> {
        let socket = context.socket(SocketType::REQ)?;
        apply_rpc_options_with(&socket, hwm)?;
        socket.connect(endpoint)?;
        Ok(socket)
    }

    /// Recreate the REQ socket after timeout / protocol errors leave it unusable.
    fn reset_socket(&self) -> Result<()> {
        let hwm = *self.hwm.borrow();
        {
            let sock = self.socket.borrow();
            let _ = sock.set_linger(0);
        }
        let new_sock = Self::connect_socket(&self.context, &self.endpoint, hwm)?;
        *self.socket.borrow_mut() = new_sock;
        Ok(())
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        Ok(HighWaterMark::from_socket(&self.socket.borrow())?)
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        hwm.apply(&self.socket.borrow())?;
        *self.hwm.borrow_mut() = hwm;
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
        {
            let sock = self.socket.borrow();
            if let Some(duration) = timeout {
                let ms = duration.as_millis().min(i32::MAX as u128) as i32;
                sock.set_sndtimeo(ms)?;
            }
            sock.send_multipart([service_name.as_bytes(), req_id.as_bytes(), body], 0)?;
        }

        if let Some(duration) = timeout {
            let ms = duration.as_millis().min(i64::MAX as u128) as i64;
            let readable = {
                let sock = self.socket.borrow();
                poll_readable(&sock, ms)?
            };
            if !readable {
                let _ = self.reset_socket();
                return Err(BusError::Timeout(format!(
                    "service '{service_name}' timed out after {}s",
                    duration.as_secs_f64()
                )));
            }
        }

        let frames = {
            let sock = self.socket.borrow();
            match sock.recv_multipart(0) {
                Ok(f) => f,
                Err(e) => {
                    drop(sock);
                    let _ = self.reset_socket();
                    return Err(e.into());
                }
            }
        };
        if frames.len() != 3 {
            let _ = self.reset_socket();
            return Err(BusError::Protocol(format!(
                "expected 3 reply frames, got {}",
                frames.len()
            )));
        }
        let reply_svc = String::from_utf8_lossy(&frames[0]);
        let reply_id = String::from_utf8_lossy(&frames[1]);
        if reply_svc != service_name {
            let _ = self.reset_socket();
            return Err(BusError::Protocol(format!(
                "service name mismatch: {reply_svc:?}"
            )));
        }
        if reply_id != req_id {
            let _ = self.reset_socket();
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
        let _ = self.socket.borrow().set_linger(0);
    }
}
