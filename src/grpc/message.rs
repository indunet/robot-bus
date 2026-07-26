//! `MessageGateway` — Subscribe (SUB→XPUB) and Publish (PUB→XSUB).

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use crate::errors::BusError;
use crate::message_bus::{Publisher, Subscriber};

use super::pb::message_gateway_server::MessageGateway;
use super::pb::{PublishResponse, SubscribeRequest, TopicMessage};

type SubscribeStream = Pin<Box<dyn Stream<Item = Result<TopicMessage, Status>> + Send + 'static>>;

#[derive(Clone)]
pub struct MessageGatewayService {
    message_xpub: String,
    message_xsub: String,
    /// Shared ZMQ PUB into the bus XSUB (lazy; reused across Publish RPCs).
    publisher: Arc<Mutex<Option<Publisher>>>,
}

impl MessageGatewayService {
    pub fn new(message_xpub: impl Into<String>, message_xsub: impl Into<String>) -> Self {
        Self {
            message_xpub: message_xpub.into(),
            message_xsub: message_xsub.into(),
            publisher: Arc::new(Mutex::new(None)),
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
}

fn bus_status(err: BusError) -> Status {
    match err {
        BusError::Timeout(msg) => Status::deadline_exceeded(msg),
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
        let xpub = self.message_xpub.clone();
        let (tx, rx) = mpsc::channel::<Result<TopicMessage, Status>>(64);

        thread::Builder::new()
            .name("grpc-zmq-sub".into())
            .spawn(move || {
                let sub = match Subscriber::new(Some(&xpub)) {
                    Ok(sub) => sub,
                    Err(err) => {
                        let _ = tx.blocking_send(Err(Status::unavailable(err.to_string())));
                        return;
                    }
                };
                if let Err(err) = sub.subscribe(&topic) {
                    let _ = tx.blocking_send(Err(Status::internal(err.to_string())));
                    return;
                }

                // Poll with a short timeout so drop of `tx` (client gone) can end the loop.
                loop {
                    match sub.receive(Some(Duration::from_millis(200))) {
                        Ok((msg_topic, payload)) => {
                            let msg = TopicMessage {
                                topic: msg_topic,
                                payload,
                            };
                            if tx.blocking_send(Ok(msg)).is_err() {
                                break;
                            }
                        }
                        Err(crate::errors::BusError::Timeout(_)) => {
                            if tx.is_closed() {
                                break;
                            }
                        }
                        Err(err) => {
                            let _ = tx.blocking_send(Err(Status::internal(err.to_string())));
                            break;
                        }
                    }
                }
            })
            .map_err(|err| Status::internal(format!("spawn subscriber thread: {err}")))?;

        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream) as Self::SubscribeStream))
    }

    async fn publish(
        &self,
        request: Request<TopicMessage>,
    ) -> Result<Response<PublishResponse>, Status> {
        let msg = request.into_inner();
        if msg.topic.is_empty() {
            return Err(Status::invalid_argument("topic is required"));
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
            Ok::<_, Status>(PublishResponse {})
        })
        .await
        .map_err(|err| Status::internal(format!("publish join: {err}")))??;

        Ok(Response::new(PublishResponse {}))
    }
}
