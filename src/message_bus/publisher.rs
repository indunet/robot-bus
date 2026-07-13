//! PUB socket that connects to the message bus XSUB side.

use zmq::{Context, Socket, SocketType};

use crate::errors::Result;
use crate::transports;
use crate::zmq_helpers::{apply_publisher_options, wait_for_connection};

pub struct Publisher {
    endpoint: String,
    context: Context,
    socket: Socket,
}

impl Publisher {
    pub fn new(endpoint: Option<&str>) -> Result<Self> {
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => transports::message_xsub_endpoint("localhost", "tcp")
                .map_err(|e| crate::errors::BusError::Protocol(e))?,
        };
        let context = Context::new();
        let socket = context.socket(SocketType::PUB)?;
        apply_publisher_options(&socket)?;
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
