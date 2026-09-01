use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use prost::{Message, Name};

use crate::errors::{BusError, Result};
use crate::message_bus::Publisher as BusPublisher;
use crate::runtime::callback_group::CallbackGroup;
use crate::runtime::qos::QosProfile;
use crate::runtime::queues::ActionMessageCallback;
use crate::runtime::registrations::{
    ActionGoalHandler, ActionGoalLiveHandler, MessageCallback, ServiceHandler,
};
use crate::runtime::timers::{SubscriptionHandle, TimerCallback, TimerHandle};
use crate::runtime::topology_register::TopologyEndpointGuard;
use crate::service_bus::ServiceClient as BusServiceClient;
use crate::typed::{Action, ActionOutcome, Service};
use crate::zmq_helpers::HighWaterMark;

use super::action_clients::{
    ActionClientInner, NodeActionClient, NodeActionClientRaw, NodeActionServer,
};
use super::options::ws_mode_unsupported;
use super::publishers::{TopicPublisher, TopicPublisherBackend, TopicPublisherRaw};
use super::service_clients::{
    NodeService, NodeServiceClient, NodeServiceClientRaw, ServiceClientInner,
};
use super::Node;

impl Node {
    /// Create a typed topic publisher (ROS 2 `create_publisher`).
    ///
    /// Uses the node's stream HWM default on first PUB connect. Prefer
    /// [`create_publisher_with_qos`](Self::create_publisher_with_qos) to set KeepLast depth.
    ///
    /// Multiple publishers on the same node share one bus PUB socket.
    /// Best-effort registers `topic → M::full_name()` with the broker console.
    pub fn create_publisher<M: Message + Name + Default>(
        &mut self,
        topic: impl Into<String>,
    ) -> Result<TopicPublisher<M>> {
        let topic = topic.into();
        let pub_ = TopicPublisher {
            inner: self.create_publisher_raw(topic.clone())?,
            _marker: PhantomData,
        };
        self.remember_topic_type(&topic, &M::full_name());
        Ok(pub_)
    }

    /// Create a typed topic publisher with topic QoS (KeepLast depth → HWM).
    ///
    /// Topic reliability is always best-effort.
    pub fn create_publisher_with_qos<M: Message + Name + Default>(
        &mut self,
        topic: impl Into<String>,
        qos: QosProfile,
    ) -> Result<TopicPublisher<M>> {
        let topic = topic.into();
        let pub_ = TopicPublisher {
            inner: self.create_publisher_raw_with_qos(topic.clone(), qos)?,
            _marker: PhantomData,
        };
        self.remember_topic_type(&topic, &M::full_name());
        Ok(pub_)
    }

    /// Create a raw-bytes topic publisher (inherits node stream HWM).
    pub fn create_publisher_raw(&mut self, topic: impl Into<String>) -> Result<TopicPublisherRaw> {
        self.create_publisher_raw_with_hwm(topic, None)
    }

    /// Create a raw-bytes topic publisher with topic QoS.
    pub fn create_publisher_raw_with_qos(
        &mut self,
        topic: impl Into<String>,
        qos: QosProfile,
    ) -> Result<TopicPublisherRaw> {
        self.create_publisher_raw_with_hwm(topic, Some(qos.to_hwm()))
    }

    /// Like [`create_publisher_raw`](Self::create_publisher_raw), optionally setting HWM
    /// on first socket connect. Prefer [`create_publisher_raw_with_qos`] for topic depth.
    pub fn create_publisher_raw_with_hwm(
        &mut self,
        topic: impl Into<String>,
        hwm: Option<HighWaterMark>,
    ) -> Result<TopicPublisherRaw> {
        let topic = topic.into();
        self.ensure_connected()?;
        let topology = Some(self.start_topology_guard("publisher", &topic));
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let _ = hwm; // shared gateway PUB; KeepLast is not per-client on WS publish
            let grpc = self.ensure_ws()?;
            return Ok(TopicPublisherRaw {
                backend: TopicPublisherBackend::Ws(grpc.client_context()),
                topic,
                _topology: topology,
            });
        }
        self.ensure_bus_publisher(hwm)?;
        Ok(TopicPublisherRaw {
            backend: TopicPublisherBackend::Zmq(Arc::clone(
                self.publisher.as_ref().expect("publisher just ensured"),
            )),
            topic,
            _topology: topology,
        })
    }

    /// Like [`create_publisher`](Self::create_publisher), setting HWM on first socket connect.
    /// Prefer [`create_publisher_with_qos`](Self::create_publisher_with_qos) for topic depth.
    pub fn create_publisher_with_hwm<M: Message + Name + Default>(
        &mut self,
        topic: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<TopicPublisher<M>> {
        let topic = topic.into();
        let pub_ = TopicPublisher {
            inner: self.create_publisher_raw_with_hwm(topic.clone(), Some(hwm))?,
            _marker: PhantomData,
        };
        self.remember_topic_type(&topic, &M::full_name());
        Ok(pub_)
    }

    fn ensure_bus_publisher(&mut self, hwm: Option<HighWaterMark>) -> Result<()> {
        if let Some(pub_) = &self.publisher {
            if let Some(hwm) = hwm {
                pub_.set_high_water_mark(hwm)?;
            }
            return Ok(());
        }
        self.ensure_connected()?;
        let hwm = match hwm {
            Some(h) => h,
            None => match &self.executor {
                Some(exec) => exec.lock()?.stream_hwm(),
                None => HighWaterMark::STREAM,
            },
        };
        let endpoint = self.options.message_xsub_endpoint()?;
        self.publisher = Some(Arc::new(BusPublisher::with_context_hwm(
            self.context.zmq(),
            Some(&endpoint),
            hwm,
        )?));
        Ok(())
    }

    /// Current shared publisher HWM, if any publisher was created.
    pub fn publisher_hwm(&self) -> Result<Option<HighWaterMark>> {
        match &self.publisher {
            Some(pub_) => Ok(Some(pub_.high_water_mark()?)),
            None => Ok(None),
        }
    }

    /// Update shared publisher HWM (error if no publisher created yet).
    pub fn set_publisher_hwm(&self, hwm: HighWaterMark) -> Result<()> {
        let Some(pub_) = self.publisher.as_ref() else {
            return Err(BusError::Protocol(
                "create_publisher() before set_publisher_hwm()".into(),
            ));
        };
        pub_.set_high_water_mark(hwm)
    }

    pub fn stream_hwm(&mut self) -> Result<HighWaterMark> {
        if self.options.is_ws() {
            return Ok(HighWaterMark::STREAM);
        }
        Ok(self.lock_executor()?.stream_hwm())
    }

    pub fn set_stream_hwm(&mut self, hwm: HighWaterMark) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "set_stream_hwm is not available in WebSocket RPC node mode".into(),
            ));
        }
        self.lock_executor()?.set_stream_hwm(hwm)
    }

    pub fn rpc_hwm(&mut self) -> Result<HighWaterMark> {
        if self.options.is_ws() {
            return Ok(HighWaterMark::RPC);
        }
        Ok(self.lock_executor()?.rpc_hwm())
    }

    pub fn set_rpc_hwm(&mut self, hwm: HighWaterMark) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "set_rpc_hwm is not available in WebSocket RPC node mode".into(),
            ));
        }
        self.lock_executor()?.set_rpc_hwm(hwm)
    }

    pub fn action_hwm(&mut self) -> Result<HighWaterMark> {
        if self.options.is_ws() {
            return Ok(HighWaterMark::ACTION);
        }
        Ok(self.lock_executor()?.action_hwm())
    }

    pub fn set_action_hwm(&mut self, hwm: HighWaterMark) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "set_action_hwm is not available in WebSocket RPC node mode".into(),
            ));
        }
        self.lock_executor()?.set_action_hwm(hwm)
    }

    /// Subscribe with a protobuf-typed callback (ROS 2 `create_subscription`).
    ///
    /// Does not change the node's stream HWM. Prefer
    /// [`create_subscription_with_qos`](Self::create_subscription_with_qos) to set
    /// KeepLast depth on the shared SUB socket.
    ///
    /// Decode failures are skipped (logged). `callback_group: None` uses the
    /// node's default mutually exclusive group.
    /// Best-effort registers `topic → M::full_name()` with the broker console.
    pub fn create_subscription<M, F>(
        &mut self,
        topic: &str,
        callback: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<SubscriptionHandle>
    where
        M: Message + Name + Default + 'static,
        F: Fn(M) + Send + Sync + 'static,
    {
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        let topic_name = topic.to_string();
        let cb: MessageCallback = Arc::new(move |payload| match M::decode(payload) {
            Ok(msg) => callback(msg),
            Err(err) => log::warn!("typed subscription decode failed on {topic_name}: {err}"),
        });
        let handle = self.create_subscription_raw(topic, cb, Some(&group))?;
        self.remember_topic_type(topic, &M::full_name());
        Ok(handle)
    }

    /// Subscribe with topic QoS (KeepLast depth).
    ///
    /// Topic reliability is always best-effort. On ZMQ, multiple subscriptions on
    /// one node share one SUB socket — the last explicit QoS depth wins for that
    /// socket. On WebSocket, depth sizes that topic's gateway→client queue.
    pub fn create_subscription_with_qos<M, F>(
        &mut self,
        topic: &str,
        qos: QosProfile,
        callback: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<SubscriptionHandle>
    where
        M: Message + Name + Default + 'static,
        F: Fn(M) + Send + Sync + 'static,
    {
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        let topic_name = topic.to_string();
        let cb: MessageCallback = Arc::new(move |payload| match M::decode(payload) {
            Ok(msg) => callback(msg),
            Err(err) => log::warn!("typed subscription decode failed on {topic_name}: {err}"),
        });
        let handle = self.create_subscription_raw_with_qos(topic, qos, cb, Some(&group))?;
        self.remember_topic_type(topic, &M::full_name());
        Ok(handle)
    }

    /// Subscribe with a raw-bytes callback (does not change stream HWM).
    pub fn create_subscription_raw(
        &mut self,
        topic: &str,
        callback: MessageCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<SubscriptionHandle> {
        self.create_subscription_raw_inner(topic, None, callback, callback_group)
    }

    /// Subscribe with a raw-bytes callback and topic QoS (KeepLast depth).
    ///
    /// ZMQ: applies to the shared SUB socket HWM. WebSocket: sizes this topic's
    /// gateway→client queue.
    pub fn create_subscription_raw_with_qos(
        &mut self,
        topic: &str,
        qos: QosProfile,
        callback: MessageCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<SubscriptionHandle> {
        self.create_subscription_raw_inner(topic, Some(qos), callback, callback_group)
    }

    fn create_subscription_raw_inner(
        &mut self,
        topic: &str,
        qos: Option<QosProfile>,
        callback: MessageCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<SubscriptionHandle> {
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        self.ensure_connected()?;
        let topology = self.start_topology_guard("subscriber", topic);
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let handle = self.ensure_ws()?.subscribe(topic, callback, group, qos)?;
            self.topology_subscriptions.insert(handle.id(), topology);
            return Ok(handle);
        }
        if let Some(qos) = qos {
            // Apply before connect so the first SUB socket gets the depth; also
            // updates an already-connected shared SUB in place.
            self.set_stream_hwm(qos.to_hwm())?;
        }
        self.ensure_subscriber()?;
        let handle = self.lock_executor()?.subscribe(topic, callback, group)?;
        self.topology_subscriptions.insert(handle.id(), topology);
        Ok(handle)
    }

    /// Destroy a subscription created by [`create_subscription`](Self::create_subscription)
    /// / raw variants. Same `start()` constraint as [`cancel_timer`](Self::cancel_timer).
    pub fn destroy_subscription(&mut self, handle: SubscriptionHandle) -> Result<()> {
        self.topology_subscriptions.remove(&handle.id());
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return self.ensure_ws()?.destroy_subscription(handle);
        }
        self.lock_executor()?.destroy_subscription(handle)
    }

    fn ensure_subscriber(&mut self) -> Result<()> {
        if self.options.is_ws() {
            return Err(BusError::Protocol(
                "internal: ensure_subscriber on gRPC node".into(),
            ));
        }
        self.ensure_connected()?;
        if !self.subscriber_connected {
            let endpoint = self.options.message_xpub_endpoint()?;
            self.lock_executor()?.connect_subscriber(Some(&endpoint))?;
            self.subscriber_connected = true;
        }
        Ok(())
    }

    /// Periodic timer (ROS 2 `create_timer`).
    ///
    /// `callback_group: None` → default mutually exclusive group.
    pub fn create_timer(
        &mut self,
        period: Duration,
        callback: TimerCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<TimerHandle> {
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return self.ensure_ws()?.create_timer(period, callback, group);
        }
        self.lock_executor()?.create_timer(period, callback, group)
    }

    /// Alias for [`create_timer`](Self::create_timer) (ROS 2 `create_wall_timer`).
    pub fn create_wall_timer(
        &mut self,
        period: Duration,
        callback: TimerCallback,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<TimerHandle> {
        self.create_timer(period, callback, callback_group)
    }

    pub fn cancel_timer(&mut self, handle: TimerHandle) -> Result<()> {
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            return self.ensure_ws()?.cancel_timer(handle);
        }
        self.lock_executor()?.cancel_timer(handle)
    }

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

    /// Register a typed action server (ROS 2–style `create_action_server`).
    pub fn create_action_server<A, F>(
        &mut self,
        action_name: &str,
        handler: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer>
    where
        A: Action,
        F: Fn(A::Goal) -> ActionOutcome<A> + Send + Sync + 'static,
    {
        let cb: ActionGoalHandler = Arc::new(move |body| match A::Goal::decode(body) {
            Ok(goal) => {
                let outcome = handler(goal);
                let mut replies = Vec::with_capacity(outcome.feedbacks.len() + 1);
                for fb in outcome.feedbacks {
                    replies.push(("FEEDBACK".into(), fb.encode_to_vec()));
                }
                replies.push(("RESULT".into(), outcome.result.encode_to_vec()));
                replies
            }
            Err(err) => {
                log::warn!("typed action goal decode failed: {err}");
                vec![("RESULT".into(), Vec::new())]
            }
        });
        self.create_action_server_raw(action_name, cb, callback_group)
    }

    /// Register a typed action server with KeepLast depth → DEALER HWM.
    pub fn create_action_server_with_qos<A, F>(
        &mut self,
        action_name: &str,
        qos: QosProfile,
        handler: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer>
    where
        A: Action,
        F: Fn(A::Goal) -> ActionOutcome<A> + Send + Sync + 'static,
    {
        let cb: ActionGoalHandler = Arc::new(move |body| match A::Goal::decode(body) {
            Ok(goal) => {
                let outcome = handler(goal);
                let mut replies = Vec::with_capacity(outcome.feedbacks.len() + 1);
                for fb in outcome.feedbacks {
                    replies.push(("FEEDBACK".into(), fb.encode_to_vec()));
                }
                replies.push(("RESULT".into(), outcome.result.encode_to_vec()));
                replies
            }
            Err(err) => {
                log::warn!("typed action goal decode failed: {err}");
                vec![("RESULT".into(), Vec::new())]
            }
        });
        self.create_action_server_raw_with_qos(action_name, qos, cb, callback_group)
    }

    /// Register a raw-bytes action server.
    pub fn create_action_server_raw(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        self.create_action_server_raw_inner(action_name, handler, callback_group, None)
    }

    /// Register a raw-bytes action server with KeepLast depth → DEALER HWM.
    pub fn create_action_server_raw_with_qos(
        &mut self,
        action_name: &str,
        qos: QosProfile,
        handler: ActionGoalHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        self.create_action_server_raw_inner(
            action_name,
            handler,
            callback_group,
            Some(qos.to_hwm()),
        )
    }

    fn create_action_server_raw_inner(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        callback_group: Option<&CallbackGroup>,
        hwm: Option<HighWaterMark>,
    ) -> Result<NodeActionServer> {
        self.finish_action_server(
            action_name,
            callback_group,
            hwm,
            |exec, group, endpoint, hwm| {
                exec.register_action(action_name, handler, group, Some(endpoint), None, hwm)
            },
        )
    }

    /// Register a live action server: handler may publish FEEDBACK and poll CANCEL.
    pub fn create_action_server_raw_live(
        &mut self,
        action_name: &str,
        handler: ActionGoalLiveHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        self.create_action_server_raw_live_with_qos_inner(action_name, handler, callback_group, None)
    }

    /// Live action server with KeepLast depth → DEALER HWM.
    pub fn create_action_server_raw_live_with_qos(
        &mut self,
        action_name: &str,
        qos: QosProfile,
        handler: ActionGoalLiveHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        self.create_action_server_raw_live_with_qos_inner(
            action_name,
            handler,
            callback_group,
            Some(qos.to_hwm()),
        )
    }

    fn create_action_server_raw_live_with_qos_inner(
        &mut self,
        action_name: &str,
        handler: ActionGoalLiveHandler,
        callback_group: Option<&CallbackGroup>,
        hwm: Option<HighWaterMark>,
    ) -> Result<NodeActionServer> {
        self.finish_action_server(
            action_name,
            callback_group,
            hwm,
            |exec, group, endpoint, hwm| {
                exec.register_action_live(action_name, handler, group, Some(endpoint), None, hwm)
            },
        )
    }

    fn finish_action_server(
        &mut self,
        action_name: &str,
        callback_group: Option<&CallbackGroup>,
        hwm: Option<HighWaterMark>,
        register: impl FnOnce(
            &mut crate::runtime::Executor,
            CallbackGroup,
            &str,
            Option<HighWaterMark>,
        ) -> Result<u64>,
    ) -> Result<NodeActionServer> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("create_action_server"));
        }
        self.ensure_connected()?;
        let endpoint = self.options.action_backend_endpoint()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        let id = register(&mut *self.lock_executor()?, group, &endpoint, hwm)?;
        let topology = self.start_topology_guard("action_server", action_name);
        self.topology_actions.insert(id, topology);
        Ok(NodeActionServer {
            id,
            action_name: action_name.to_string(),
        })
    }

    /// Destroy an action server created by [`create_action_server`](Self::create_action_server).
    /// Same `start()` constraint as [`cancel_timer`](Self::cancel_timer).
    pub fn destroy_action_server(&mut self, handle: &NodeActionServer) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("destroy_action_server"));
        }
        self.topology_actions.remove(&handle.id);
        self.lock_executor()?.destroy_action_server(handle.id)
    }

    /// Create a typed action client (ROS 2–style `create_action_client`).
    pub fn create_action_client<A: Action>(
        &mut self,
        action_name: impl Into<String>,
    ) -> Result<NodeActionClient<A>> {
        Ok(NodeActionClient {
            inner: self.create_action_client_raw(action_name)?,
            _marker: PhantomData,
        })
    }

    /// Create a typed action client with KeepLast depth → DEALER HWM.
    pub fn create_action_client_with_qos<A: Action>(
        &mut self,
        action_name: impl Into<String>,
        qos: QosProfile,
    ) -> Result<NodeActionClient<A>> {
        Ok(NodeActionClient {
            inner: self.create_action_client_raw_with_qos(action_name, qos)?,
            _marker: PhantomData,
        })
    }

    /// Create a raw-bytes action client bound to `action_name`.
    pub fn create_action_client_raw(
        &mut self,
        action_name: impl Into<String>,
    ) -> Result<NodeActionClientRaw> {
        self.create_action_client_raw_with_hwm(action_name, self.client_action_hwm())
    }

    /// Create a raw-bytes action client with KeepLast depth → DEALER HWM.
    pub fn create_action_client_raw_with_qos(
        &mut self,
        action_name: impl Into<String>,
        qos: QosProfile,
    ) -> Result<NodeActionClientRaw> {
        self.create_action_client_raw_with_hwm(action_name, qos.to_hwm())
    }

    /// Like [`create_action_client_raw`](Self::create_action_client_raw), with an explicit HWM.
    pub fn create_action_client_raw_with_hwm(
        &mut self,
        action_name: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<NodeActionClientRaw> {
        let action_name = action_name.into();
        self.ensure_connected()?;
        let topology = Some(self.start_topology_guard("action_client", &action_name));
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let ctx = self.ensure_ws()?.client_context();
            return Ok(NodeActionClientRaw {
                inner: ActionClientInner::Ws(ctx),
                action_name,
                console_url: self.console_url_opt(),
                _topology: topology,
            });
        }
        let endpoint = self.options.action_frontend_endpoint()?;
        Ok(NodeActionClientRaw {
            inner: ActionClientInner::Zmq {
                context: self.context.clone_zmq(),
                endpoint,
                hwm: Mutex::new(hwm),
            },
            action_name,
            console_url: self.console_url_opt(),
            _topology: topology,
        })
    }

    fn client_action_hwm(&self) -> HighWaterMark {
        match &self.executor {
            Some(exec) => exec
                .lock()
                .map(|e| e.action_hwm())
                .unwrap_or(HighWaterMark::ACTION),
            None => HighWaterMark::ACTION,
        }
    }

    fn start_topology_guard(&self, kind: &str, name: &str) -> Arc<TopologyEndpointGuard> {
        let guard = TopologyEndpointGuard::start(
            self.options.service_frontend.as_deref(),
            &self.options.host,
            &self.options.transport,
            &self.name,
            kind,
            name,
        );
        self.control_plane.remember_topology(&guard);
        guard
    }

    /// Connect the executor-owned action client used by callback-style [`send_goal`](Self::send_goal).
    pub fn connect_action_client(&mut self) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported(
                "connect_action_client (use create_action_client)",
            ));
        }
        let endpoint = self.options.action_frontend_endpoint()?;
        self.lock_executor()?.connect_action_client(Some(&endpoint))
    }

    /// Submit a goal via the executor (callback receives FEEDBACK / RESULT). Prefer
    /// [`create_action_client`](Self::create_action_client) for a ROS 2–style sync handle.
    pub fn send_goal(
        &mut self,
        action_name: &str,
        body: &[u8],
        callback: ActionMessageCallback,
        goal_id: Option<&str>,
    ) -> Result<String> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("send_goal (use create_action_client)"));
        }
        self.lock_executor()?
            .send_goal(action_name, body, callback, goal_id)
    }

    pub fn cancel_goal(&mut self, action_name: &str, goal_id: &str, body: &[u8]) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported(
                "cancel_goal (use create_action_client)",
            ));
        }
        self.lock_executor()?
            .cancel_goal(action_name, goal_id, body)
    }
}
