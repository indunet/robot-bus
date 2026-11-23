//! PUB socket that connects to the message bus XSUB side.
//!
//! The PUB socket is guarded by a [`Mutex`] so a publisher handle is `Send + Sync`
//! and safe to publish from multiple threads (sends are serialised per handle).

use std::sync::Mutex;

use zmq::{Context as ZmqContext, Socket, SocketType};

use crate::errors::{BusError, Result};
use crate::runtime::Context;
use crate::transports;
use crate::zmq_helpers::{HighWaterMark, apply_publisher_options_with, wait_for_connection};

pub struct Publisher {
    endpoint: String,
    socket: Mutex<Socket>,
}

impl Publisher {
    pub fn new(endpoint: Option<&str>) -> Result<Self> {
        Self::with_hwm(endpoint, HighWaterMark::STREAM)
    }

    pub fn with_hwm(endpoint: Option<&str>, hwm: HighWaterMark) -> Result<Self> {
        Self::with_zmq_hwm(&ZmqContext::new(), endpoint, hwm)
    }

    /// Create a publisher using a shared [`Context`] (required for inproc).
    pub fn with_shared_context(
        context: &Context,
        endpoint: Option<&str>,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        Self::with_zmq_hwm(context.zmq(), endpoint, hwm)
    }

    pub(crate) fn with_context_hwm(
        context: &ZmqContext,
        endpoint: Option<&str>,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        Self::with_zmq_hwm(context, endpoint, hwm)
    }

    fn with_zmq_hwm(
        context: &ZmqContext,
        endpoint: Option<&str>,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => transports::message_xsub_endpoint("localhost", "tcp")
                .map_err(|e| crate::errors::BusError::Protocol(e))?,
        };
        let socket = context.socket(SocketType::PUB)?;
        apply_publisher_options_with(&socket, hwm)?;
        socket.connect(&endpoint)?;
        wait_for_connection();
        log::info!("publisher connected to {endpoint}");
        Ok(Self {
            endpoint,
            socket: Mutex::new(socket),
        })
    }

    fn lock_socket(&self) -> Result<std::sync::MutexGuard<'_, Socket>> {
        self.socket
            .lock()
            .map_err(|_| BusError::Protocol("publisher socket mutex poisoned".into()))
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Current send / receive high-water marks.
    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        let sock = self.lock_socket()?;
        Ok(HighWaterMark::from_socket(&sock)?)
    }

    /// Update send / receive high-water marks on the live socket.
    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        let sock = self.lock_socket()?;
        hwm.apply(&sock)?;
        Ok(())
    }

    /// Milliseconds; `-1` waits forever. Used by console status publisher so
    /// shutdown cannot block on a full capture HWM.
    pub fn set_send_timeout_ms(&self, ms: i32) -> Result<()> {
        let sock = self.lock_socket()?;
        sock.set_sndtimeo(ms)?;
        Ok(())
    }

    pub fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let sock = self.lock_socket()?;
        sock.send_multipart([topic.as_bytes(), payload], 0)?;
        Ok(())
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        if let Ok(sock) = self.socket.get_mut() {
            let _ = sock.set_linger(0);
        }
    }
}

#[cfg(test)]
mod sync_assert {
    use super::Publisher;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn publisher_is_send_sync() {
        assert_send_sync::<Publisher>();
    }
}
