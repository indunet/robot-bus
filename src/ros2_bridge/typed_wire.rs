//! Generic ROS↔bus wiring driven by [`TypedServiceMapper`] / [`TypedActionMapper`].

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread;
use std::time::{Duration, Instant};

use rclrs::{BeginAcceptedGoal, GoalClient};
use rosidl_runtime_rs::{Action as ActionIdl, Service as ServiceIdl};

use crate::ActionKind;
use crate::errors::{BusError, Result};
use crate::runtime::{ActionGoalHandler, ServiceHandler};
use crate::ros2_bridge::mapper::{
    ActionWireContext, Direction, ServiceWireContext, TypedActionMapper, TypedServiceMapper,
};

fn wait_service_ready(
    client_ready: impl Fn() -> bool,
    timeout: Duration,
) -> std::result::Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if client_ready() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err("timed out waiting for ROS service".into())
}

fn call_ros_service<S: ServiceIdl>(
    client: &rclrs::Client<S>,
    ros_req: S::Request,
    timeout: Duration,
) -> std::result::Result<S::Response, String>
where
    S::Request: Send + 'static,
    S::Response: Send + 'static,
{
    wait_service_ready(|| client.service_is_ready().unwrap_or(false), timeout)?;
    let (tx, rx) = mpsc::sync_channel(1);
    let _promise = client
        .call_then(ros_req, move |resp: S::Response| {
            let _ = tx.send(resp);
        })
        .map_err(|e| format!("ros service call: {e}"))?;
    match rx.recv_timeout(timeout) {
        Ok(resp) => Ok(resp),
        Err(_) => Err("timed out waiting for ROS service response".into()),
    }
}

/// Wire a service route using only [`TypedServiceMapper`] convert methods.
pub fn wire_typed_service<M>(mapper: &M, ctx: ServiceWireContext<'_>) -> Result<()>
where
    M: TypedServiceMapper,
    <M::Ros as ServiceIdl>::Request: Send + Sync + 'static,
    <M::Ros as ServiceIdl>::Response: Send + Sync + Default + 'static,
{
    let mapper = mapper.clone();
    match ctx.direction {
        Direction::Ros2ToBus => {
            let bus_client = Arc::new(Mutex::new(
                ctx.bus_node.create_client_raw(ctx.bus_service)?,
            ));
            let timeout = ctx.timeout;
            let type_name = mapper.type_name().to_string();
            let cb_mapper = mapper.clone();
            let srv = ctx
                .ros_node
                .create_service::<M::Ros, _>(
                    ctx.ros_service,
                    move |req: <M::Ros as ServiceIdl>::Request| {
                        let bus_req = match cb_mapper.ros_req_to_bus(&req) {
                            Ok(b) => b,
                            Err(e) => {
                                return cb_mapper.error_response(&format!("encode request: {e}"));
                            }
                        };
                        let guard = match bus_client.lock() {
                            Ok(g) => g,
                            Err(e) => {
                                return cb_mapper
                                    .error_response(&format!("bus client lock poisoned: {e}"));
                            }
                        };
                        match guard.call(&bus_req, Some(timeout)) {
                            Ok(bus_resp) => cb_mapper.bus_resp_to_ros(&bus_resp).unwrap_or_else(
                                |e| cb_mapper.error_response(&format!("decode: {e}")),
                            ),
                            Err(e) => cb_mapper.error_response(&format!("bus call failed: {e}")),
                        }
                    },
                )
                .map_err(|e| {
                    BusError::Protocol(format!("ros create_service {type_name}: {e}"))
                })?;
            ctx.ros_entities.push(Box::new(srv));
        }
        Direction::BusToRos2 => {
            let type_name = mapper.type_name().to_string();
            let ros_client = ctx
                .ros_node
                .create_client::<M::Ros>(ctx.ros_service)
                .map_err(|e| {
                    BusError::Protocol(format!("ros create_client {type_name}: {e}"))
                })?;
            ctx.ros_entities.push(Box::new(Arc::clone(&ros_client)));
            let timeout = ctx.timeout;
            let handler: ServiceHandler = Arc::new(move |body| {
                let ros_req = match mapper.bus_req_to_ros(body) {
                    Ok(r) => r,
                    Err(e) => {
                        return mapper
                            .ros_resp_to_bus(&mapper.error_response(&format!("decode request: {e}")))
                            .unwrap_or_default();
                    }
                };
                match call_ros_service(&ros_client, ros_req, timeout) {
                    Ok(resp) => mapper.ros_resp_to_bus(&resp).unwrap_or_default(),
                    Err(msg) => mapper
                        .ros_resp_to_bus(&mapper.error_response(&msg))
                        .unwrap_or_default(),
                }
            });
            let _ = ctx
                .bus_node
                .create_service_raw(ctx.bus_service, handler, None)?;
        }
    }
    Ok(())
}

fn noop_raw_waker() -> RawWaker {
    fn clone(_: *const ()) -> RawWaker {
        noop_raw_waker()
    }
    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}
    RawWaker::new(
        std::ptr::null(),
        &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
    )
}

fn poll_once<F: Future + Unpin>(fut: &mut F) -> Poll<F::Output> {
    let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
    let mut cx = Context::from_waker(&waker);
    Pin::new(fut).poll(&mut cx)
}

fn await_with_timeout<F: Future + Unpin>(
    mut fut: F,
    timeout: Duration,
) -> std::result::Result<F::Output, String> {
    let deadline = Instant::now() + timeout;
    loop {
        match poll_once(&mut fut) {
            Poll::Ready(v) => return Ok(v),
            Poll::Pending => {
                if Instant::now() >= deadline {
                    return Err("timed out waiting for ROS action".into());
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

#[allow(dead_code)]
fn call_ros_action<A: ActionIdl>(
    client: &rclrs::ActionClient<A>,
    ros_goal: A::Goal,
    timeout: Duration,
) -> std::result::Result<Vec<(String, Vec<u8>)>, String>
where
    A::Goal: Clone + Send + Sync + 'static,
    A::Feedback: Clone + Send + Sync + 'static,
    A::Result: Clone + Send + Sync + 'static,
{
    // Placeholder — filled by wire_typed_action with mapper converts via closure.
    let _ = (client, ros_goal, timeout);
    Err("internal: use call_ros_action_with_mapper".into())
}

/// Wire an action route using only [`TypedActionMapper`] convert methods.
pub fn wire_typed_action<M>(mapper: &M, ctx: ActionWireContext<'_>) -> Result<()>
where
    M: TypedActionMapper,
    <M::Ros as ActionIdl>::Goal: Clone + Send + Sync + 'static,
    <M::Ros as ActionIdl>::Feedback: Clone + Send + Sync + 'static,
    <M::Ros as ActionIdl>::Result: Default + Clone + Send + Sync + 'static,
{
    let mapper = mapper.clone();
    match ctx.direction {
        Direction::Ros2ToBus => {
            let bus_client = Arc::new(Mutex::new(
                ctx.bus_node.create_action_client_raw(ctx.bus_action)?,
            ));
            let timeout = ctx.timeout;
            let type_name = mapper.type_name().to_string();
            let srv = ctx
                .ros_node
                .create_action_server::<M::Ros, _>(
                    ctx.ros_action,
                    move |requested| {
                        let bus_client = Arc::clone(&bus_client);
                        let mapper = mapper.clone();
                        async move {
                            let goal = (**requested.goal()).clone();
                            let accepted = requested.accept();
                            let executing = match accepted.begin() {
                                BeginAcceptedGoal::Execute(e) => e,
                                BeginAcceptedGoal::Cancel(c) => {
                                    return c.cancelled_with(Default::default());
                                }
                            };
                            let bus_goal = match mapper.ros_goal_to_bus(&goal) {
                                Ok(b) => b,
                                Err(e) => {
                                    log::warn!("ros→bus encode goal failed: {e}");
                                    return executing.aborted_with(Default::default());
                                }
                            };
                            let call = tokio::task::spawn_blocking(move || {
                                let guard = bus_client.lock().map_err(|e| {
                                    format!("bus action client lock poisoned: {e}")
                                })?;
                                guard
                                    .send_goal_and_wait(&bus_goal, None, Some(timeout))
                                    .map_err(|e| e.to_string())
                            })
                            .await;
                            match call {
                                Ok(Ok(messages)) => {
                                    let mut result = <M::Ros as ActionIdl>::Result::default();
                                    let mut got_result = false;
                                    for msg in &messages {
                                        match msg.kind {
                                            ActionKind::Feedback => {
                                                if let Ok(fb) =
                                                    mapper.bus_feedback_to_ros(&msg.body)
                                                {
                                                    executing.publish_feedback(fb);
                                                }
                                            }
                                            ActionKind::Result => {
                                                match mapper.bus_result_to_ros(&msg.body) {
                                                    Ok(r) => {
                                                        result = r;
                                                        got_result = true;
                                                    }
                                                    Err(e) => {
                                                        log::warn!(
                                                            "ros→bus decode result failed: {e}"
                                                        );
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    if got_result {
                                        executing.succeeded_with(result)
                                    } else {
                                        executing.aborted_with(Default::default())
                                    }
                                }
                                Ok(Err(e)) => {
                                    log::warn!("ros→bus action goal failed: {e}");
                                    executing.aborted_with(Default::default())
                                }
                                Err(e) => {
                                    log::warn!("ros→bus action join failed: {e}");
                                    executing.aborted_with(Default::default())
                                }
                            }
                        }
                    },
                )
                .map_err(|e| {
                    BusError::Protocol(format!("ros create_action_server {type_name}: {e}"))
                })?;
            ctx.ros_entities.push(Box::new(srv));
        }
        Direction::BusToRos2 => {
            let type_name = mapper.type_name().to_string();
            let ros_client = ctx
                .ros_node
                .create_action_client::<M::Ros>(ctx.ros_action)
                .map_err(|e| {
                    BusError::Protocol(format!("ros create_action_client {type_name}: {e}"))
                })?;
            ctx.ros_entities.push(Box::new(Arc::clone(&ros_client)));
            let timeout = ctx.timeout;
            let handler: ActionGoalHandler = Arc::new(move |body| {
                let ros_goal = match mapper.bus_goal_to_ros(body) {
                    Ok(g) => g,
                    Err(e) => {
                        log::warn!("decode action goal: {e}");
                        return vec![(
                            "RESULT".into(),
                            mapper
                                .ros_result_to_bus(&Default::default())
                                .unwrap_or_default(),
                        )];
                    }
                };
                match call_ros_action_mapped(&ros_client, &mapper, ros_goal, timeout) {
                    Ok(replies) => replies,
                    Err(msg) => {
                        log::warn!("bus→ros action failed: {msg}");
                        vec![(
                            "RESULT".into(),
                            mapper
                                .ros_result_to_bus(&Default::default())
                                .unwrap_or_default(),
                        )]
                    }
                }
            });
            let _ = ctx
                .bus_node
                .create_action_server_raw(ctx.bus_action, handler, None)?;
        }
    }
    Ok(())
}

fn call_ros_action_mapped<M: TypedActionMapper>(
    client: &rclrs::ActionClient<M::Ros>,
    mapper: &M,
    ros_goal: <M::Ros as ActionIdl>::Goal,
    timeout: Duration,
) -> std::result::Result<Vec<(String, Vec<u8>)>, String>
where
    <M::Ros as ActionIdl>::Goal: Clone + Send + Sync + 'static,
    <M::Ros as ActionIdl>::Feedback: Clone + Send + Sync + 'static,
    <M::Ros as ActionIdl>::Result: Clone + Send + Sync + 'static,
{
    let requested = client
        .try_request_goal(ros_goal)
        .map_err(|e| format!("ros action request_goal: {e}"))?;
    let goal_client = match await_with_timeout(requested, timeout)? {
        Some(gc) => gc,
        None => return Err("ROS action server rejected goal".into()),
    };
    let GoalClient {
        mut feedback,
        result,
        ..
    } = goal_client;
    let mut replies = Vec::new();
    let deadline = Instant::now() + timeout;
    let mut result_fut = result;
    loop {
        while let Ok(fb) = feedback.try_recv() {
            let bus_fb = mapper
                .ros_feedback_to_bus(&fb)
                .map_err(|e| e.to_string())?;
            replies.push(("FEEDBACK".into(), bus_fb));
        }
        match poll_once(&mut result_fut) {
            Poll::Ready((_status, res)) => {
                let bus_res = mapper
                    .ros_result_to_bus(&res)
                    .map_err(|e| e.to_string())?;
                replies.push(("RESULT".into(), bus_res));
                return Ok(replies);
            }
            Poll::Pending => {
                if Instant::now() >= deadline {
                    return Err("timed out waiting for ROS action result".into());
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

/// Keep `ros_entities` type in scope for docs / unused import silence.
#[allow(dead_code)]
fn _entity_slot(_: &mut Vec<Box<dyn Any + Send + Sync>>) {}
