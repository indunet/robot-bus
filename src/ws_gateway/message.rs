//! `MessageGateway` — Subscribe (SUB→XPUB) and Publish (PUB→XSUB).

use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

use crate::errors::BusError;
use crate::message_bus::Publisher;

use super::pb::{PublishResponse, SubscribeRequest, TopicMessage};
use super::rpc_status::RpcStatus;
use super::sub_demux::SubDemux;

#[derive(Clone)]
pub struct MessageGatewayService {
    message_xsub: String,
    /// Shared ZMQ PUB into the bus XSUB (lazy; reused across Publish RPCs).
    publisher: Arc<Mutex<Option<Publisher>>>,
    demux: SubDemux,
}

impl MessageGatewayService {
    pub fn new(message_xpub: impl Into<String>, message_xsub: impl Into<String>) -> Self {
        let message_xpub = message_xpub.into();
        Self {
            message_xsub: message_xsub.into(),
            publisher: Arc::new(Mutex::new(None)),
            demux: SubDemux::new(message_xpub),
        }
    }

    fn ensure_publisher(publisher: &Mutex<Option<Publisher>>, xsub: &str) -> Result<(), RpcStatus> {
        let mut guard = publisher
            .lock()
            .map_err(|_| RpcStatus::internal("publisher mutex poisoned"))?;
        if guard.is_none() {
            let pub_ = Publisher::new(Some(xsub)).map_err(bus_status)?;
            *guard = Some(pub_);
        }
        Ok(())
    }

    /// Start a topic subscription; returns an mpsc that closes when the sender is dropped.
    pub fn open_subscribe(
        &self,
        topic: String,
        qos_depth: i32,
    ) -> Result<mpsc::Receiver<Result<TopicMessage, RpcStatus>>, RpcStatus> {
        self.demux.open_subscribe(topic, qos_depth)
    }

    pub async fn publish_message(&self, msg: TopicMessage) -> Result<(), RpcStatus> {
        if msg.topic.is_empty() {
            return Err(RpcStatus::invalid_argument("topic is required"));
        }

        let publisher = Arc::clone(&self.publisher);
        let xsub = self.message_xsub.clone();
        let topic = msg.topic;
        let payload = msg.payload;

        tokio::task::spawn_blocking(move || {
            Self::ensure_publisher(&publisher, &xsub)?;
            let guard = publisher
                .lock()
                .map_err(|_| RpcStatus::internal("publisher mutex poisoned"))?;
            let pub_ = guard
                .as_ref()
                .ok_or_else(|| RpcStatus::internal("publisher missing after ensure"))?;
            pub_.publish(&topic, &payload).map_err(bus_status)?;
            Ok::<_, RpcStatus>(())
        })
        .await
        .map_err(|err| RpcStatus::internal(format!("publish join: {err}")))??;
        Ok(())
    }

    /// Decode + publish; used by the WebSocket gateway.
    pub async fn handle_publish(&self, payload: &[u8]) -> Result<PublishResponse, RpcStatus> {
        use prost::Message as ProstMessage;
        let msg = TopicMessage::decode(payload)
            .map_err(|err| RpcStatus::invalid_argument(format!("decode TopicMessage: {err}")))?;
        self.publish_message(msg).await?;
        Ok(PublishResponse {})
    }

    pub fn handle_subscribe_request(
        &self,
        payload: &[u8],
    ) -> Result<(String, mpsc::Receiver<Result<TopicMessage, RpcStatus>>), RpcStatus> {
        use prost::Message as ProstMessage;
        let req = SubscribeRequest::decode(payload).map_err(|err| {
            RpcStatus::invalid_argument(format!("decode SubscribeRequest: {err}"))
        })?;
        let rx = self.open_subscribe(req.topic.clone(), req.qos_depth)?;
        Ok((req.topic, rx))
    }
}

fn bus_status(err: BusError) -> RpcStatus {
    match err {
        BusError::Timeout(msg) => RpcStatus::deadline_exceeded(msg),
        other => RpcStatus::internal(other.to_string()),
    }
}
