//! ROS 2–style executors: [`SingleThreadedExecutor`] and [`MultiThreadedExecutor`].
//!
//! Simple single-node path can skip these and call [`Node::spin`](crate::runtime::Node::spin)
//! (auto SingleThreadedExecutor). Shared / multi-threaded flow:
//! ```ignore
//! let mut node = Node::new("pilot");
//! let executor = SingleThreadedExecutor::new();
//! executor.add_node(&mut node)?;
//! executor.spin()?;
//! ```

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::errors::{BusError, Result};
use crate::runtime::context::Context;
use crate::runtime::executor::{Executor, ShutdownHandle};
use crate::runtime::node::{Node, NodeOptions};

/// Shared handle to the underlying poll-loop [`Executor`].
#[derive(Clone)]
pub struct ExecutorHandle {
    inner: Arc<Mutex<Executor>>,
}

impl ExecutorHandle {
    fn new(executor: Executor) -> Self {
        Self {
            inner: Arc::new(Mutex::new(executor)),
        }
    }

    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Executor>> {
        self.inner
            .lock()
            .map_err(|_| BusError::Protocol("executor mutex poisoned".into()))
    }

    /// Attach an existing [`Node`] (ROS 2 `add_node`).
    pub fn add_node(&self, node: &mut Node) -> Result<()> {
        node.attach_executor(self.clone())
    }

    /// Convenience: `Node::new` + [`add_node`](Self::add_node).
    pub fn create_node(&self, name: impl Into<String>) -> Result<Node> {
        self.create_node_with_options(name, NodeOptions::default())
    }

    /// Convenience: `Node::with_options` + [`add_node`](Self::add_node).
    pub fn create_node_with_options(
        &self,
        name: impl Into<String>,
        options: NodeOptions,
    ) -> Result<Node> {
        let context = self.lock()?.context().clone();
        let mut node = Node::with_context(context, name, options);
        self.add_node(&mut node)?;
        Ok(node)
    }

    /// Convenience: shared-context node + [`add_node`](Self::add_node).
    pub fn create_node_with_context(
        &self,
        context: Context,
        name: impl Into<String>,
        options: NodeOptions,
    ) -> Result<Node> {
        let mut node = Node::with_context(context, name, options);
        self.add_node(&mut node)?;
        Ok(node)
    }

    pub fn shutdown_handle(&self) -> Result<ShutdownHandle> {
        Ok(self.lock()?.shutdown_handle())
    }

    pub fn shutdown(&self) -> Result<()> {
        self.lock()?.shutdown();
        Ok(())
    }

    pub fn spin_once(&self, timeout: Option<Duration>) -> Result<bool> {
        self.lock()?.spin_once(timeout)
    }

    pub fn spin_some(&self, timeout: Option<Duration>) -> Result<()> {
        self.lock()?.spin_some(timeout)
    }

    pub fn spin(&self) -> Result<()> {
        self.lock()?.spin()
    }

    pub fn start(&self) -> Result<()> {
        self.lock()?.start()
    }

    pub fn stop(&self) -> Result<()> {
        self.lock()?.stop();
        Ok(())
    }

    pub fn wait(&self) -> Result<()> {
        self.lock()?.wait();
        Ok(())
    }
}

/// All callbacks run on the spin / I/O thread (ROS 2 `SingleThreadedExecutor`).
#[derive(Clone)]
pub struct SingleThreadedExecutor {
    handle: ExecutorHandle,
}

impl Default for SingleThreadedExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SingleThreadedExecutor {
    pub fn new() -> Self {
        Self {
            handle: ExecutorHandle::new(Executor::new()),
        }
    }

    /// Executor whose sockets share `context` (required for inproc with broker/Nodes).
    pub fn with_context(context: Context) -> Self {
        Self {
            handle: ExecutorHandle::new(Executor::with_context(context)),
        }
    }

    pub fn handle(&self) -> &ExecutorHandle {
        &self.handle
    }

    pub fn add_node(&self, node: &mut Node) -> Result<()> {
        self.handle.add_node(node)
    }

    pub fn create_node(&self, name: impl Into<String>) -> Result<Node> {
        self.handle.create_node(name)
    }

    pub fn create_node_with_options(
        &self,
        name: impl Into<String>,
        options: NodeOptions,
    ) -> Result<Node> {
        self.handle.create_node_with_options(name, options)
    }

    pub fn create_node_with_context(
        &self,
        context: Context,
        name: impl Into<String>,
        options: NodeOptions,
    ) -> Result<Node> {
        self.handle
            .create_node_with_context(context, name, options)
    }

    pub fn shutdown_handle(&self) -> Result<ShutdownHandle> {
        self.handle.shutdown_handle()
    }

    pub fn shutdown(&self) -> Result<()> {
        self.handle.shutdown()
    }

    pub fn spin_once(&self, timeout: Option<Duration>) -> Result<bool> {
        self.handle.spin_once(timeout)
    }

    pub fn spin_some(&self, timeout: Option<Duration>) -> Result<()> {
        self.handle.spin_some(timeout)
    }

    pub fn spin(&self) -> Result<()> {
        self.handle.spin()
    }

    pub fn start(&self) -> Result<()> {
        self.handle.start()
    }

    pub fn stop(&self) -> Result<()> {
        self.handle.stop()
    }

    pub fn wait(&self) -> Result<()> {
        self.handle.wait()
    }
}

/// Service / action / subscription / timer callbacks may use a bounded worker
/// pool (simplified ROS 2 `MultiThreadedExecutor`), subject to each
/// [`crate::runtime::CallbackGroup`]'s type.
#[derive(Clone)]
pub struct MultiThreadedExecutor {
    handle: ExecutorHandle,
}

impl MultiThreadedExecutor {
    /// `num_threads` is the max concurrent service/action handler workers.
    pub fn new(num_threads: usize) -> Self {
        Self {
            handle: ExecutorHandle::new(Executor::with_worker_pool(num_threads)),
        }
    }

    /// Like [`new`](Self::new), sharing `context` for ZMQ sockets.
    pub fn with_context(context: Context, num_threads: usize) -> Self {
        Self {
            handle: ExecutorHandle::new(Executor::with_context_and_worker_pool(
                context,
                num_threads,
            )),
        }
    }

    pub fn handle(&self) -> &ExecutorHandle {
        &self.handle
    }

    pub fn add_node(&self, node: &mut Node) -> Result<()> {
        self.handle.add_node(node)
    }

    pub fn create_node(&self, name: impl Into<String>) -> Result<Node> {
        self.handle.create_node(name)
    }

    pub fn create_node_with_options(
        &self,
        name: impl Into<String>,
        options: NodeOptions,
    ) -> Result<Node> {
        self.handle.create_node_with_options(name, options)
    }

    pub fn create_node_with_context(
        &self,
        context: Context,
        name: impl Into<String>,
        options: NodeOptions,
    ) -> Result<Node> {
        self.handle
            .create_node_with_context(context, name, options)
    }

    pub fn shutdown_handle(&self) -> Result<ShutdownHandle> {
        self.handle.shutdown_handle()
    }

    pub fn shutdown(&self) -> Result<()> {
        self.handle.shutdown()
    }

    pub fn spin_once(&self, timeout: Option<Duration>) -> Result<bool> {
        self.handle.spin_once(timeout)
    }

    pub fn spin_some(&self, timeout: Option<Duration>) -> Result<()> {
        self.handle.spin_some(timeout)
    }

    pub fn spin(&self) -> Result<()> {
        self.handle.spin()
    }

    pub fn start(&self) -> Result<()> {
        self.handle.start()
    }

    pub fn stop(&self) -> Result<()> {
        self.handle.stop()
    }

    pub fn wait(&self) -> Result<()> {
        self.handle.wait()
    }
}
