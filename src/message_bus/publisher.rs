//! PUB socket that connects to the message bus XSUB side.

use zmq::{Context, Socket, SocketType};

use crate::errors::Result;
use crate::transports;
use crate::zmq_helpers::{
    apply_publisher_options_with, wait_for_connection, HighWaterMark,
};

pub struct Publisher {
    endpoint: String,
    context: Context,
    socket: Socket,
}

impl Publisher {
    pub fn new(endpoint: Option<&str>) -> Result<Self> {
        Self::with_hwm(endpoint, HighWaterMark::STREAM)
    }

    pub fn with_hwm(endpoint: Option<&str>, hwm: HighWaterMark) -> Result<Self> {
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => transports::message_xsub_endpoint("localhost", "tcp")
                .map_err(|e| crate::errors::BusError::Protocol(e))?,
        };
        let context = Context::new();
        let socket = context.socket(SocketType::PUB)?;
        apply_publisher_options_with(&socket, hwm)?;
        socket.connect(&endpoint)?;
        wait_for_connection();
        log::info!("publisher connected to {endpoint}");
        Ok(Self {
            endpoint,
            context,
            socket,
        })
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
        self.socket
            .send_multipart([topic.as_bytes(), payload], 0)?;
        Ok(())
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        let _ = self.socket.set_linger(0);
    }
}
