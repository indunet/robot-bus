//! `MessageGateway` — server-streaming Subscribe bridged to a ZMQ SUB socket.

use std::pin::Pin;
use std::thread;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use crate::message_bus::Subscriber;

use super::pb::message_gateway_server::MessageGateway;
use super::pb::{SubscribeRequest, TopicMessage};

type SubscribeStream = Pin<Box<dyn Stream<Item = Result<TopicMessage, Status>> + Send + 'static>>;

#[derive(Clone, Debug)]
pub struct MessageGatewayService {
    message_xpub: String,
}

impl MessageGatewayService {
    pub fn new(message_xpub: impl Into<String>) -> Self {
        Self {
            message_xpub: message_xpub.into(),
        }
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
}
