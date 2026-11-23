use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use prost::{Message, Name};

use crate::errors::{BusError, Result};
use crate::message_bus::Publisher as BusPublisher;
use crate::runtime::callback_group::CallbackGroup;
use crate::runtime::qos::QosProfile;
use crate::runtime::registrations::MessageCallback;
use crate::runtime::timers::{SubscriptionHandle, TimerCallback, TimerHandle};
use crate::zmq_helpers::HighWaterMark;

use super::super::Node;
use super::super::publishers::{TopicPublisher, TopicPublisherBackend, TopicPublisherRaw};

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
            let _ = hwm; // shared server PUB; KeepLast is not per-client on WS publish
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
    /// socket. On WebSocket, depth sizes that topic's server→client queue.
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
    /// server→client queue.
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
}
