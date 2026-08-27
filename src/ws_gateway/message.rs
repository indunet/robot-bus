//! `MessageGateway` — Subscribe (SUB→XPUB) and Publish (PUB→XSUB).

use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

use crate::errors::BusError;
use crate::message_bus::Publisher;

use super::rpc_status::RpcStatus;
use super::sub_demux::{BusMsg, SubDemux};

enum PubCmd {
    Publish {
        topic: String,
        payload: Vec<u8>,
        ack: tokio::sync::oneshot::Sender<Result<(), RpcStatus>>,
    },
    Shutdown,
}

struct PubWorker {
    tx: Mutex<Option<std::sync::mpsc::Sender<PubCmd>>>,
    xsub: String,
}

impl PubWorker {
    fn new(xsub: String) -> Self {
        Self {
            tx: Mutex::new(None),
            xsub,
        }
    }

    fn ensure_started(&self) -> Result<std::sync::mpsc::Sender<PubCmd>, RpcStatus> {
        let mut guard = self
            .tx
            .lock()
            .map_err(|_| RpcStatus::internal("pub worker mutex poisoned"))?;
        if let Some(tx) = guard.as_ref() {
            return Ok(tx.clone());
        }
        let (tx, rx) = std::sync::mpsc::channel::<PubCmd>();
        let xsub = self.xsub.clone();
        std::thread::Builder::new()
            .name("ws-zmq-pub".into())
            .spawn(move || pub_loop(xsub, rx))
            .map_err(|err| RpcStatus::internal(format!("spawn pub worker: {err}")))?;
        *guard = Some(tx.clone());
        Ok(tx)
    }
}

impl Drop for PubWorker {
    fn drop(&mut self) {
        if let Ok(guard) = self.tx.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(PubCmd::Shutdown);
            }
        }
    }
}

fn pub_loop(xsub: String, rx: std::sync::mpsc::Receiver<PubCmd>) {
    let publisher = match Publisher::new(Some(&xsub)) {
        Ok(p) => p,
        Err(err) => {
            while let Ok(cmd) = rx.recv() {
                if let PubCmd::Publish { ack, .. } = cmd {
                    let _ = ack.send(Err(RpcStatus::unavailable(err.to_string())));
                }
            }
            return;
        }
    };
    while let Ok(cmd) = rx.recv() {
        match cmd {
            PubCmd::Shutdown => return,
            PubCmd::Publish {
                topic,
                payload,
                ack,
            } => {
                let result = publisher.publish(&topic, &payload).map_err(bus_status);
                let _ = ack.send(result);
            }
        }
    }
}

#[derive(Clone)]
pub struct MessageGatewayService {
    pub_worker: Arc<PubWorker>,
    demux: SubDemux,
}

impl MessageGatewayService {
    pub fn new(message_xpub: impl Into<String>, message_xsub: impl Into<String>) -> Self {
        let message_xpub = message_xpub.into();
        let message_xsub = message_xsub.into();
        Self {
            pub_worker: Arc::new(PubWorker::new(message_xsub)),
            demux: SubDemux::new(message_xpub),
        }
    }

    /// Start a topic subscription; returns an mpsc that closes when the sender is dropped.
    pub fn open_subscribe(
        &self,
        topic: String,
        qos_depth: i32,
    ) -> Result<mpsc::Receiver<Result<BusMsg, RpcStatus>>, RpcStatus> {
        self.demux.open_subscribe(topic, qos_depth)
    }

    pub async fn publish_message(&self, topic: String, payload: Vec<u8>) -> Result<(), RpcStatus> {
        if topic.is_empty() {
            return Err(RpcStatus::invalid_argument("topic is required"));
        }
        let tx = self.pub_worker.ensure_started()?;
        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.send(PubCmd::Publish {
            topic,
            payload,
            ack: ack_tx,
        })
        .map_err(|_| RpcStatus::internal("pub worker channel closed"))?;
        ack_rx
            .await
            .map_err(|_| RpcStatus::internal("pub worker ack dropped"))?
    }
}

fn bus_status(err: BusError) -> RpcStatus {
    match err {
        BusError::Timeout(msg) => RpcStatus::deadline_exceeded(msg),
        other => RpcStatus::internal(other.to_string()),
    }
}
