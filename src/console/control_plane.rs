//! Subscribe to `/robot_bus/topology/*` and `/robot_bus/topic_type/register`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use prost::Message;

use crate::console_topics;
use crate::message_bus::{Publisher, Subscriber};
use crate::robot_bus_interfaces::msg::v1::{
    TopicDemand, TopicTypeRegister, TopologyRegister, TopologyUnregister,
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
        message_xsub: String,
        service_backend: String,
    ) -> anyhow::Result<Self> {
        let demand_pub = match Publisher::new(Some(&message_xsub)) {
            Ok(p) => {
                if let Err(err) = p.set_send_timeout_ms(50) {
                    log::warn!("topic demand publisher sndtimeo: {err}");
                }
                Some(Arc::new(p))
            }
            Err(err) => {
                log::warn!("topic demand publisher connect failed: {err}");
                None
            }
        };

        let mut workers = Vec::with_capacity(console_topics::CONTROL_SUBSCRIBE.len());
        for &service_name in console_topics::CONTROL_SUBSCRIBE {
            let worker_state = Arc::clone(&state);
            let worker_pub = demand_pub.clone();
            let handler = Arc::new(move |payload: &[u8]| {
                handle_message(&worker_state, worker_pub.as_ref(), service_name, payload);
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
                        Ok((topic, payload)) => {
                            handle_message(&state, demand_pub.as_ref(), &topic, &payload)
                        }
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

fn publish_topic_demand(publisher: &Publisher, state: &ConsoleState, topic: &str) {
    let (_pubs, subscribers) = state.topology.counts_for_topic(topic);
    let payload = TopicDemand {
        topic: topic.to_string(),
        subscribers: subscribers as u32,
    }
    .encode_to_vec();
    if let Err(err) = publisher.publish(console_topics::TOPIC_DEMAND, &payload) {
        log::warn!("publish topic demand {topic}: {err}");
    }
}

fn handle_message(
    state: &ConsoleState,
    demand_pub: Option<&Arc<Publisher>>,
    topic: &str,
    payload: &[u8],
) {
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
            state
                .topology
                .register(endpoint_id, node_name, kind, topic_name);
            if kind == EndpointKind::Subscriber {
                if let Some(pub_) = demand_pub {
                    publish_topic_demand(pub_, state, topic_name);
                }
            }
        }
        console_topics::TOPOLOGY_UNREGISTER => {
            let Ok(msg) = TopologyUnregister::decode(payload) else {
                log::warn!("invalid TopologyUnregister on {topic}");
                return;
            };
            let endpoint_id = msg.endpoint_id.trim();
            if endpoint_id.is_empty() {
                return;
            }
            if let Some(rec) = state.topology.unregister(endpoint_id) {
                if rec.kind == EndpointKind::Subscriber {
                    if let Some(pub_) = demand_pub {
                        publish_topic_demand(pub_, state, &rec.topic);
                    }
                }
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
