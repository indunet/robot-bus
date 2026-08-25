//! Build protobuf snapshots from [`ConsoleState`] for `/robot_bus/*` topics.

use std::collections::HashMap;
use std::sync::Arc;

use prost::Message;

use crate::console_topics;
use crate::message_bus::Publisher;
use crate::robot_bus_interfaces::msg::v1::{
    ActionStats, ActionStatsList, BrokerStatus, ConsoleEvent, ServiceStats, ServiceStatsList,
    TopicStats, TopicStatsList, TopologyEdge, TopologyNode, TopologySnapshot,
};

use super::state::{ConsoleState, LogEntryDto, TopicRate, quantize_hz};
use super::topology_registry::{self, TopologyGraph};

pub fn encode_status(state: &ConsoleState) -> Vec<u8> {
    let rates = state.rates();
    let svc = state.service_rates();
    let act = state.action_rates();
    BrokerStatus {
        status: "ONLINE".into(),
        version: state.version.to_string(),
        uptime: state.uptime_secs(),
        pid: state.pid,
        grpc_addr: state.endpoints.ws.clone(),
        web_addr: state.endpoints.web.clone(),
        msg_bus_x_sub: state.endpoints.msg_xsub.clone(),
        msg_bus_x_pub: state.endpoints.msg_xpub.clone(),
        svc_fe: state.endpoints.svc_fe.clone(),
        svc_be: state.endpoints.svc_be.clone(),
        act_fe: state.endpoints.act_fe.clone(),
        act_be: state.endpoints.act_be.clone(),
        msg_per_sec: quantize_hz(rates.msg_per_sec),
        bytes_per_sec: rates.bytes_per_sec.round() as u64,
        svc_calls_per_sec: quantize_hz(svc.calls_per_sec),
        act_runs_per_sec: quantize_hz(act.runs_per_sec),
        total_messages: rates.total_msgs,
        total_errors: 0,
    }
    .encode_to_vec()
}

pub fn encode_topics(state: &ConsoleState) -> Vec<u8> {
    TopicStatsList {
        topics: merge_topic_stats(state),
    }
    .encode_to_vec()
}

pub fn encode_services(state: &ConsoleState) -> Vec<u8> {
    let rates = state.service_rates();
    ServiceStatsList {
        services: rates
            .services
            .into_iter()
            .map(|s| ServiceStats {
                name: s.name,
                calls: s.calls,
                calls_per_sec: quantize_hz(s.calls_per_sec),
                errors: s.errors,
                timeouts: 0,
                avg_latency_ms: s.avg_latency_ms,
                last_call_at: s.last_seen_unix_ms,
                workers: s.workers,
            })
            .collect(),
    }
    .encode_to_vec()
}

pub fn encode_actions(state: &ConsoleState) -> Vec<u8> {
    let rates = state.action_rates();
    ActionStatsList {
        actions: rates
            .actions
            .into_iter()
            .map(|a| ActionStats {
                name: a.name,
                runs: a.runs,
                runs_per_sec: quantize_hz(a.runs_per_sec),
                active: a.active,
                errors: a.errors,
                avg_duration_ms: a.avg_duration_ms,
                last_run_at: a.last_seen_unix_ms,
            })
            .collect(),
    }
    .encode_to_vec()
}

pub fn encode_topology(state: &ConsoleState) -> Vec<u8> {
    build_topology_proto(state).encode_to_vec()
}

pub fn encode_event(entry: &LogEntryDto) -> Vec<u8> {
    ConsoleEvent {
        id: entry.id.clone(),
        ts: entry.ts,
        level: entry.level.clone(),
        source: entry.source.clone(),
        message: entry.message.clone(),
    }
    .encode_to_vec()
}

fn merge_topic_stats(state: &ConsoleState) -> Vec<TopicStats> {
    let rates = state.rates();
    let type_map: HashMap<String, String> = state.topic_types.snapshot().into_iter().collect();
    let counts = state.topology.counts_by_topic();

    let mut by_name: HashMap<String, TopicRate> = rates
        .topics
        .into_iter()
        .map(|t| (t.name.clone(), t))
        .collect();

    for topic in type_map.keys().chain(counts.keys()) {
        by_name.entry(topic.clone()).or_insert_with(|| TopicRate {
            name: topic.clone(),
            total_msgs: 0,
            total_bytes: 0,
            last_seen_unix_ms: 0,
            msg_per_sec: 0.0,
            bytes_per_sec: 0.0,
        });
    }

    let mut names: Vec<_> = by_name.keys().cloned().collect();
    names.sort();

    names
        .into_iter()
        .filter_map(|name| {
            let t = by_name.remove(&name)?;
            let rate = quantize_hz(t.msg_per_sec);
            let (publishers, subscribers) = counts.get(&name).copied().unwrap_or((0, 0));
            Some(TopicStats {
                name: t.name,
                type_name: type_map.get(&name).cloned(),
                msg_per_sec: rate,
                bytes_per_sec: t.bytes_per_sec.round() as u64,
                last_seen: t.last_seen_unix_ms,
                total_msgs: t.total_msgs,
                sparkline: vec![rate; 20],
                subscribers,
                publishers,
            })
        })
        .collect()
}

pub(super) fn topology_graph(state: &ConsoleState) -> TopologyGraph {
    let type_map: HashMap<String, String> = state.topic_types.snapshot().into_iter().collect();
    let rates = state.rates();
    let topic_ops: HashMap<String, f64> = rates
        .topics
        .iter()
        .map(|t| (t.name.clone(), quantize_hz(t.msg_per_sec)))
        .collect();
    let service_ops: HashMap<String, f64> = state
        .service_rates()
        .services
        .iter()
        .map(|s| (s.name.clone(), quantize_hz(s.calls_per_sec)))
        .collect();
    let action_ops: HashMap<String, f64> = state
        .action_rates()
        .actions
        .iter()
        .map(|a| (a.name.clone(), quantize_hz(a.runs_per_sec)))
        .collect();
    topology_registry::build_topology_graph(
        &state.topology.snapshot(),
        &type_map,
        &topic_ops,
        &service_ops,
        &action_ops,
    )
}

fn build_topology_proto(state: &ConsoleState) -> TopologySnapshot {
    let graph = topology_graph(state);
    TopologySnapshot {
        nodes: graph
            .nodes
            .into_iter()
            .map(|n| TopologyNode {
                id: n.id,
                kind: n.kind.into(),
                label: n.label,
                type_name: n.type_name,
                msg_per_sec: n.msg_per_sec,
            })
            .collect(),
        edges: graph
            .edges
            .into_iter()
            .map(|e| TopologyEdge {
                id: e.id,
                source: e.source,
                target: e.target,
                kind: e.kind.into(),
                topic: e.topic,
            })
            .collect(),
    }
}

/// Background publisher: 1 Hz snapshots + live event fan-out on the message bus.
pub struct StatusPublisherHandle {
    stop: Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl StatusPublisherHandle {
    pub fn start(state: Arc<ConsoleState>, message_xsub: String) -> anyhow::Result<Self> {
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);
        let handle = std::thread::Builder::new()
            .name("rbus-console-pub".into())
            .spawn(move || {
                let publisher = match Publisher::new(Some(&message_xsub)) {
                    Ok(p) => p,
                    Err(err) => {
                        log::error!("console status publisher connect failed: {err}");
                        return;
                    }
                };
                if let Err(err) = publisher.set_send_timeout_ms(50) {
                    log::warn!("console status publisher sndtimeo: {err}");
                }
                let mut events_rx = state.events.subscribe();
                for entry in state.events.recent() {
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    let _ = publisher.publish(console_topics::EVENTS, &encode_event(&entry));
                }
                while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    let _ = publisher.publish(console_topics::STATUS, &encode_status(&state));
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                    let _ = publisher.publish(console_topics::TOPICS, &encode_topics(&state));
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                    let _ = publisher.publish(console_topics::SERVICES, &encode_services(&state));
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                    let _ = publisher.publish(console_topics::ACTIONS, &encode_actions(&state));
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                    let _ = publisher.publish(console_topics::TOPOLOGY, &encode_topology(&state));

                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
                    while std::time::Instant::now() < deadline
                        && !stop_flag.load(std::sync::atomic::Ordering::Relaxed)
                    {
                        match events_rx.try_recv() {
                            Ok(entry) => {
                                let _ = publisher
                                    .publish(console_topics::EVENTS, &encode_event(&entry));
                            }
                            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                                std::thread::sleep(std::time::Duration::from_millis(20));
                            }
                            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {}
                            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => return,
                        }
                    }
                }
            })?;
        Ok(Self {
            stop,
            handle: Some(handle),
        })
    }

    pub fn request_stop(&self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn stop(mut self) {
        self.request_stop();
        if let Some(handle) = self.handle.take() {
            join_with_timeout(handle, std::time::Duration::from_secs(2));
        }
    }
}

impl Drop for StatusPublisherHandle {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            join_with_timeout(handle, std::time::Duration::from_secs(2));
        }
    }
}

fn join_with_timeout(handle: std::thread::JoinHandle<()>, limit: std::time::Duration) {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = handle.join();
        let _ = tx.send(());
    });
    match rx.recv_timeout(limit) {
        Ok(()) => {}
        Err(_) => log::warn!("console background thread did not exit within {limit:?}"),
    }
}
