//! Single- and multi-threaded executors.

use std::time::Duration;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use robot_bus::runtime::{
    MultiThreadedExecutor as RustMultiThreadedExecutor,
    SingleThreadedExecutor as RustSingleThreadedExecutor,
};

use crate::handles::ShutdownHandle;
use crate::node::{Context, Node};
use crate::util::{bus_err, node_options};

#[napi]
pub struct SingleThreadedExecutor {
    pub(crate) inner: RustSingleThreadedExecutor,
}

#[napi]
impl SingleThreadedExecutor {
    #[napi(constructor)]
    pub fn new(context: Option<&Context>) -> Self {
        Self {
            inner: match context {
                Some(c) => RustSingleThreadedExecutor::with_context(c.inner.clone()),
                None => RustSingleThreadedExecutor::new(),
            },
        }
    }

    #[napi]
    pub fn add_node(&self, node: &mut Node) -> Result<()> {
        self.inner.add_node(&mut node.inner).map_err(bus_err)
    }

    #[napi]
    pub fn create_node(
        &self,
        name: String,
        host: Option<String>,
        transport: Option<String>,
        ws_url: Option<String>,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
        service_frontend: Option<String>,
        service_backend: Option<String>,
        action_backend: Option<String>,
        action_frontend: Option<String>,
    ) -> Result<Node> {
        let host = host.unwrap_or_else(|| "localhost".into());
        let transport = transport.unwrap_or_else(|| "tcp".into());
        let options = node_options(
            &host,
            &transport,
            ws_url,
            message_xsub,
            message_xpub,
            service_frontend,
            service_backend,
            action_backend,
            action_frontend,
        )?;
        Ok(Node {
            inner: self
                .inner
                .create_node_with_options(name, options)
                .map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn shutdown_handle(&self) -> Result<ShutdownHandle> {
        Ok(ShutdownHandle {
            inner: self.inner.shutdown_handle().map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn shutdown(&self) -> Result<()> {
        self.inner.shutdown().map_err(bus_err)
    }

    #[napi]
    pub fn spin_once(&self, timeout: Option<f64>) -> Result<bool> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.spin_once(timeout).map_err(bus_err)
    }

    #[napi]
    pub fn spin(&self) -> Result<()> {
        self.inner.spin().map_err(bus_err)
    }

    #[napi]
    pub fn start(&self) -> Result<()> {
        self.inner.start().map_err(bus_err)
    }

    #[napi]
    pub fn stop(&self) -> Result<()> {
        self.inner.stop().map_err(bus_err)
    }

    #[napi]
    pub fn wait(&self) -> Result<()> {
        self.inner.wait().map_err(bus_err)
    }
}

#[napi]
pub struct MultiThreadedExecutor {
    pub(crate) inner: RustMultiThreadedExecutor,
}

#[napi]
impl MultiThreadedExecutor {
    #[napi(constructor)]
    pub fn new(num_threads: Option<u32>, context: Option<&Context>) -> Self {
        let n = num_threads.unwrap_or(4) as usize;
        Self {
            inner: match context {
                Some(c) => RustMultiThreadedExecutor::with_context(c.inner.clone(), n),
                None => RustMultiThreadedExecutor::new(n),
            },
        }
    }

    #[napi]
    pub fn add_node(&self, node: &mut Node) -> Result<()> {
        self.inner.add_node(&mut node.inner).map_err(bus_err)
    }

    #[napi]
    pub fn create_node(
        &self,
        name: String,
        host: Option<String>,
        transport: Option<String>,
        ws_url: Option<String>,
        message_xsub: Option<String>,
        message_xpub: Option<String>,
        service_frontend: Option<String>,
        service_backend: Option<String>,
        action_backend: Option<String>,
        action_frontend: Option<String>,
    ) -> Result<Node> {
        let host = host.unwrap_or_else(|| "localhost".into());
        let transport = transport.unwrap_or_else(|| "tcp".into());
        let options = node_options(
            &host,
            &transport,
            ws_url,
            message_xsub,
            message_xpub,
            service_frontend,
            service_backend,
            action_backend,
            action_frontend,
        )?;
        Ok(Node {
            inner: self
                .inner
                .create_node_with_options(name, options)
                .map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn shutdown_handle(&self) -> Result<ShutdownHandle> {
        Ok(ShutdownHandle {
            inner: self.inner.shutdown_handle().map_err(bus_err)?,
        })
    }

    #[napi]
    pub fn shutdown(&self) -> Result<()> {
        self.inner.shutdown().map_err(bus_err)
    }

    #[napi]
    pub fn spin_once(&self, timeout: Option<f64>) -> Result<bool> {
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.spin_once(timeout).map_err(bus_err)
    }

    #[napi]
    pub fn spin(&self) -> Result<()> {
        self.inner.spin().map_err(bus_err)
    }
}
