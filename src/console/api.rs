//! REST / SSE monitoring endpoints for the embedded console.

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use serde::Serialize;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use super::state::{ConsoleState, LogEntryDto};

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
    errors: u64,
    avg_duration_ms: u64,
    last_run_at: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionsEnvelope {
    actions: Vec<ActionResponse>,
}

pub async fn status(State(state): State<Arc<ConsoleState>>) -> impl IntoResponse {
    let rates = state.rates();
    let svc = state.service_rates();
    let act = state.action_rates();
    Json(StatusResponse {
        status: "ONLINE",
        version: state.version.to_string(),
        uptime: state.uptime_secs(),
        pid: state.pid,
        grpc_addr: state.endpoints.grpc.clone(),
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

pub async fn topics(State(state): State<Arc<ConsoleState>>) -> impl IntoResponse {
    let rates = state.rates();
    let topics = rates
        .topics
        .into_iter()
        .map(|t| {
            let rate = t.msg_per_sec.round() as u64;
            TopicResponse {
                name: t.name,
                msg_per_sec: rate,
                bytes_per_sec: t.bytes_per_sec.round() as u64,
                last_seen: t.last_seen_unix_ms,
                total_msgs: t.total_msgs,
                sparkline: vec![rate; 20],
                subscribers: 0,
                publishers: 0,
            }
        })
        .collect();
    Json(TopicsEnvelope { topics })
}

pub async fn services(State(state): State<Arc<ConsoleState>>) -> impl IntoResponse {
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

pub async fn actions(State(state): State<Arc<ConsoleState>>) -> impl IntoResponse {
    let rates = state.action_rates();
    let actions = rates
        .actions
        .into_iter()
        .map(|a| ActionResponse {
            name: a.name,
            runs: a.runs,
            runs_per_sec: a.runs_per_sec.round() as u64,
            active: a.active,
            errors: a.errors,
            avg_duration_ms: a.avg_duration_ms,
            last_run_at: a.last_seen_unix_ms,
        })
        .collect();
    Json(ActionsEnvelope { actions })
}

pub async fn events(
    State(state): State<Arc<ConsoleState>>,
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
