//! REST / SSE monitoring endpoints for the embedded console.

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use serde::Serialize;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use super::state::{ConsoleState, LogEntryDto, TopicRate};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusResponse {
    status: &'static str,
    version: String,
    uptime: u64,
    pid: u32,
    grpc_addr: String,
    web_addr: String,
    msg_bus_x_sub: String,
    msg_bus_x_pub: String,
    #[serde(rename = "svcFE")]
    svc_fe: String,
    #[serde(rename = "svcBE")]
    svc_be: String,
    #[serde(rename = "actFE")]
    act_fe: String,
    #[serde(rename = "actBE")]
    act_be: String,
    msg_per_sec: u64,
    bytes_per_sec: u64,
    svc_calls_per_sec: u64,
    act_runs_per_sec: u64,
    total_messages: u64,
    total_errors: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TopicResponse {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_name: Option<String>,
    msg_per_sec: u64,
    bytes_per_sec: u64,
    last_seen: u64,
    total_msgs: u64,
    sparkline: Vec<u64>,
    subscribers: u64,
    publishers: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TopicsEnvelope {
    topics: Vec<TopicResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TopologyNodeDto {
    id: String,
    kind: &'static str,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_per_sec: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TopologyEdgeDto {
    id: String,
    source: String,
    target: String,
    kind: &'static str,
    topic: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TopologyResponse {
    nodes: Vec<TopologyNodeDto>,
    edges: Vec<TopologyEdgeDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ServiceResponse {
    name: String,
    calls: u64,
    calls_per_sec: u64,
    errors: u64,
    timeouts: u64,
    avg_latency_ms: u64,
    last_call_at: u64,
    workers: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ServicesEnvelope {
    services: Vec<ServiceResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionResponse {
    name: String,
    runs: u64,
    runs_per_sec: u64,
    active: u64,
    workers: u64,
    errors: u64,
    avg_duration_ms: u64,
    last_run_at: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionsEnvelope {
    actions: Vec<ActionResponse>,
}

pub async fn status(Extension(state): Extension<Arc<ConsoleState>>) -> impl IntoResponse {
    let rates = state.rates();
    let svc = state.service_rates();
    let act = state.action_rates();
    Json(StatusResponse {
        status: "ONLINE",
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
        msg_per_sec: rates.msg_per_sec.round() as u64,
        bytes_per_sec: rates.bytes_per_sec.round() as u64,
        svc_calls_per_sec: svc.calls_per_sec.round() as u64,
        act_runs_per_sec: act.runs_per_sec.round() as u64,
        total_messages: rates.total_msgs,
        total_errors: 0,
    })
}

pub async fn discover(Extension(state): Extension<Arc<ConsoleState>>) -> impl IntoResponse {
    Json(state.endpoints.discover.clone())
}

pub async fn topics(Extension(state): Extension<Arc<ConsoleState>>) -> impl IntoResponse {
    Json(TopicsEnvelope {
        topics: merge_topics(&state),
    })
}

pub async fn topic_info(
    Extension(state): Extension<Arc<ConsoleState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let name = normalize_topic_path(&name);
    match merge_topics(&state).into_iter().find(|t| t.name == name) {
        Some(t) => (StatusCode::OK, Json(t)).into_response(),
        None => (StatusCode::NOT_FOUND, "topic not found").into_response(),
    }
}

pub async fn topology(Extension(state): Extension<Arc<ConsoleState>>) -> impl IntoResponse {
    Json(build_topology(&state))
}

fn build_topology(state: &ConsoleState) -> TopologyResponse {
    let graph = super::bus_publish::topology_graph(state);
    TopologyResponse {
        nodes: graph
            .nodes
            .into_iter()
            .map(|n| TopologyNodeDto {
                id: n.id,
                kind: n.kind,
                label: n.label,
                type_name: n.type_name,
                msg_per_sec: n.msg_per_sec,
            })
            .collect(),
        edges: graph
            .edges
            .into_iter()
            .map(|e| TopologyEdgeDto {
                id: e.id,
                source: e.source,
                target: e.target,
                kind: e.kind,
                topic: e.topic,
            })
            .collect(),
    }
}

fn merge_topics(state: &ConsoleState) -> Vec<TopicResponse> {
    let rates = state.rates();
    let type_map: HashMap<String, String> = state.topic_types.snapshot().into_iter().collect();
    let counts = state.topology.counts_by_topic();

    let mut by_name: HashMap<String, TopicRate> = rates
        .topics
        .into_iter()
        .map(|t| (t.name.clone(), t))
        .collect();

    for topic in type_map.keys() {
        by_name.entry(topic.clone()).or_insert_with(|| TopicRate {
            name: topic.clone(),
            total_msgs: 0,
            total_bytes: 0,
            last_seen_unix_ms: 0,
            msg_per_sec: 0.0,
            bytes_per_sec: 0.0,
        });
    }
    for topic in counts.keys() {
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
            let rate = t.msg_per_sec.round() as u64;
            let (publishers, subscribers) = counts.get(&name).copied().unwrap_or((0, 0));
            Some(TopicResponse {
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

/// Axum `{*name}` may omit a leading `/`; registered topics usually include it.
fn normalize_topic_path(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return name.to_string();
    }
    if name.starts_with('/') {
        name.to_string()
    } else {
        format!("/{name}")
    }
}

pub async fn services(Extension(state): Extension<Arc<ConsoleState>>) -> impl IntoResponse {
    let rates = state.service_rates();
    let services = rates
        .services
        .into_iter()
        .map(|s| ServiceResponse {
            name: s.name,
            calls: s.calls,
            calls_per_sec: s.calls_per_sec.round() as u64,
            errors: s.errors,
            timeouts: 0,
            avg_latency_ms: s.avg_latency_ms,
            last_call_at: s.last_seen_unix_ms,
            workers: s.workers,
        })
        .collect();
    Json(ServicesEnvelope { services })
}

pub async fn actions(Extension(state): Extension<Arc<ConsoleState>>) -> impl IntoResponse {
    let rates = state.action_rates();
    let actions = rates
        .actions
        .into_iter()
        .map(|a| ActionResponse {
            name: a.name,
            runs: a.runs,
            runs_per_sec: a.runs_per_sec.round() as u64,
            active: a.active,
            workers: a.workers,
            errors: a.errors,
            avg_duration_ms: a.avg_duration_ms,
            last_run_at: a.last_seen_unix_ms,
        })
        .collect();
    Json(ActionsEnvelope { actions })
}

pub async fn events(
    Extension(state): Extension<Arc<ConsoleState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let history = state.events.recent();
    let rx = state.events.subscribe();

    let hist = tokio_stream::iter(
        history
            .into_iter()
            .map(|e| Ok::<Event, Infallible>(event_from_dto(&e))),
    );
    let live = BroadcastStream::new(rx).filter_map(|res| match res {
        Ok(entry) => Some(Ok::<Event, Infallible>(event_from_dto(&entry))),
        Err(_) => None,
    });

    let stream = hist.chain(live);
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

fn event_from_dto(e: &LogEntryDto) -> Event {
    match Event::default().event("log").id(e.id.clone()).json_data(e) {
        Ok(ev) => ev,
        Err(_) => Event::default().data("serialize error"),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TankStatusResponse {
    /// False when broker was started with `--no-tank` (menu hidden / acquire rejected).
    enabled: bool,
    running: bool,
    viewers: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TankSessionResponse {
    session_id: String,
    lease_ms: u64,
    viewers: usize,
}

pub async fn tank_status(Extension(state): Extension<Arc<ConsoleState>>) -> impl IntoResponse {
    let st = state.tank.status();
    Json(TankStatusResponse {
        enabled: state.tank_enabled,
        running: st.running && state.tank_enabled,
        viewers: if state.tank_enabled { st.viewers } else { 0 },
    })
}

pub async fn tank_session(Extension(state): Extension<Arc<ConsoleState>>) -> impl IntoResponse {
    if !state.tank_enabled {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "tank disabled",
                "enabled": false,
            })),
        )
            .into_response();
    }
    // `acquire` may block briefly while tank publishes the first pose — keep it
    // off the async worker so long-lived gateway streams stay responsive.
    let acquired = tokio::task::spawn_blocking({
        let tank = Arc::clone(&state.tank);
        move || tank.acquire()
    })
    .await;

    match acquired {
        Ok(Ok(session)) => {
            state.events.emit(
                "INFO",
                "tank",
                format!(
                    "session {} acquired (viewers={})",
                    &session.session_id[..8.min(session.session_id.len())],
                    session.viewers
                ),
            );
            (
                StatusCode::OK,
                Json(TankSessionResponse {
                    session_id: session.session_id,
                    lease_ms: session.lease.as_millis() as u64,
                    viewers: session.viewers,
                }),
            )
                .into_response()
        }
        Ok(Err(err)) => {
            state
                .events
                .emit("ERROR", "tank", format!("acquire failed: {err}"));
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": err.to_string() })),
            )
                .into_response()
        }
        Err(err) => {
            state.events.emit(
                "ERROR",
                "tank",
                format!("acquire task join failed: {err}"),
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "tank acquire interrupted" })),
            )
                .into_response()
        }
    }
}

pub async fn tank_heartbeat(
    Extension(state): Extension<Arc<ConsoleState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.tank.heartbeat(&id) {
        Ok(session) => (
            StatusCode::OK,
            Json(TankSessionResponse {
                session_id: session.session_id,
                lease_ms: session.lease.as_millis() as u64,
                viewers: session.viewers,
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "session not found" })),
        )
            .into_response(),
    }
}

pub async fn tank_release(
    Extension(state): Extension<Arc<ConsoleState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.tank.release(&id) {
        Ok(st) => {
            state.events.emit(
                "INFO",
                "tank",
                format!(
                    "session released (running={}, viewers={})",
                    st.running, st.viewers
                ),
            );
            (
                StatusCode::OK,
                Json(TankStatusResponse {
                    enabled: state.tank_enabled,
                    running: st.running && state.tank_enabled,
                    viewers: if state.tank_enabled { st.viewers } else { 0 },
                }),
            )
                .into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}
