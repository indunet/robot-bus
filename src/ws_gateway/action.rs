//! `ActionGateway` — unary goal requests with server-streaming action events.
//!
//! Intentional cancel (WebSocket `CANCEL` frame / explicit cancel channel) submits
//! cancel on the action bus and **keeps** streaming until `RESULT`.
//! True transport disconnect (drop of the event receiver) still submits cancel and
//! abandons the session.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use zmq::Context;

use crate::action_bus::{ActionClient, ActionKind as WireKind, ActionMessage};
use crate::errors::{BusError, parse_error_body};
use crate::zmq_helpers::HighWaterMark;

use super::rpc_status::RpcStatus;
use super::ws_frame::{
    ACTION_KIND_CANCEL, ACTION_KIND_FEEDBACK, ACTION_KIND_GOAL, ACTION_KIND_RESULT,
};

const POLL_TICK: Duration = Duration::from_millis(5);

pub struct GoalSpec {
    pub action_name: String,
    pub goal: Vec<u8>,
    pub goal_id: String,
    pub timeout_ms: u32,
}

/// Wire action event: kind byte matches V3 DATA payload.
pub struct ActionWireEvent {
    pub kind: u8,
    pub body: Vec<u8>,
}

#[derive(Clone)]
pub struct ActionGatewayService {
    action_frontend: String,
    context: Arc<Context>,
}

impl std::fmt::Debug for ActionGatewayService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionGatewayService")
            .field("action_frontend", &self.action_frontend)
            .finish()
    }
}

impl ActionGatewayService {
    pub fn new(action_frontend: impl Into<String>) -> Self {
        Self {
            action_frontend: action_frontend.into(),
            context: Arc::new(Context::new()),
        }
    }

    /// Start a goal session.
    ///
    /// - Send on [`SendGoalSession::cancel`] for soft cancel (wait for RESULT).
    /// - Drop [`SendGoalSession::events`] for hard disconnect cancel.
    pub fn open_send_goal(&self, goal: GoalSpec) -> Result<SendGoalSession, RpcStatus> {
        if goal.action_name.is_empty() {
            return Err(RpcStatus::invalid_argument("action_name is required"));
        }

        let frontend = self.action_frontend.clone();
        let context = Arc::clone(&self.context);
        let (event_tx, event_rx) = mpsc::channel::<Result<ActionWireEvent, RpcStatus>>(64);
        let (cancel_tx, cancel_rx) = mpsc::channel::<Vec<u8>>(4);
        thread::Builder::new()
            .name("ws-zmq-action-goal".into())
            .spawn(move || run_goal(context, frontend, goal, event_tx, cancel_rx))
            .map_err(|err| RpcStatus::internal(format!("spawn action goal thread: {err}")))?;
        Ok(SendGoalSession {
            events: event_rx,
            cancel: cancel_tx,
        })
    }
}

/// Live SendGoal session: events plus an explicit cancel channel.
pub struct SendGoalSession {
    pub events: mpsc::Receiver<Result<ActionWireEvent, RpcStatus>>,
    /// Soft cancel: submit CANCEL on the bus and keep waiting for RESULT.
    pub cancel: mpsc::Sender<Vec<u8>>,
}

fn bus_status(err: BusError) -> RpcStatus {
    match err {
        BusError::Timeout(msg) => RpcStatus::deadline_exceeded(msg),
        BusError::NoWorker { name } => RpcStatus::unavailable(format!("no worker for '{name}'")),
        BusError::WorkerDied { name } => {
            RpcStatus::unavailable(format!("worker died for '{name}'"))
        }
        BusError::Cancelled { name } => RpcStatus::cancelled(format!("cancelled '{name}'")),
        BusError::NoGoal { goal_id } => RpcStatus::not_found(format!("no goal '{goal_id}'")),
        other => RpcStatus::internal(other.to_string()),
    }
}

fn timeout_from_ms(timeout_ms: u32) -> Option<Duration> {
    if timeout_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(u64::from(timeout_ms)))
    }
}

fn wire_kind_to_u8(kind: WireKind) -> u8 {
    match kind {
        WireKind::Goal => ACTION_KIND_GOAL,
        WireKind::Feedback => ACTION_KIND_FEEDBACK,
        WireKind::Result => ACTION_KIND_RESULT,
        WireKind::Cancel => ACTION_KIND_CANCEL,
    }
}

fn to_event(msg: ActionMessage) -> ActionWireEvent {
    ActionWireEvent {
        kind: wire_kind_to_u8(msg.kind),
        body: msg.body,
    }
}

fn run_goal(
    context: Arc<Context>,
    frontend: String,
    goal: GoalSpec,
    event_tx: mpsc::Sender<Result<ActionWireEvent, RpcStatus>>,
    mut cancel_rx: mpsc::Receiver<Vec<u8>>,
) {
    let client = match ActionClient::with_context_hwm(
        context.as_ref(),
        Some(&frontend),
        HighWaterMark::ACTION,
    ) {
        Ok(client) => client,
        Err(err) => {
            let _ = event_tx.blocking_send(Err(bus_status(err)));
            return;
        }
    };

    let goal_id = if goal.goal_id.is_empty() {
        None
    } else {
        Some(goal.goal_id.as_str())
    };
    let goal_id = match client.submit_goal(&goal.action_name, &goal.goal, goal_id) {
        Ok(goal_id) => goal_id,
        Err(err) => {
            let _ = event_tx.blocking_send(Err(bus_status(err)));
            return;
        }
    };
    let deadline = timeout_from_ms(goal.timeout_ms).map(|duration| Instant::now() + duration);
    let mut cancel_submitted = false;

    loop {
        while let Ok(body) = cancel_rx.try_recv() {
            if !cancel_submitted {
                let _ = client.submit_cancel(&goal.action_name, &goal_id, &body);
                cancel_submitted = true;
            }
        }

        if event_tx.is_closed() {
            if !cancel_submitted {
                let _ = client.submit_cancel(&goal.action_name, &goal_id, b"");
            }
            return;
        }

        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            if !cancel_submitted {
                let _ = client.submit_cancel(&goal.action_name, &goal_id, b"");
            }
            let _ = event_tx.blocking_send(Err(RpcStatus::deadline_exceeded(
                "action session timed out waiting for bus reply",
            )));
            return;
        }

        let poll = deadline
            .map(|deadline| {
                deadline
                    .saturating_duration_since(Instant::now())
                    .min(POLL_TICK)
            })
            .unwrap_or(POLL_TICK);

        match client.recv_message(Some(poll)) {
            Ok(msg) => {
                if msg.action_name != goal.action_name || msg.goal_id != goal_id {
                    continue;
                }
                let is_result = msg.kind == WireKind::Result;
                if is_result {
                    if let Some(err) = parse_error_body(&msg.body) {
                        let _ = event_tx.blocking_send(Err(bus_status(err)));
                        return;
                    }
                }
                if event_tx.blocking_send(Ok(to_event(msg))).is_err() {
                    if !is_result && !cancel_submitted {
                        let _ = client.submit_cancel(&goal.action_name, &goal_id, b"");
                    }
                    return;
                }
                if is_result {
                    return;
                }
            }
            Err(BusError::Timeout(_)) => {}
            Err(err) => {
                let _ = event_tx.blocking_send(Err(bus_status(err)));
                return;
            }
        }
    }
}
