//! gRPC-mode runtime for [`super::Node`]: client-side subscribe / service / action.
//!
//! Does not use the ZMQ [`super::Executor`] poll loop. Messages arrive via the
//! broker gRPC gateway and are dispatched on the `spin` thread.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio_stream::StreamExt;
use tonic::Request;
use uuid::Uuid;

use crate::action_bus::{ActionKind, ActionMessage};
use crate::errors::{BusError, Result};
use crate::grpc::pb::action_gateway_client::ActionGatewayClient;
use crate::grpc::pb::message_gateway_client::MessageGatewayClient;
use crate::grpc::pb::service_gateway_client::ServiceGatewayClient;
use crate::grpc::pb::{
    action_client_message, ActionClientMessage, ActionKind as PbActionKind, CancelCommand,
    GoalCommand, ServiceCallRequest, SubscribeRequest,
};
use crate::runtime::callback_group::{CallbackGroup, SubscriptionCallback};
use crate::runtime::executor::ShutdownHandle;
use crate::runtime::registrations::MessageCallback;
use crate::runtime::timers::{
    effective_poll_timeout_ms, tick_timers, Timer, TimerCallback, TimerHandle,
};

const DEFAULT_GRPC_URL: &str = "http://127.0.0.1:15770";
const DEFAULT_SPIN_TIMEOUT_MS: i64 = 250;

#[derive(Debug)]
struct TopicEvent {
    topic: String,
    payload: Vec<u8>,
}

/// Shared handle used by gRPC service / action clients (keeps the runtime alive).
#[derive(Clone)]
pub(crate) struct GrpcClientContext {
    url: String,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl GrpcClientContext {
    pub(crate) fn call_service(
        &self,
        service_name: &str,
        body: &[u8],
        request_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>> {
        let url = self.url.clone();
        let service_name = service_name.to_string();
        let body = body.to_vec();
        let request_id = request_id.map(str::to_string).unwrap_or_default();
        let timeout_ms = timeout_ms_u32(timeout);

        self.runtime.block_on(async move {
            let mut client = ServiceGatewayClient::connect(url)
                .await
                .map_err(|err| BusError::Protocol(format!("gRPC connect failed: {err}")))?;
            let resp = client
                .call(Request::new(ServiceCallRequest {
                    service_name,
                    request: body,
                    request_id,
                    timeout_ms,
                }))
                .await
                .map_err(map_tonic_status)?;
            Ok(resp.into_inner().response)
        })
    }

    pub(crate) fn send_goal(
        &self,
        action_name: &str,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ActionMessage>> {
        let url = self.url.clone();
        let action_name = action_name.to_string();
        let body = body.to_vec();
        let goal_id = goal_id
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
        let timeout_ms = timeout_ms_u32(timeout);

        self.runtime.block_on(async move {
            let mut client = ActionGatewayClient::connect(url)
                .await
                .map_err(|err| BusError::Protocol(format!("gRPC connect failed: {err}")))?;
            let outbound = tokio_stream::once(ActionClientMessage {
                msg: Some(action_client_message::Msg::Goal(GoalCommand {
                    action_name: action_name.clone(),
                    goal: body,
                    goal_id: goal_id.clone(),
                    timeout_ms,
                })),
            });
            let mut stream = client
                .run(Request::new(outbound))
                .await
                .map_err(map_tonic_status)?
                .into_inner();

            let mut out = Vec::new();
            while let Some(item) = stream.next().await {
                let ev = item.map_err(map_tonic_status)?;
                let kind = pb_action_kind(ev.kind)?;
                let done = kind == ActionKind::Result;
                out.push(ActionMessage {
                    action_name: ev.action_name,
                    goal_id: ev.goal_id,
                    kind,
                    body: ev.body,
                });
                if done {
                    break;
                }
            }
            if out.last().map(|m| m.kind) != Some(ActionKind::Result) {
                return Err(BusError::Protocol(format!(
                    "action '{action_name}' completed without RESULT"
                )));
            }
            Ok(out)
        })
    }

    pub(crate) fn cancel_goal(
        &self,
        action_name: &str,
        goal_id: &str,
        body: &[u8],
        _timeout: Option<Duration>,
    ) -> Result<ActionMessage> {
        let url = self.url.clone();
        let action_name = action_name.to_string();
        let goal_id = goal_id.to_string();
        let body = body.to_vec();

        self.runtime.block_on(async move {
            let mut client = ActionGatewayClient::connect(url)
                .await
                .map_err(|err| BusError::Protocol(format!("gRPC connect failed: {err}")))?;
            let outbound = tokio_stream::once(ActionClientMessage {
                msg: Some(action_client_message::Msg::Cancel(CancelCommand {
                    action_name: action_name.clone(),
                    goal_id: goal_id.clone(),
                    body,
                })),
            });
            let mut stream = client
                .run(Request::new(outbound))
                .await
                .map_err(map_tonic_status)?
                .into_inner();

            let ev = stream
                .next()
                .await
                .ok_or_else(|| {
                    BusError::Protocol(format!(
                        "action '{action_name}' cancel stream ended without event"
                    ))
                })?
                .map_err(map_tonic_status)?;
            Ok(ActionMessage {
                action_name: ev.action_name,
                goal_id: ev.goal_id,
                kind: pb_action_kind(ev.kind)?,
                body: ev.body,
            })
        })
    }
}

struct GrpcState {
    topic_callbacks: HashMap<String, Vec<SubscriptionCallback>>,
    active_topics: HashSet<String>,
    timers: Vec<Timer>,
    next_timer_id: u64,
}

/// Owns a tokio runtime and dispatches gRPC subscription / timer callbacks.
pub struct GrpcRuntime {
    url: String,
    runtime: Arc<tokio::runtime::Runtime>,
    running: Arc<AtomicBool>,
    inbound_tx: Sender<TopicEvent>,
    inbound_rx: Mutex<Receiver<TopicEvent>>,
    state: Mutex<GrpcState>,
}

impl GrpcRuntime {
    pub fn new(url: impl Into<String>) -> Result<Self> {
        let url = url.into();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("robot-bus-grpc")
            .build()
            .map_err(|err| BusError::Protocol(format!("tokio runtime: {err}")))?;
        let (inbound_tx, inbound_rx) = mpsc::channel();
        Ok(Self {
            url,
            runtime: Arc::new(runtime),
            running: Arc::new(AtomicBool::new(false)),
            inbound_tx,
            inbound_rx: Mutex::new(inbound_rx),
            state: Mutex::new(GrpcState {
                topic_callbacks: HashMap::new(),
                active_topics: HashSet::new(),
                timers: Vec::new(),
                next_timer_id: 1,
            }),
        })
    }

    pub fn default_url() -> &'static str {
        DEFAULT_GRPC_URL
    }

    pub fn client_context(&self) -> GrpcClientContext {
        GrpcClientContext {
            url: self.url.clone(),
            runtime: Arc::clone(&self.runtime),
        }
    }

    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle::from_flag(Arc::clone(&self.running))
    }

    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Release);
    }

    pub fn subscribe(
        &self,
        topic: &str,
        callback: MessageCallback,
        group: CallbackGroup,
    ) -> Result<()> {
        let mut state = self.lock_state()?;
        state
            .topic_callbacks
            .entry(topic.to_string())
            .or_default()
            .push(SubscriptionCallback { callback, group });

        if state.active_topics.insert(topic.to_string()) {
            self.spawn_subscription(topic.to_string());
        }
        Ok(())
    }

    pub fn create_timer(
        &self,
        period: Duration,
        callback: TimerCallback,
        group: CallbackGroup,
    ) -> Result<TimerHandle> {
        let mut state = self.lock_state()?;
        let id = state.next_timer_id;
        state.next_timer_id += 1;
        state
            .timers
            .push(Timer::new(id, period, callback, group));
        Ok(TimerHandle { id })
    }

    pub fn cancel_timer(&self, handle: TimerHandle) -> Result<()> {
        let mut state = self.lock_state()?;
        if let Some(timer) = state.timers.iter_mut().find(|t| t.id == handle.id) {
            timer.cancelled = true;
            Ok(())
        } else {
            Err(BusError::Protocol(format!(
                "unknown timer id {}",
                handle.id
            )))
        }
    }

    pub fn spin_once(&self, timeout: Option<Duration>) -> Result<bool> {
        self.running.store(true, Ordering::Release);
        let timeout_ms = timeout
            .map(|d| d.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or(DEFAULT_SPIN_TIMEOUT_MS);
        self.poll_once(timeout_ms)
    }

    pub fn spin_some(&self, timeout: Option<Duration>) -> Result<()> {
        let _ = self.spin_once(timeout)?;
        Ok(())
    }

    pub fn spin(&self) -> Result<()> {
        self.running.store(true, Ordering::Release);
        while self.running.load(Ordering::Acquire) {
            let timeout_ms = {
                let state = self.lock_state()?;
                effective_poll_timeout_ms(&state.timers, DEFAULT_SPIN_TIMEOUT_MS, Instant::now())
            };
            let _ = self.poll_once(timeout_ms)?;
        }
        Ok(())
    }

    fn poll_once(&self, timeout_ms: i64) -> Result<bool> {
        let mut worked = false;
        {
            let mut state = self.lock_state()?;
            if tick_timers(&mut state.timers, Instant::now(), None) {
                worked = true;
            }
        }

        let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(0) as u64);
        loop {
            let event = {
                let rx = self
                    .inbound_rx
                    .lock()
                    .map_err(|_| BusError::Protocol("gRPC inbound mutex poisoned".into()))?;
                match rx.try_recv() {
                    Ok(ev) => Some(ev),
                    Err(TryRecvError::Empty) => None,
                    Err(TryRecvError::Disconnected) => {
                        return Err(BusError::Protocol(
                            "gRPC subscription channel disconnected".into(),
                        ));
                    }
                }
            };

            if let Some(event) = event {
                self.dispatch_topic(&event.topic, &event.payload)?;
                worked = true;
                continue;
            }

            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(worked)
    }

    fn dispatch_topic(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let callbacks = {
            let state = self.lock_state()?;
            callbacks_for_topic(topic, &state.topic_callbacks)
        };
        for entry in callbacks {
            let callback = Arc::clone(&entry.callback);
            let topic = topic.to_string();
            let payload = payload.to_vec();
            entry.group.run(None, move || callback(&topic, &payload));
        }
        Ok(())
    }

    fn spawn_subscription(&self, topic: String) {
        let url = self.url.clone();
        let tx = self.inbound_tx.clone();
        let running = Arc::clone(&self.running);
        self.runtime.spawn(async move {
            // Keep trying while the node may still spin; brief backoff on errors.
            loop {
                if let Err(err) = run_subscribe_stream(&url, &topic, &tx).await {
                    log::warn!("gRPC subscribe '{topic}' ended: {err}");
                }
                if !running.load(Ordering::Acquire) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        });
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, GrpcState>> {
        self.state
            .lock()
            .map_err(|_| BusError::Protocol("gRPC state mutex poisoned".into()))
    }
}

async fn run_subscribe_stream(url: &str, topic: &str, tx: &Sender<TopicEvent>) -> Result<()> {
    let mut client = MessageGatewayClient::connect(url.to_string())
        .await
        .map_err(|err| BusError::Protocol(format!("gRPC connect failed: {err}")))?;
    let mut stream = client
        .subscribe(Request::new(SubscribeRequest {
            topic: topic.to_string(),
        }))
        .await
        .map_err(map_tonic_status)?
        .into_inner();

    while let Some(item) = stream.next().await {
        let msg = item.map_err(map_tonic_status)?;
        if tx
            .send(TopicEvent {
                topic: msg.topic,
                payload: msg.payload,
            })
            .is_err()
        {
            break;
        }
    }
    Ok(())
}

fn callbacks_for_topic(
    topic: &str,
    topic_callbacks: &HashMap<String, Vec<SubscriptionCallback>>,
) -> Vec<SubscriptionCallback> {
    let mut matched = Vec::new();
    let mut seen = HashSet::new();
    for (pattern, callbacks) in topic_callbacks {
        if topic == pattern.as_str() || (!pattern.is_empty() && topic.starts_with(pattern)) {
            for entry in callbacks {
                let ptr = Arc::as_ptr(&entry.callback) as *const ();
                if seen.insert(ptr) {
                    matched.push(entry.clone());
                }
            }
        }
    }
    matched
}

fn timeout_ms_u32(timeout: Option<Duration>) -> u32 {
    match timeout {
        None => 0,
        Some(d) => d.as_millis().min(u32::MAX as u128) as u32,
    }
}

fn pb_action_kind(kind: i32) -> Result<ActionKind> {
    match PbActionKind::try_from(kind) {
        Ok(PbActionKind::Goal) => Ok(ActionKind::Goal),
        Ok(PbActionKind::Feedback) => Ok(ActionKind::Feedback),
        Ok(PbActionKind::Result) => Ok(ActionKind::Result),
        Ok(PbActionKind::Cancel) => Ok(ActionKind::Cancel),
        Ok(PbActionKind::Unspecified) | Err(_) => Err(BusError::Protocol(format!(
            "unknown gRPC action kind: {kind}"
        ))),
    }
}

fn map_tonic_status(status: tonic::Status) -> BusError {
    use tonic::Code;
    match status.code() {
        Code::DeadlineExceeded => BusError::Timeout(status.message().to_string()),
        Code::NotFound => {
            let msg = status.message();
            if let Some(rest) = msg.strip_prefix("no goal ") {
                BusError::NoGoal {
                    goal_id: rest.trim_matches('\'').to_string(),
                }
            } else {
                BusError::Protocol(status.to_string())
            }
        }
        Code::Unavailable => {
            let msg = status.message();
            if let Some(rest) = msg.strip_prefix("no worker for ") {
                BusError::NoWorker {
                    name: rest.trim_matches('\'').to_string(),
                }
            } else if let Some(rest) = msg.strip_prefix("worker died for ") {
                BusError::WorkerDied {
                    name: rest.trim_matches('\'').to_string(),
                }
            } else {
                BusError::Protocol(status.to_string())
            }
        }
        _ => BusError::Protocol(status.to_string()),
    }
}
