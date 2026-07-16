//! ROS 2–style executors: [`SingleThreadedExecutor`] and [`MultiThreadedExecutor`].
//!
//! Create nodes with [`ExecutorHandle::create_node`], then drive callbacks with
//! `spin` / `spin_once` / `spin_some` on the executor (not on the node).

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::errors::{BusError, Result};
use crate::runtime::executor::{Executor, ShutdownHandle};
use crate::runtime::node::{Node, NodeOptions};

/// Shared handle to the underlying poll-loop [`Executor`].
///
/// Both executor wrappers and [`Node`] hold clones of this handle so that
/// `create_node` + `spin` match the ROS 2 ownership split without fighting
/// the borrow checker.
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

    /// Create a [`Node`] attached to this executor (ROS 2 `create_node` / `add_node`).
    pub fn create_node(&self, name: impl Into<String>) -> Node {
        self.create_node_with_options(name, NodeOptions::default())
    }

    /// Like [`create_node`](Self::create_node), with explicit broker endpoints.
    pub fn create_node_with_options(
        &self,
        name: impl Into<String>,
        options: NodeOptions,
    ) -> Node {
        Node::attach(name, options, self.clone())
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

    pub fn handle(&self) -> &ExecutorHandle {
        &self.handle
    }

    pub fn create_node(&self, name: impl Into<String>) -> Node {
        self.handle.create_node(name)
    }

    pub fn create_node_with_options(
        &self,
        name: impl Into<String>,
        options: NodeOptions,
    ) -> Node {
        self.handle.create_node_with_options(name, options)
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

/// Service / action handlers may run on a bounded worker pool
/// (simplified ROS 2 `MultiThreadedExecutor`).
///
/// Subscription, timer, and action-client callbacks still run on the I/O thread.
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

    pub fn handle(&self) -> &ExecutorHandle {
        &self.handle
    }

    pub fn create_node(&self, name: impl Into<String>) -> Node {
        self.handle.create_node(name)
    }

    pub fn create_node_with_options(
        &self,
        name: impl Into<String>,
        options: NodeOptions,
    ) -> Node {
        self.handle.create_node_with_options(name, options)
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
