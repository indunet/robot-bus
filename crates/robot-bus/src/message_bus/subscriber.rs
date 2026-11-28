//! SUB socket that connects to the message bus XPUB side.

use std::time::{Duration, Instant};

use crate::zmq::{self, Context, Socket, SocketType};

use crate::errors::{BusError, Result};
use crate::runtime::retained_topic::is_internal_topic;
use crate::transports;
use crate::zmq_helpers::{
    HighWaterMark, apply_subscriber_options_with, poll_readable, wait_for_connection,
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

    /// Current socket high-water marks. The send side is unlimited (`0`) for
    /// subscription control commands.
    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        Ok(HighWaterMark::from_socket(&self.socket)?)
    }

    /// Update the receive high-water mark on the live socket. `hwm.snd` is ignored.
    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        self.socket.set_rcvhwm(hwm.rcv)?;
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
        let started = Instant::now();
        let timed_out = || {
            let secs = timeout.map(|d| d.as_secs_f64()).unwrap_or(0.0);
            BusError::Timeout(format!("no message within {secs}s"))
        };
        self.socket.set_rcvtimeo(-1)?;
        loop {
            if let Some(duration) = timeout {
                let remaining = duration.saturating_sub(started.elapsed());
                let ms = remaining.as_millis().min(i64::MAX as u128) as i64;
                if !poll_readable(&self.socket, ms)? {
                    return Err(timed_out());
                }
            }
            let mut frames = match self.socket.recv_multipart(0) {
                Ok(frames) => frames,
                Err(zmq::Error::EAGAIN) => return Err(timed_out()),
                Err(err) => return Err(err.into()),
            };
            if frames.len() < 2 {
                return Err(BusError::Protocol(format!(
                    "expected multipart [topic][payload], got {} frames",
                    frames.len()
                )));
            }
            let topic = String::from_utf8_lossy(&frames[0]);
            if is_internal_topic(&topic) {
                if timeout.is_some_and(|duration| started.elapsed() >= duration) {
                    return Err(timed_out());
                }
                continue;
            }
            return Ok((topic.into_owned(), std::mem::take(&mut frames[1])));
        }
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        let _ = self.socket.set_linger(0);
    }
}
