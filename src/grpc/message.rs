//! `MessageGateway` — Subscribe (SUB→XPUB) and Publish (PUB→XSUB).

use std::pin::Pin;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::errors::BusError;
use crate::message_bus::Publisher;

use super::pb::message_gateway_server::MessageGateway;
use super::pb::{PublishResponse, SubscribeRequest, TopicMessage};
use super::sub_demux::SubDemux;

type SubscribeStream = Pin<Box<dyn Stream<Item = Result<TopicMessage, Status>> + Send + 'static>>;

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

    fn ensure_publisher(publisher: &Mutex<Option<Publisher>>, xsub: &str) -> Result<(), Status> {
        let mut guard = publisher
            .lock()
            .map_err(|_| Status::internal("publisher mutex poisoned"))?;
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
    ) -> Result<mpsc::Receiver<Result<TopicMessage, Status>>, Status> {
        self.demux.open_subscribe(topic)
    }

    pub async fn publish_message(&self, msg: TopicMessage) -> Result<(), Status> {
        if msg.topic.is_empty() {
            return Err(Status::invalid_argument("topic is required"));
        }
        if let Err(err) = crate::console_topics::check_not_reserved(&msg.topic) {
            return Err(bus_status(err));
        }

        let publisher = Arc::clone(&self.publisher);
        let xsub = self.message_xsub.clone();
        let topic = msg.topic;
        let payload = msg.payload;

        tokio::task::spawn_blocking(move || {
            Self::ensure_publisher(&publisher, &xsub)?;
            let guard = publisher
                .lock()
                .map_err(|_| Status::internal("publisher mutex poisoned"))?;
            let pub_ = guard
                .as_ref()
                .ok_or_else(|| Status::internal("publisher missing after ensure"))?;
            pub_.publish(&topic, &payload).map_err(bus_status)?;
            Ok::<_, Status>(())
        })
        .await
        .map_err(|err| Status::internal(format!("publish join: {err}")))??;
        Ok(())
    }
}

fn bus_status(err: BusError) -> Status {
    match err {
        BusError::Timeout(msg) => Status::deadline_exceeded(msg),
        BusError::ReservedName { .. } => Status::invalid_argument(err.to_string()),
        other => Status::internal(other.to_string()),
    }
}

#[tonic::async_trait]
impl MessageGateway for MessageGatewayService {
    type SubscribeStream = SubscribeStream;

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let topic = request.into_inner().topic;
        let rx = self.open_subscribe(topic)?;
        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream) as Self::SubscribeStream))
    }

    async fn publish(
        &self,
        request: Request<TopicMessage>,
    ) -> Result<Response<PublishResponse>, Status> {
        self.publish_message(request.into_inner()).await?;
        Ok(Response::new(PublishResponse {}))
    }
}
