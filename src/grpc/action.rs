//! `ActionGateway` — unary goal requests with server-streaming action events.

use std::pin::Pin;
use std::thread;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::action_bus::{ActionClient, ActionKind as WireKind, ActionMessage};
use crate::errors::{BusError, parse_error_body};

use super::pb::action_gateway_server::ActionGateway;
use super::pb::{ActionEvent, ActionKind, GoalCommand};

type SendGoalStream = Pin<Box<dyn Stream<Item = Result<ActionEvent, Status>> + Send + 'static>>;

const POLL_TICK: Duration = Duration::from_millis(50);

#[derive(Clone, Debug)]
pub struct ActionGatewayService {
    action_frontend: String,
}

impl ActionGatewayService {
    pub fn new(action_frontend: impl Into<String>) -> Self {
        Self {
            action_frontend: action_frontend.into(),
        }
    }
}

fn bus_status(err: BusError) -> Status {
    match err {
        BusError::Timeout(msg) => Status::deadline_exceeded(msg),
        BusError::NoWorker { name } => Status::unavailable(format!("no worker for '{name}'")),
        BusError::WorkerDied { name } => Status::unavailable(format!("worker died for '{name}'")),
        BusError::Cancelled { name } => Status::cancelled(format!("cancelled '{name}'")),
        BusError::NoGoal { goal_id } => Status::not_found(format!("no goal '{goal_id}'")),
        other => Status::internal(other.to_string()),
    }
}

fn timeout_from_ms(timeout_ms: u32) -> Option<Duration> {
    if timeout_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(u64::from(timeout_ms)))
    }
}

fn wire_kind_to_proto(kind: WireKind) -> ActionKind {
    match kind {
        WireKind::Goal => ActionKind::Goal,
        WireKind::Feedback => ActionKind::Feedback,
        WireKind::Result => ActionKind::Result,
        WireKind::Cancel => ActionKind::Cancel,
    }
}

fn to_event(msg: ActionMessage) -> ActionEvent {
    ActionEvent {
        action_name: msg.action_name,
        goal_id: msg.goal_id,
        kind: wire_kind_to_proto(msg.kind).into(),
        body: msg.body,
    }
}

fn run_goal(
    frontend: String,
    goal: GoalCommand,
    event_tx: mpsc::Sender<Result<ActionEvent, Status>>,
) {
    let client = match ActionClient::new(Some(&frontend)) {
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

    loop {
        if event_tx.is_closed() {
            let _ = client.submit_cancel(&goal.action_name, &goal_id, b"");
            return;
        }

        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            let _ = client.submit_cancel(&goal.action_name, &goal_id, b"");
            let _ = event_tx.blocking_send(Err(Status::deadline_exceeded(
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
                    if !is_result {
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

#[tonic::async_trait]
impl ActionGateway for ActionGatewayService {
    type SendGoalStream = SendGoalStream;

    async fn send_goal(
        &self,
        request: Request<GoalCommand>,
    ) -> Result<Response<Self::SendGoalStream>, Status> {
        let goal = request.into_inner();
        if goal.action_name.is_empty() {
            return Err(Status::invalid_argument("action_name is required"));
        }

        let frontend = self.action_frontend.clone();
        let (event_tx, event_rx) = mpsc::channel::<Result<ActionEvent, Status>>(64);
        thread::Builder::new()
            .name("grpc-zmq-action-goal".into())
            .spawn(move || run_goal(frontend, goal, event_tx))
            .map_err(|err| Status::internal(format!("spawn action goal thread: {err}")))?;

        let stream = ReceiverStream::new(event_rx);
        Ok(Response::new(Box::pin(stream) as Self::SendGoalStream))
    }
}
