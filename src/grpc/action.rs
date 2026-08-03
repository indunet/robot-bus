//! `ActionGateway` — bidirectional `Run` bridged to a ZMQ action-bus DEALER client.

use std::pin::Pin;
use std::sync::mpsc::{self as std_mpsc, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use crate::action_bus::{ActionClient, ActionKind as WireKind, ActionMessage};
use crate::errors::{parse_error_body, BusError};

use super::pb::action_gateway_server::ActionGateway;
use super::pb::{
    action_client_message, ActionClientMessage, ActionEvent, ActionKind, CancelCommand,
    GoalCommand,
};

type RunStream = Pin<Box<dyn Stream<Item = Result<ActionEvent, Status>> + Send + 'static>>;

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

enum SessionCommand {
    Goal(GoalCommand),
    Cancel(CancelCommand),
    Closed,
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

struct ActiveGoal {
    action_name: String,
    goal_id: String,
    /// Absolute deadline for waiting on bus replies; `None` means no deadline.
    deadline: Option<Instant>,
}

fn run_session(
    frontend: String,
    cmd_rx: std_mpsc::Receiver<SessionCommand>,
    event_tx: mpsc::Sender<Result<ActionEvent, Status>>,
) {
    let client = match ActionClient::new(Some(&frontend)) {
        Ok(client) => client,
        Err(err) => {
            let _ = event_tx.blocking_send(Err(bus_status(err)));
            return;
        }
    };

    let mut active: Option<ActiveGoal> = None;
    let mut client_closed = false;

    loop {
        // When a goal is in flight, do not block on the command channel — RESULT may
        // already be sitting on the ZMQ socket. Still drain non-blocking so CANCEL
        // can interrupt. When idle, block briefly for the next GOAL / disconnect.
        let cmd = if active.is_some() {
            match cmd_rx.try_recv() {
                Ok(cmd) => Some(cmd),
                Err(std_mpsc::TryRecvError::Empty) => None,
                Err(std_mpsc::TryRecvError::Disconnected) => {
                    client_closed = true;
                    if let Some(goal) = active.as_ref() {
                        let _ = client.submit_cancel(&goal.action_name, &goal.goal_id, b"");
                    }
                    None
                }
            }
        } else {
            match cmd_rx.recv_timeout(POLL_TICK) {
                Ok(cmd) => Some(cmd),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        };

        if let Some(cmd) = cmd {
            match cmd {
                SessionCommand::Goal(goal) => {
                    if goal.action_name.is_empty() {
                        let _ = event_tx.blocking_send(Err(Status::invalid_argument(
                            "action_name is required",
                        )));
                        return;
                    }
                    let goal_id = if goal.goal_id.is_empty() {
                        None
                    } else {
                        Some(goal.goal_id.as_str())
                    };
                    match client.submit_goal(&goal.action_name, &goal.goal, goal_id) {
                        Ok(gid) => {
                            active = Some(ActiveGoal {
                                action_name: goal.action_name,
                                goal_id: gid,
                                deadline: timeout_from_ms(goal.timeout_ms)
                                    .map(|d| Instant::now() + d),
                            });
                        }
                        Err(err) => {
                            let _ = event_tx.blocking_send(Err(bus_status(err)));
                            return;
                        }
                    }
                }
                SessionCommand::Cancel(cancel) => {
                    if cancel.action_name.is_empty() {
                        let _ = event_tx.blocking_send(Err(Status::invalid_argument(
                            "action_name is required",
                        )));
                        return;
                    }
                    if cancel.goal_id.is_empty() {
                        let _ = event_tx
                            .blocking_send(Err(Status::invalid_argument("goal_id is required")));
                        return;
                    }
                    if let Err(err) =
                        client.submit_cancel(&cancel.action_name, &cancel.goal_id, &cancel.body)
                    {
                        let _ = event_tx.blocking_send(Err(bus_status(err)));
                        return;
                    }
                    // Track so RESULT / NO_GOAL for this cancel is attributed.
                    if active.as_ref().map(|a| a.goal_id.as_str()) != Some(cancel.goal_id.as_str()) {
                        active = Some(ActiveGoal {
                            action_name: cancel.action_name,
                            goal_id: cancel.goal_id,
                            deadline: Some(Instant::now() + Duration::from_secs(30)),
                        });
                    }
                }
                SessionCommand::Closed => {
                    client_closed = true;
                    if let Some(goal) = active.as_ref() {
                        let _ = client.submit_cancel(&goal.action_name, &goal.goal_id, b"");
                    }
                    if active.is_none() {
                        break;
                    }
                }
            }
        }

        let Some(goal) = active.as_ref() else {
            if client_closed {
                break;
            }
            continue;
        };
        let action_name = goal.action_name.clone();
        let goal_id = goal.goal_id.clone();

        if let Some(deadline) = goal.deadline {
            if Instant::now() >= deadline {
                let _ = client.submit_cancel(&action_name, &goal_id, b"");
                let _ = event_tx.blocking_send(Err(Status::deadline_exceeded(
                    "action session timed out waiting for bus reply",
                )));
                return;
            }
        }

        let poll = goal
            .deadline
            .map(|d| d.saturating_duration_since(Instant::now()).min(POLL_TICK))
            .unwrap_or(POLL_TICK);

        match client.recv_message(Some(poll)) {
            Ok(msg) => {
                let matches = msg.action_name == action_name && msg.goal_id == goal_id;
                if !matches {
                    continue;
                }
                if msg.kind == WireKind::Result {
                    if let Some(err) = parse_error_body(&msg.body) {
                        let _ = event_tx.blocking_send(Err(bus_status(err)));
                        return;
                    }
                    let done = to_event(msg);
                    active = None;
                    if event_tx.blocking_send(Ok(done)).is_err() {
                        return;
                    }
                    if client_closed {
                        break;
                    }
                } else if event_tx.blocking_send(Ok(to_event(msg))).is_err() {
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
    type RunStream = RunStream;

    async fn run(
        &self,
        request: Request<Streaming<ActionClientMessage>>,
    ) -> Result<Response<Self::RunStream>, Status> {
        let mut inbound = request.into_inner();
        let frontend = self.action_frontend.clone();
        let (event_tx, event_rx) = mpsc::channel::<Result<ActionEvent, Status>>(64);
        let (cmd_tx, cmd_rx) = std_mpsc::channel::<SessionCommand>();

        thread::Builder::new()
            .name("grpc-zmq-action-run".into())
            .spawn(move || run_session(frontend, cmd_rx, event_tx))
            .map_err(|err| Status::internal(format!("spawn action run thread: {err}")))?;

        tokio::spawn(async move {
            while let Some(item) = inbound.next().await {
                match item {
                    Ok(ActionClientMessage {
                        msg: Some(action_client_message::Msg::Goal(goal)),
                    }) => {
                        if cmd_tx.send(SessionCommand::Goal(goal)).is_err() {
                            break;
                        }
                    }
                    Ok(ActionClientMessage {
                        msg: Some(action_client_message::Msg::Cancel(cancel)),
                    }) => {
                        if cmd_tx.send(SessionCommand::Cancel(cancel)).is_err() {
                            break;
                        }
                    }
                    Ok(ActionClientMessage { msg: None }) => {
                        let _ = cmd_tx.send(SessionCommand::Closed);
                        break;
                    }
                    Err(_) => {
                        let _ = cmd_tx.send(SessionCommand::Closed);
                        break;
                    }
                }
            }
            let _ = cmd_tx.send(SessionCommand::Closed);
        });

        let stream = ReceiverStream::new(event_rx);
        Ok(Response::new(Box::pin(stream) as Self::RunStream))
    }
}
