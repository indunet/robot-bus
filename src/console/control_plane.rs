//! Subscribe to `/robot_bus/topology/*` and `/robot_bus/topic_type/register`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use prost::Message;

use crate::console_topics;
use crate::message_bus::Subscriber;
use crate::robot_bus_interface::msg::v1::{
    TopicTypeRegister, TopologyRegister, TopologyUnregister,
};
use crate::worker_thread::WorkerThread;
use crate::zmq_helpers::HighWaterMark;

use super::state::ConsoleState;
use super::topology_registry::EndpointKind;

/// Background control-plane subscriber that updates console registries.
pub struct ControlPlaneHandle {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    workers: Vec<WorkerThread>,
}

impl ControlPlaneHandle {
    pub fn start(
        state: Arc<ConsoleState>,
        message_xpub: String,
        service_backend: String,
    ) -> anyhow::Result<Self> {
        let mut workers = Vec::with_capacity(console_topics::CONTROL_SUBSCRIBE.len());
        for &service_name in console_topics::CONTROL_SUBSCRIBE {
            let worker_state = Arc::clone(&state);
            let handler = Arc::new(move |payload: &[u8]| {
                handle_message(&worker_state, service_name, payload);
                Vec::new()
            });
            workers.push(WorkerThread::spawn_service(
                service_name,
                handler,
                service_backend.clone(),
            )?);
        }

        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);
        let handle = thread::Builder::new()
            .name("rbus-console-ctl".into())
            .spawn(move || {
                let sub = match Subscriber::with_hwm(Some(&message_xpub), HighWaterMark::STREAM) {
                    Ok(s) => s,
                    Err(err) => {
                        log::error!("console control-plane subscribe failed: {err}");
                        return;
                    }
                };
                for topic in console_topics::CONTROL_SUBSCRIBE {
                    if let Err(err) = sub.subscribe(topic) {
                        log::error!("console control-plane subscribe {topic}: {err}");
                        return;
                    }
                }
                while !stop_flag.load(Ordering::Relaxed) {
                    match sub.receive(Some(Duration::from_millis(200))) {
                        Ok((topic, payload)) => handle_message(&state, &topic, &payload),
                        Err(crate::errors::BusError::Timeout(_)) => {}
                        Err(err) => {
                            log::warn!("console control-plane receive: {err}");
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
            })?;
        Ok(Self {
            stop,
            handle: Some(handle),
            workers,
        })
    }

    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn stop(mut self) {
        self.request_stop();
        if let Some(handle) = self.handle.take() {
            join_with_timeout(handle, Duration::from_secs(2));
        }
        for worker in self.workers.drain(..) {
            worker.stop();
        }
    }
}

impl Drop for ControlPlaneHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            join_with_timeout(handle, Duration::from_secs(2));
        }
        for worker in self.workers.drain(..) {
            worker.stop();
        }
    }
}

fn join_with_timeout(handle: thread::JoinHandle<()>, limit: Duration) {
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let _ = handle.join();
        let _ = tx.send(());
    });
    match rx.recv_timeout(limit) {
        Ok(()) => {}
        Err(_) => log::warn!("console control-plane thread did not exit within {limit:?}"),
    }
}

fn handle_message(state: &ConsoleState, topic: &str, payload: &[u8]) {
    match topic {
        console_topics::TOPOLOGY_REGISTER => {
            let Ok(msg) = TopologyRegister::decode(payload) else {
                log::warn!("invalid TopologyRegister on {topic}");
                return;
            };
            let endpoint_id = msg.endpoint_id.trim();
            let node_name = msg.node_name.trim();
            let topic_name = msg.topic.trim();
            let Some(kind) = EndpointKind::parse(&msg.kind) else {
                return;
            };
            if endpoint_id.is_empty() || node_name.is_empty() || topic_name.is_empty() {
                return;
            }
            if kind == EndpointKind::Publisher && console_topics::is_reserved_name(topic_name) {
                log::warn!("reject topology publisher on reserved topic {topic_name}");
                return;
            }
            state
                .topology
                .register(endpoint_id, node_name, kind, topic_name);
        }
        console_topics::TOPOLOGY_UNREGISTER => {
            let Ok(msg) = TopologyUnregister::decode(payload) else {
                log::warn!("invalid TopologyUnregister on {topic}");
                return;
            };
            let endpoint_id = msg.endpoint_id.trim();
            if !endpoint_id.is_empty() {
                let _ = state.topology.unregister(endpoint_id);
            }
        }
        console_topics::TOPIC_TYPE_REGISTER => {
            let Ok(msg) = TopicTypeRegister::decode(payload) else {
                log::warn!("invalid TopicTypeRegister on {topic}");
                return;
            };
            let topic_name = msg.topic.trim();
            let type_name = msg.type_name.trim();
            if topic_name.is_empty() || type_name.is_empty() {
                return;
            }
            if console_topics::is_reserved_name(topic_name) {
                log::warn!("reject topic type register on reserved topic {topic_name}");
                return;
            }
            let previous = state.topic_types.register(topic_name, type_name);
            if previous.as_deref() != Some(type_name) {
                state.events.emit(
                    "INFO",
                    "topic-registry",
                    format!("registered {topic_name} -> {type_name}"),
                );
            }
        }
        _ => {}
    }
}
