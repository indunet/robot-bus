use std::marker::PhantomData;
use std::sync::Arc;

use prost::Message;

use crate::errors::Result;
use crate::runtime::callback_group::CallbackGroup;
use crate::runtime::qos::QosProfile;
use crate::runtime::registrations::ServiceHandler;
use crate::service_bus::ServiceClient as BusServiceClient;
use crate::typed::Service;
use crate::zmq_helpers::HighWaterMark;

use super::super::Node;
use super::super::options::ws_mode_unsupported;
use super::super::service_clients::{
    NodeService, NodeServiceClient, NodeServiceClientRaw, ServiceClientInner,
};

impl Node {
    /// Register a typed service server (ROS 2 / rclrs `create_service`).
    ///
    /// Decode failures log a warning and return an empty response body.
    pub fn create_service<S, F>(
        &mut self,
        service_name: &str,
        handler: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeService>
    where
        S: Service,
        F: Fn(S::Request) -> S::Response + Send + Sync + 'static,
    {
        let cb: ServiceHandler = Arc::new(move |body| match S::Request::decode(body) {
            Ok(req) => handler(req).encode_to_vec(),
            Err(err) => {
                log::warn!("typed service decode failed: {err}");
                Vec::new()
            }
        });
        self.create_service_raw(service_name, cb, callback_group)
    }

    /// Register a typed service with KeepLast depth → DEALER HWM.
    pub fn create_service_with_qos<S, F>(
        &mut self,
        service_name: &str,
        qos: QosProfile,
        handler: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeService>
    where
        S: Service,
        F: Fn(S::Request) -> S::Response + Send + Sync + 'static,
    {
        let cb: ServiceHandler = Arc::new(move |body| match S::Request::decode(body) {
            Ok(req) => handler(req).encode_to_vec(),
            Err(err) => {
                log::warn!("typed service decode failed: {err}");
                Vec::new()
            }
        });
        self.create_service_raw_with_qos(service_name, qos, cb, callback_group)
    }

    /// Register a raw-bytes service server.
    pub fn create_service_raw(
        &mut self,
        service_name: &str,
        handler: ServiceHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeService> {
        self.create_service_raw_inner(service_name, handler, callback_group, None)
    }

    /// Register a raw-bytes service with KeepLast depth → DEALER HWM.
    pub fn create_service_raw_with_qos(
        &mut self,
        service_name: &str,
        qos: QosProfile,
        handler: ServiceHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeService> {
        self.create_service_raw_inner(service_name, handler, callback_group, Some(qos.to_hwm()))
    }

    fn create_service_raw_inner(
        &mut self,
        service_name: &str,
        handler: ServiceHandler,
        callback_group: Option<&CallbackGroup>,
        hwm: Option<HighWaterMark>,
    ) -> Result<NodeService> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("create_service"));
        }
        self.ensure_connected()?;
        let endpoint = self.options.service_backend_endpoint()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        let id = self.lock_executor()?.register_service(
            service_name,
            handler,
            group,
            Some(&endpoint),
            None,
            hwm,
        )?;
        let topology = self.start_topology_guard("service_server", service_name);
        self.topology_services.insert(id, topology);
        Ok(NodeService {
            id,
            service_name: service_name.to_string(),
        })
    }

    /// Destroy a service server created by [`create_service`](Self::create_service).
    /// Same `start()` constraint as [`cancel_timer`](Self::cancel_timer).
    pub fn destroy_service(&mut self, handle: &NodeService) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("destroy_service"));
        }
        self.topology_services.remove(&handle.id);
        self.lock_executor()?.destroy_service(handle.id)
    }

    /// Create a typed service client (ROS 2 / rclrs `create_client`).
    pub fn create_client<S: Service>(
        &mut self,
        service_name: impl Into<String>,
    ) -> Result<NodeServiceClient<S>> {
        Ok(NodeServiceClient {
            inner: self.create_client_raw(service_name)?,
            _marker: PhantomData,
        })
    }

    /// Create a typed service client with KeepLast depth → DEALER HWM.
    pub fn create_client_with_qos<S: Service>(
        &mut self,
        service_name: impl Into<String>,
        qos: QosProfile,
    ) -> Result<NodeServiceClient<S>> {
        Ok(NodeServiceClient {
            inner: self.create_client_raw_with_qos(service_name, qos)?,
            _marker: PhantomData,
        })
    }

    /// Create a raw-bytes service client bound to `service_name`.
    pub fn create_client_raw(
        &mut self,
        service_name: impl Into<String>,
    ) -> Result<NodeServiceClientRaw> {
        self.create_client_raw_with_hwm(service_name, self.client_rpc_hwm())
    }

    /// Create a raw-bytes service client with KeepLast depth → DEALER HWM.
    pub fn create_client_raw_with_qos(
        &mut self,
        service_name: impl Into<String>,
        qos: QosProfile,
    ) -> Result<NodeServiceClientRaw> {
        self.create_client_raw_with_hwm(service_name, qos.to_hwm())
    }

    /// Like [`create_client_raw`](Self::create_client_raw), with an explicit HWM.
    pub fn create_client_raw_with_hwm(
        &mut self,
        service_name: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<NodeServiceClientRaw> {
        let service_name = service_name.into();
        self.ensure_connected()?;
        let topology = Some(self.start_topology_guard("service_client", &service_name));
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let ctx = self.ensure_ws()?.client_context();
            return Ok(NodeServiceClientRaw {
                inner: ServiceClientInner::Ws(ctx),
                service_name,
                console_url: self.console_url_opt(),
                _topology: topology,
            });
        }
        let endpoint = self.options.service_frontend_endpoint()?;
        Ok(NodeServiceClientRaw {
            inner: ServiceClientInner::Zmq(BusServiceClient::with_context_hwm(
                self.context.zmq(),
                Some(&endpoint),
                hwm,
            )?),
            service_name,
            console_url: self.console_url_opt(),
            _topology: topology,
        })
    }

    fn client_rpc_hwm(&self) -> HighWaterMark {
        match &self.executor {
            Some(exec) => exec
                .lock()
                .map(|e| e.rpc_hwm())
                .unwrap_or(HighWaterMark::RPC),
            None => HighWaterMark::RPC,
        }
    }
}
