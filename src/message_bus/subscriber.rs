//! SUB socket that connects to the message bus XPUB side.

use std::time::Duration;

use zmq::{Context, Socket, SocketType};

use crate::errors::{BusError, Result};
use crate::transports;
use crate::zmq_helpers::{
    apply_subscriber_options_with, poll_readable, wait_for_connection, HighWaterMark,
};

pub struct Subscriber {
    endpoint: String,
    socket: Socket,
}

impl Subscriber {
    pub fn new(endpoint: Option<&str>) -> Result<Self> {
        Self::with_hwm(endpoint, HighWaterMark::STREAM)
    }

    pub fn with_hwm(endpoint: Option<&str>, hwm: HighWaterMark) -> Result<Self> {
        Self::with_context_hwm(&Context::new(), endpoint, hwm)
    }

    /// Create a subscriber using a shared ZeroMQ context (required for inproc).
    pub fn with_context_hwm(
        context: &Context,
        endpoint: Option<&str>,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => transports::message_xpub_endpoint("localhost", "tcp")
                .map_err(|e| BusError::Protocol(e))?,
        };
        let socket = context.socket(SocketType::SUB)?;
        apply_subscriber_options_with(&socket, hwm)?;
        socket.connect(&endpoint)?;
        wait_for_connection();
        log::info!("subscriber connected to {endpoint}");
        Ok(Self { endpoint, socket })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Current send / receive high-water marks.
    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        Ok(HighWaterMark::from_socket(&self.socket)?)
    }

    /// Update send / receive high-water marks on the live socket.
    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        hwm.apply(&self.socket)?;
        Ok(())
    }

    pub fn subscribe(&self, topic: &str) -> Result<()> {
        self.socket.set_subscribe(topic.as_bytes())?;
        Ok(())
    }

    pub fn unsubscribe(&self, topic: &str) -> Result<()> {
        self.socket.set_unsubscribe(topic.as_bytes())?;
        Ok(())
    }

    /// Return `(topic, payload)`. Returns `BusError::Timeout` on timeout.
    pub fn receive(&self, timeout: Option<Duration>) -> Result<(String, Vec<u8>)> {
        if let Some(duration) = timeout {
            let ms = duration.as_millis().min(i64::MAX as u128) as i64;
            self.socket.set_rcvtimeo(ms as i32)?;
            if !poll_readable(&self.socket, ms)? {
                return Err(BusError::Timeout(format!(
                    "no message within {}s",
                    duration.as_secs_f64()
                )));
            }
        } else {
            self.socket.set_rcvtimeo(-1)?;
        }

        let frames = match self.socket.recv_multipart(0) {
            Ok(frames) => frames,
            Err(zmq::Error::EAGAIN) => {
                let secs = timeout.map(|d| d.as_secs_f64()).unwrap_or(0.0);
                return Err(BusError::Timeout(format!("no message within {secs}s")));
            }
            Err(err) => return Err(err.into()),
        };

        if frames.len() < 2 {
            return Err(BusError::Protocol(format!(
                "expected multipart [topic][payload], got {} frames",
                frames.len()
            )));
        }
        let topic = String::from_utf8_lossy(&frames[0]).into_owned();
        Ok((topic, frames[1].clone()))
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        let _ = self.socket.set_linger(0);
    }
}
