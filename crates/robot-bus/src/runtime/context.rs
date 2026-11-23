//! Process-scoped runtime [`Context`]: owns the shared ZeroMQ context.
//!
//! Prefer creating one [`Context`] per process (ROS 2–style) and building nodes with
//! [`crate::Node::with_context`] / [`crate::Node::with_context_options`].
//!
//! ZeroMQ `inproc://` endpoints are **context-local**. For same-process inproc
//! between an embedded [`crate::RobotBusBroker`] and [`crate::Node`]s, create one
//! [`Context`], pass it to [`RobotBusBroker::start_with_context`](crate::RobotBusBroker::start_with_context)
//! and to every Node / executor that connects inproc.
//!
//! TCP and IPC work across separate contexts; sharing is only required for inproc.
//! [`crate::Node::new`] still creates a private context for the simple tcp path.

use zmq::Context as ZmqContext;

/// Shared runtime handle (ROS 2–style entry point for ZMQ participants).
///
/// Internally holds a cloneable [`zmq::Context`]. Clone is cheap (refcounted).
#[derive(Clone)]
pub struct Context {
    inner: ZmqContext,
}

impl Context {
    /// Create a new ZeroMQ context.
    pub fn new() -> Self {
        Self {
            inner: ZmqContext::new(),
        }
    }

    /// Underlying ZeroMQ context (for creating sockets).
    pub(crate) fn zmq(&self) -> &ZmqContext {
        &self.inner
    }

    /// Clone of the underlying ZeroMQ context.
    pub(crate) fn clone_zmq(&self) -> ZmqContext {
        self.inner.clone()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
