use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use prost::Message;

use crate::action_bus::{ActionClient as BusActionClient, ActionKind, ActionMessage};
use crate::errors::{BusError, Result, parse_error_body};
use crate::runtime::console_ready::{self, ReadyKind};
use crate::runtime::topology_register::TopologyEndpointGuard;
#[cfg(feature = "ws")]
use crate::runtime::ws_runtime::WsClientContext;
use crate::typed::{Action, ActionOutcome};
use crate::zmq_helpers::HighWaterMark;

/// Action server handle returned by [`Node::create_action_server`] /
/// [`Node::create_action_server_raw`].
#[derive(Clone, Debug)]
pub struct NodeActionServer {
    pub(super) id: u64,
    pub(super) action_name: String,
}

impl NodeActionServer {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn action_name(&self) -> &str {
        &self.action_name
    }
}

/// Callback invoked for each action feedback as it arrives.
pub type RawActionFeedbackCallback = Arc<dyn Fn(&ActionMessage) + Send + Sync + 'static>;

pub(super) fn spawn_zmq_goal(
    context: zmq::Context,
    endpoint: String,
    action_name: String,
    body: Vec<u8>,
    requested_goal_id: Option<String>,
    timeout: Option<Duration>,
    hwm: HighWaterMark,
    feedback_callback: Option<RawActionFeedbackCallback>,
) -> Result<RawGoalHandle> {
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let (event_tx, event_rx) = mpsc::channel();
    let (command_tx, command_rx) = mpsc::channel();
    let thread_action_name = action_name.clone();

    thread::Builder::new()
        .name(format!("action-{}", action_name))
        .spawn(move || {
            let client = match BusActionClient::with_context_hwm(&context, Some(&endpoint), hwm) {
                Ok(client) => client,
                Err(err) => {
                    let _ = ready_tx.send(Err(err));
                    return;
                }
            };
            let goal_id = match client.submit_goal(
                &thread_action_name,
                &body,
                requested_goal_id.as_deref(),
            ) {
                Ok(goal_id) => goal_id,
                Err(err) => {
                    let _ = ready_tx.send(Err(err));
                    return;
                }
            };
            if ready_tx.send(Ok(goal_id.clone())).is_err() {
                return;
            }

            let deadline = timeout.map(|duration| Instant::now() + duration);
            loop {
                while let Ok(command) = command_rx.try_recv() {
                    match command {
                        GoalCommand::Cancel(body) => {
                            if let Err(err) =
                                client.submit_cancel(&thread_action_name, &goal_id, &body)
                            {
                                let _ = event_tx.send(Err(err));
                                return;
                            }
                        }
                    }
                }

                if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                    let _ = client.submit_cancel(&thread_action_name, &goal_id, b"");
                    let _ = event_tx.send(Err(BusError::Timeout(format!(
                        "action client timed out after {}s",
                        timeout.unwrap_or_default().as_secs_f64()
                    ))));
                    return;
                }

                let poll_timeout = deadline
                    .map(|deadline| {
                        deadline
                            .saturating_duration_since(Instant::now())
                            .min(Duration::from_millis(20))
                    })
                    .unwrap_or(Duration::from_millis(20));
                let message = match client.recv_message(Some(poll_timeout)) {
                    Ok(message) => message,
                    Err(BusError::Timeout(_)) => continue,
                    Err(err) => {
                        let _ = event_tx.send(Err(err));
                        return;
                    }
                };
                if message.action_name != thread_action_name || message.goal_id != goal_id {
                    let _ = event_tx.send(Err(BusError::Protocol(format!(
                        "unexpected message for {:?}/{:?}",
                        message.action_name, message.goal_id
                    ))));
                    return;
                }
                if message.kind == ActionKind::Feedback {
                    if let Some(callback) = &feedback_callback {
                        callback(&message);
                    }
                }
                let done = message.kind == ActionKind::Result;
                if done {
                    if let Some(err) = parse_error_body(&message.body) {
                        let _ = event_tx.send(Err(err));
                        return;
                    }
                }
                if event_tx.send(Ok(message)).is_err() || done {
                    return;
                }
            }
        })
        .map_err(|err| BusError::Protocol(format!("spawn action thread: {err}")))?;

    let goal_id = ready_rx
        .recv()
        .map_err(|_| BusError::Protocol("action thread ended before submitting goal".into()))??;
    Ok(RawGoalHandle {
        inner: Arc::new(GoalHandleCore {
            action_name,
            goal_id,
            events: Mutex::new(event_rx),
            messages: Mutex::new(Vec::new()),
            control: GoalControl::Zmq(command_tx),
            completed: AtomicBool::new(false),
        }),
    })
}

pub(super) enum GoalControl {
    Zmq(Sender<GoalCommand>),
    #[cfg(feature = "ws")]
    Ws(crate::runtime::ws_runtime::WsCancelHandle),
}

pub(super) enum GoalCommand {
    Cancel(Vec<u8>),
}

pub(super) struct GoalHandleCore {
    action_name: String,
    goal_id: String,
    events: Mutex<Receiver<Result<ActionMessage>>>,
    messages: Mutex<Vec<ActionMessage>>,
    control: GoalControl,
    completed: AtomicBool,
}

impl GoalHandleCore {
    fn wait_result(&self) -> Result<ActionMessage> {
        if let Some(result) = self
            .messages
            .lock()
            .map_err(|_| BusError::Protocol("action messages mutex poisoned".into()))?
            .iter()
            .find(|message| message.kind == ActionKind::Result)
            .cloned()
        {
            return Ok(result);
        }

        loop {
            let event = self
                .events
                .lock()
                .map_err(|_| BusError::Protocol("action event mutex poisoned".into()))?
                .recv()
                .map_err(|_| {
                    BusError::Protocol(format!(
                        "action '{}' goal '{}' ended without RESULT",
                        self.action_name, self.goal_id
                    ))
                })??;
            let done = event.kind == ActionKind::Result;
            self.messages
                .lock()
                .map_err(|_| BusError::Protocol("action messages mutex poisoned".into()))?
                .push(event.clone());
            if done {
                self.completed.store(true, Ordering::Release);
                return Ok(event);
            }
        }
    }

    fn collect(&self) -> Result<Vec<ActionMessage>> {
        self.wait_result()?;
        self.messages
            .lock()
            .map(|messages| messages.clone())
            .map_err(|_| BusError::Protocol("action messages mutex poisoned".into()))
    }

    fn cancel(&self, body: &[u8]) -> Result<()> {
        match &self.control {
            GoalControl::Zmq(commands) => commands
                .send(GoalCommand::Cancel(body.to_vec()))
                .map_err(|_| BusError::Closed),
            #[cfg(feature = "ws")]
            GoalControl::Ws(abort) => {
                abort.abort();
                Ok(())
            }
        }
    }
}

impl Drop for GoalHandleCore {
    fn drop(&mut self) {
        if self.completed.load(Ordering::Acquire) {
            return;
        }
        match &self.control {
            GoalControl::Zmq(commands) => {
                let _ = commands.send(GoalCommand::Cancel(Vec::new()));
            }
            #[cfg(feature = "ws")]
            GoalControl::Ws(abort) => abort.abort(),
        }
    }
}

/// Live handle for one raw (opaque bytes) action goal.
#[derive(Clone)]
pub struct RawGoalHandle {
    pub(super) inner: Arc<GoalHandleCore>,
}

impl RawGoalHandle {
    pub fn goal_id(&self) -> &str {
        &self.inner.goal_id
    }

    pub fn action_name(&self) -> &str {
        &self.inner.action_name
    }

    pub fn wait_result(&self) -> Result<ActionMessage> {
        self.inner.wait_result()
    }

    pub fn collect(&self) -> Result<Vec<ActionMessage>> {
        self.inner.collect()
    }

    /// Best-effort cancellation. This does not wait for server acknowledgement.
    pub fn cancel(&self) -> Result<()> {
        self.inner.cancel(&[])
    }

    /// Best-effort cancellation with an opaque ZMQ cancel payload.
    ///
    /// On native gRPC this aborts the response stream (body ignored). Browser
    /// WebSocket clients send an explicit CANCEL frame instead.
    pub fn cancel_with_body(&self, body: &[u8]) -> Result<()> {
        self.inner.cancel(body)
    }
}

/// Live handle for one typed action goal.
pub struct GoalHandle<A: Action> {
    pub(super) inner: RawGoalHandle,
    pub(super) _marker: PhantomData<A>,
}

impl<A: Action> Clone for GoalHandle<A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<A: Action> GoalHandle<A> {
    pub fn goal_id(&self) -> &str {
        self.inner.goal_id()
    }

    pub fn action_name(&self) -> &str {
        self.inner.action_name()
    }

    pub fn wait_result(&self) -> Result<A::Result> {
        let message = self.inner.wait_result()?;
        A::Result::decode(message.body.as_slice()).map_err(|err| {
            BusError::Protocol(format!(
                "action '{}' result decode failed: {err}",
                self.action_name()
            ))
        })
    }

    pub fn cancel(&self) -> Result<()> {
        self.inner.cancel()
    }
}

/// Raw (opaque bytes) action client from [`Node::create_action_client_raw`].
pub struct NodeActionClientRaw {
    pub(super) inner: ActionClientInner,
    pub(super) action_name: String,
    pub(super) console_url: Option<String>,
    pub(super) _topology: Option<Arc<TopologyEndpointGuard>>,
}

pub(super) enum ActionClientInner {
    Zmq {
        context: zmq::Context,
        endpoint: String,
        hwm: Mutex<HighWaterMark>,
    },
    #[cfg(feature = "ws")]
    Ws(WsClientContext),
}

impl NodeActionClientRaw {
    pub fn action_name(&self) -> &str {
        &self.action_name
    }

    /// Best-effort: console reports `workers > 0` for this action.
    pub fn action_server_is_ready(&self) -> bool {
        console_ready::is_ready(
            self.console_url.as_deref(),
            ReadyKind::Action,
            &self.action_name,
        )
    }

    /// Poll until [`action_server_is_ready`](Self::action_server_is_ready) or `timeout`.
    pub fn wait_for_action_server(&self, timeout: Option<Duration>) -> bool {
        console_ready::wait_until_ready(
            self.console_url.as_deref(),
            ReadyKind::Action,
            &self.action_name,
            timeout,
        )
    }

    pub fn send_goal(
        &self,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
        feedback_callback: Option<RawActionFeedbackCallback>,
    ) -> Result<RawGoalHandle> {
        match &self.inner {
            ActionClientInner::Zmq {
                context,
                endpoint,
                hwm,
            } => {
                let hwm = *hwm
                    .lock()
                    .map_err(|_| BusError::Protocol("action HWM mutex poisoned".into()))?;
                spawn_zmq_goal(
                    context.clone(),
                    endpoint.clone(),
                    self.action_name.clone(),
                    body.to_vec(),
                    goal_id.map(str::to_string),
                    timeout,
                    hwm,
                    feedback_callback,
                )
            }
            #[cfg(feature = "ws")]
            ActionClientInner::Ws(ctx) => ctx
                .send_goal(&self.action_name, body, goal_id, timeout, feedback_callback)
                .map(|session| RawGoalHandle {
                    inner: Arc::new(GoalHandleCore {
                        action_name: self.action_name.clone(),
                        goal_id: session.goal_id,
                        events: Mutex::new(session.events),
                        messages: Mutex::new(Vec::new()),
                        control: GoalControl::Ws(session.abort),
                        completed: AtomicBool::new(false),
                    }),
                }),
        }
    }

    /// Compatibility helper that waits for and collects FEEDBACK/RESULT.
    pub fn send_goal_and_wait(
        &self,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ActionMessage>> {
        self.send_goal(body, goal_id, timeout, None)?.collect()
    }

    /// Alias for [`send_goal_and_wait`](Self::send_goal_and_wait).
    pub fn collect(
        &self,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ActionMessage>> {
        self.send_goal_and_wait(body, goal_id, timeout)
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        match &self.inner {
            ActionClientInner::Zmq { hwm, .. } => hwm
                .lock()
                .map(|hwm| *hwm)
                .map_err(|_| BusError::Protocol("action HWM mutex poisoned".into())),
            #[cfg(feature = "ws")]
            ActionClientInner::Ws(_) => Err(BusError::Protocol(
                "high_water_mark is not available in WebSocket RPC node mode".into(),
            )),
        }
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        match &self.inner {
            ActionClientInner::Zmq { hwm: current, .. } => {
                *current
                    .lock()
                    .map_err(|_| BusError::Protocol("action HWM mutex poisoned".into()))? = hwm;
                Ok(())
            }
            #[cfg(feature = "ws")]
            ActionClientInner::Ws(_) => Err(BusError::Protocol(
                "set_high_water_mark is not available in WebSocket RPC node mode".into(),
            )),
        }
    }
}

/// Typed action client returned by [`Node::create_action_client`] (ROS 2 style).
pub struct NodeActionClient<A: Action> {
    pub(super) inner: NodeActionClientRaw,
    pub(super) _marker: PhantomData<A>,
}

impl<A: Action> NodeActionClient<A> {
    pub fn action_name(&self) -> &str {
        self.inner.action_name()
    }

    pub fn action_server_is_ready(&self) -> bool {
        self.inner.action_server_is_ready()
    }

    pub fn wait_for_action_server(&self, timeout: Option<Duration>) -> bool {
        self.inner.wait_for_action_server(timeout)
    }

    pub fn send_goal(
        &self,
        goal: &A::Goal,
        goal_id: Option<&str>,
        timeout: Option<Duration>,
        feedback_callback: Option<Arc<dyn Fn(A::Feedback) + Send + Sync + 'static>>,
    ) -> Result<GoalHandle<A>> {
        let action_name = self.action_name().to_string();
        let raw_callback = feedback_callback.map(|callback| {
            Arc::new(move |message: &ActionMessage| {
                match A::Feedback::decode(message.body.as_slice()) {
                    Ok(feedback) => callback(feedback),
                    Err(err) => {
                        log::warn!("action '{}' feedback decode failed: {err}", action_name)
                    }
                }
            }) as RawActionFeedbackCallback
        });
        let inner = self
            .inner
            .send_goal(&goal.encode_to_vec(), goal_id, timeout, raw_callback)?;
        Ok(GoalHandle {
            inner,
            _marker: PhantomData,
        })
    }

    /// Compatibility helper that waits for a result and collects feedback.
    pub fn send_goal_and_wait(
        &self,
        goal: &A::Goal,
        goal_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<ActionOutcome<A>> {
        let messages = self
            .inner
            .send_goal_and_wait(&goal.encode_to_vec(), goal_id, timeout)?;
        let mut feedbacks = Vec::new();
        let mut result = None;
        for msg in messages {
            match msg.kind {
                ActionKind::Feedback => {
                    let fb = A::Feedback::decode(msg.body.as_slice()).map_err(|err| {
                        BusError::Protocol(format!(
                            "action '{}' feedback decode failed: {err}",
                            self.action_name()
                        ))
                    })?;
                    feedbacks.push(fb);
                }
                ActionKind::Result => {
                    let res = A::Result::decode(msg.body.as_slice()).map_err(|err| {
                        BusError::Protocol(format!(
                            "action '{}' result decode failed: {err}",
                            self.action_name()
                        ))
                    })?;
                    result = Some(res);
                }
                ActionKind::Goal | ActionKind::Cancel => {}
            }
        }
        let result = result.ok_or_else(|| {
            BusError::Protocol(format!(
                "action '{}' completed without RESULT",
                self.action_name()
            ))
        })?;
        Ok(ActionOutcome { feedbacks, result })
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        self.inner.high_water_mark()
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        self.inner.set_high_water_mark(hwm)
    }
}
