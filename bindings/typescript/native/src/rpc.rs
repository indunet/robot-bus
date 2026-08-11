//! Service and action client bindings.

use std::sync::Arc;
use std::time::Duration;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{
    ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use napi_derive::napi;
use robot_bus::runtime::{
    NodeActionClientRaw as RustNodeActionClient, NodeServiceClientRaw as RustNodeServiceClient,
    RawActionFeedbackCallback, RawGoalHandle as RustRawGoalHandle,
};

use crate::util::{action_kind_str, bus_err};

#[napi(object)]
pub struct ActionEvent {
    pub kind: String,
    pub body: Buffer,
    pub goal_id: String,
    pub action_name: String,
}

#[napi(object)]
pub struct ActionPhaseReply {
    pub phase: String,
    pub body: Buffer,
}

pub(crate) fn action_event(message: robot_bus::action_bus::ActionMessage) -> ActionEvent {
    ActionEvent {
        kind: action_kind_str(message.kind).to_string(),
        body: Buffer::from(message.body),
        goal_id: message.goal_id,
        action_name: message.action_name,
    }
}

#[napi]
pub struct ServiceClient {
    pub(crate) inner: RustNodeServiceClient,
}

#[napi]
impl ServiceClient {
    #[napi(getter)]
    pub fn service_name(&self) -> String {
        self.inner.service_name().to_string()
    }

    /// Call the bound service. `timeout` is seconds; omit to wait indefinitely.
    #[napi]
    pub fn call(&self, body: Buffer, timeout: Option<f64>) -> Result<Buffer> {
        let timeout = timeout.map(Duration::from_secs_f64);
        let reply = self.inner.call(&body, timeout).map_err(bus_err)?;
        Ok(Buffer::from(reply))
    }
}

#[napi]
pub struct ActionClient {
    pub(crate) inner: RustNodeActionClient,
}

#[napi]
pub struct GoalHandle {
    pub(crate) inner: RustRawGoalHandle,
}

#[napi(object)]
pub struct SendGoalOptions {
    pub goal_id: Option<String>,
    pub timeout_seconds: Option<f64>,
    pub on_feedback: Option<JsFunction>,
}

pub struct ActionResultTask {
    handle: RustRawGoalHandle,
}

impl Task for ActionResultTask {
    type Output = ActionEvent;
    type JsValue = ActionEvent;

    fn compute(&mut self) -> Result<Self::Output> {
        self.handle.wait_result().map(action_event).map_err(bus_err)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi]
impl GoalHandle {
    #[napi(getter)]
    pub fn goal_id(&self) -> String {
        self.inner.goal_id().to_string()
    }

    #[napi(getter)]
    pub fn action_name(&self) -> String {
        self.inner.action_name().to_string()
    }

    /// Resolve with the terminal RESULT without blocking the Node.js event loop.
    #[napi]
    pub fn result(&self) -> AsyncTask<ActionResultTask> {
        AsyncTask::new(ActionResultTask {
            handle: self.inner.clone(),
        })
    }

    /// Best-effort cancellation. An optional payload is sent on ZMQ transports.
    #[napi]
    pub fn cancel(&self, body: Option<Buffer>) -> Result<()> {
        match body {
            Some(body) => self.inner.cancel_with_body(body.as_ref()),
            None => self.inner.cancel(),
        }
        .map_err(bus_err)
    }
}

#[napi]
impl ActionClient {
    #[napi(getter)]
    pub fn action_name(&self) -> String {
        self.inner.action_name().to_string()
    }

    #[napi]
    pub fn send_goal(
        &self,
        body: Buffer,
        options: Option<SendGoalOptions>,
    ) -> Result<GoalHandle> {
        let options = options.unwrap_or(SendGoalOptions {
            goal_id: None,
            timeout_seconds: None,
            on_feedback: None,
        });
        let timeout = options.timeout_seconds.map(Duration::from_secs_f64);
        let goal_id = options.goal_id;
        let feedback_callback = options.on_feedback;
        let feedback_callback = feedback_callback
            .map(|callback| {
                let tsfn: ActionFeedbackTsfn =
                    callback.create_threadsafe_function(0, |ctx: ThreadSafeCallContext<(
                        String,
                        Vec<u8>,
                        String,
                        String,
                    )>| {
                        let (kind, body, goal_id, action_name) = ctx.value;
                        let mut event = ctx.env.create_object()?;
                        event.set_named_property("kind", ctx.env.create_string(&kind)?)?;
                        event.set_named_property(
                            "body",
                            ctx.env.create_buffer_with_data(body)?.into_unknown(),
                        )?;
                        event.set_named_property(
                            "goalId",
                            ctx.env.create_string(&goal_id)?,
                        )?;
                        event.set_named_property(
                            "actionName",
                            ctx.env.create_string(&action_name)?,
                        )?;
                        Ok(vec![event.into_unknown()])
                    })?;
                let tsfn = Arc::new(tsfn);
                Ok::<RawActionFeedbackCallback, Error>(
                    Arc::new(move |message: &robot_bus::action_bus::ActionMessage| {
                        let _ = tsfn.call(
                            (
                                action_kind_str(message.kind).to_string(),
                                message.body.clone(),
                                message.goal_id.clone(),
                                message.action_name.clone(),
                            ),
                            ThreadsafeFunctionCallMode::NonBlocking,
                        );
                    }) as RawActionFeedbackCallback,
                )
            })
            .transpose()?;
        let handle = self
            .inner
            .send_goal(&body, goal_id.as_deref(), timeout, feedback_callback)
            .map_err(bus_err)?;
        Ok(GoalHandle { inner: handle })
    }
}

type ActionFeedbackTsfn =
    ThreadsafeFunction<(String, Vec<u8>, String, String), ErrorStrategy::Fatal>;
