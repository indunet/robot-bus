//! PUB socket that connects to the message bus XSUB side.

use zmq::{Context as ZmqContext, Socket, SocketType};

use crate::errors::Result;
use crate::runtime::Context;
use crate::transports;
use crate::zmq_helpers::{HighWaterMark, apply_publisher_options_with, wait_for_connection};

pub struct Publisher {
    endpoint: String,
    socket: Socket,
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

    pub fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        self.socket.send_multipart([topic.as_bytes(), payload], 0)?;
        Ok(())
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        let _ = self.socket.set_linger(0);
    }
}
