//! Library-owned typed ROS↔bus service/action wiring (Track A).
//!
//! Builtins dispatch here by `type_name`. Arbitrary custom codecs without an
//! `attach` override need dynamic service/action support (Track B).

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread;
use std::time::{Duration, Instant};

use prost::Message as ProstMessage;
use rclrs::vendor::example_interfaces::action as ros_act;
use rclrs::{BeginAcceptedGoal, GoalClient};

use crate::action::v1::{
    Fibonacci as BusFibonacci, FibonacciGoal as BusFibonacciGoal,
    FibonacciResult as BusFibonacciResult,
};
use crate::errors::{BusError, Result};
use crate::runtime::{ActionGoalHandler, ServiceHandler};
use crate::ros2_bridge::mapper::{ActionWireContext, Direction, ServiceWireContext};
use crate::ros2_bridge::mappers::action;
use crate::ros2_bridge::mappers::service;
use crate::ros2_bridge::vendor::std_srvs::srv as ros_srv;
use crate::std_srvs::srv::v1::{
    SetBool as BusSetBool, SetBoolRequest as BusSetBoolRequest,
    SetBoolResponse as BusSetBoolResponse, Trigger as BusTrigger,
    TriggerRequest as BusTriggerRequest, TriggerResponse as BusTriggerResponse,
};

/// Dispatch builtin service backends by ROS type string.
pub fn attach_builtin_service(type_name: &str, ctx: ServiceWireContext<'_>) -> Result<()> {
    match type_name {
        "std_srvs/srv/Trigger" => attach_trigger(ctx),
        "std_srvs/srv/SetBool" => attach_set_bool(ctx),
        other => Err(BusError::Protocol(format!(
            "no typed service backend for {other:?}; \
             builtins: std_srvs/srv/Trigger, std_srvs/srv/SetBool; \
             override ServiceMapper::attach for a Rust typed backend, \
             or wait for dynamic service support (Track B) for arbitrary codecs"
        ))),
    }
}

/// Dispatch builtin action backends by ROS type string.
pub fn attach_builtin_action(type_name: &str, ctx: ActionWireContext<'_>) -> Result<()> {
    match type_name {
        "example_interfaces/action/Fibonacci" => attach_fibonacci(ctx),
        other => Err(BusError::Protocol(format!(
            "no typed action backend for {other:?}; \
             builtin: example_interfaces/action/Fibonacci; \
             override ActionMapper::attach for a Rust typed backend, \
             or wait for dynamic action support (Track B) for arbitrary codecs"
        ))),
    }
}

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

fn call_ros_trigger(
    client: &rclrs::Client<ros_srv::Trigger>,
    bus_req: &BusTriggerRequest,
    timeout: Duration,
) -> std::result::Result<BusTriggerResponse, String> {
    wait_service_ready(|| client.service_is_ready().unwrap_or(false), timeout)?;
    let ros_req = service::trigger_bus_req_to_ros(bus_req);
    let (tx, rx) = mpsc::sync_channel(1);
    let _promise = client
        .call_then(ros_req, move |resp: ros_srv::Trigger_Response| {
            let _ = tx.send(resp);
        })
        .map_err(|e| format!("ros Trigger call: {e}"))?;
    match rx.recv_timeout(timeout) {
        Ok(resp) => Ok(service::trigger_ros_resp_to_bus(&resp)),
        Err(_) => Err("timed out waiting for ROS Trigger response".into()),
    }
}

fn call_ros_set_bool(
    client: &rclrs::Client<ros_srv::SetBool>,
    bus_req: &BusSetBoolRequest,
    timeout: Duration,
) -> std::result::Result<BusSetBoolResponse, String> {
    wait_service_ready(|| client.service_is_ready().unwrap_or(false), timeout)?;
    let ros_req = service::set_bool_bus_req_to_ros(bus_req);
    let (tx, rx) = mpsc::sync_channel(1);
    let _promise = client
        .call_then(ros_req, move |resp: ros_srv::SetBool_Response| {
            let _ = tx.send(resp);
        })
        .map_err(|e| format!("ros SetBool call: {e}"))?;
    match rx.recv_timeout(timeout) {
        Ok(resp) => Ok(service::set_bool_ros_resp_to_bus(&resp)),
        Err(_) => Err("timed out waiting for ROS SetBool response".into()),
    }
}

/// Wire `std_srvs/srv/Trigger` (library-owned typed path).
pub fn attach_trigger(ctx: ServiceWireContext<'_>) -> Result<()> {
    match ctx.direction {
        Direction::Ros2ToBus => {
            let bus_client = Arc::new(Mutex::new(
                ctx.bus_node.create_client::<BusTrigger>(ctx.bus_service)?,
            ));
            let timeout = ctx.timeout;
            let srv = ctx
                .ros_node
                .create_service::<ros_srv::Trigger, _>(
                    ctx.ros_service,
                    move |_req: ros_srv::Trigger_Request| {
                        let bus_req = service::trigger_ros_req_to_bus(&_req);
                        let guard = match bus_client.lock() {
                            Ok(g) => g,
                            Err(e) => {
                                return ros_srv::Trigger_Response {
                                    success: false,
                                    message: format!("bus client lock poisoned: {e}"),
                                };
                            }
                        };
                        match guard.call(&bus_req, Some(timeout)) {
                            Ok(bus_resp) => service::trigger_bus_resp_to_ros(&bus_resp),
                            Err(e) => ros_srv::Trigger_Response {
                                success: false,
                                message: format!("bus call failed: {e}"),
                            },
                        }
                    },
                )
                .map_err(|e| BusError::Protocol(format!("ros create_service Trigger: {e}")))?;
            ctx.ros_entities.push(Box::new(srv));
        }
        Direction::BusToRos2 => {
            let ros_client = ctx
                .ros_node
                .create_client::<ros_srv::Trigger>(ctx.ros_service)
                .map_err(|e| BusError::Protocol(format!("ros create_client Trigger: {e}")))?;
            ctx.ros_entities.push(Box::new(Arc::clone(&ros_client)));
            let timeout = ctx.timeout;
            let handler: ServiceHandler = Arc::new(move |body| {
                let bus_req = match BusTriggerRequest::decode(body) {
                    Ok(r) => r,
                    Err(e) => {
                        return BusTriggerResponse {
                            success: false,
                            message: format!("decode TriggerRequest: {e}"),
                        }
                        .encode_to_vec();
                    }
                };
                match call_ros_trigger(&ros_client, &bus_req, timeout) {
                    Ok(resp) => resp.encode_to_vec(),
                    Err(msg) => BusTriggerResponse {
                        success: false,
                        message: msg,
                    }
                    .encode_to_vec(),
                }
            });
            let _ = ctx
                .bus_node
                .create_service_raw(ctx.bus_service, handler, None)?;
        }
    }
    Ok(())
}

/// Wire `std_srvs/srv/SetBool` (library-owned typed path).
pub fn attach_set_bool(ctx: ServiceWireContext<'_>) -> Result<()> {
    match ctx.direction {
        Direction::Ros2ToBus => {
            let bus_client = Arc::new(Mutex::new(
                ctx.bus_node.create_client::<BusSetBool>(ctx.bus_service)?,
            ));
            let timeout = ctx.timeout;
            let srv = ctx
                .ros_node
                .create_service::<ros_srv::SetBool, _>(
                    ctx.ros_service,
                    move |req: ros_srv::SetBool_Request| {
                        let bus_req = service::set_bool_ros_req_to_bus(&req);
                        let guard = match bus_client.lock() {
                            Ok(g) => g,
                            Err(e) => {
                                return ros_srv::SetBool_Response {
                                    success: false,
                                    message: format!("bus client lock poisoned: {e}"),
                                };
                            }
                        };
                        match guard.call(&bus_req, Some(timeout)) {
                            Ok(bus_resp) => service::set_bool_bus_resp_to_ros(&bus_resp),
                            Err(e) => ros_srv::SetBool_Response {
                                success: false,
                                message: format!("bus call failed: {e}"),
                            },
                        }
                    },
                )
                .map_err(|e| BusError::Protocol(format!("ros create_service SetBool: {e}")))?;
            ctx.ros_entities.push(Box::new(srv));
        }
        Direction::BusToRos2 => {
            let ros_client = ctx
                .ros_node
                .create_client::<ros_srv::SetBool>(ctx.ros_service)
                .map_err(|e| BusError::Protocol(format!("ros create_client SetBool: {e}")))?;
            ctx.ros_entities.push(Box::new(Arc::clone(&ros_client)));
            let timeout = ctx.timeout;
            let handler: ServiceHandler = Arc::new(move |body| {
                let bus_req = match BusSetBoolRequest::decode(body) {
                    Ok(r) => r,
                    Err(e) => {
                        return BusSetBoolResponse {
                            success: false,
                            message: format!("decode SetBoolRequest: {e}"),
                        }
                        .encode_to_vec();
                    }
                };
                match call_ros_set_bool(&ros_client, &bus_req, timeout) {
                    Ok(resp) => resp.encode_to_vec(),
                    Err(msg) => BusSetBoolResponse {
                        success: false,
                        message: msg,
                    }
                    .encode_to_vec(),
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

fn call_ros_fibonacci(
    client: &rclrs::ActionClient<ros_act::Fibonacci>,
    bus_goal: &BusFibonacciGoal,
    timeout: Duration,
) -> std::result::Result<Vec<(String, Vec<u8>)>, String> {
    let ros_goal = action::fibonacci_bus_goal_to_ros(bus_goal);
    let requested = client
        .try_request_goal(ros_goal)
        .map_err(|e| format!("ros Fibonacci request_goal: {e}"))?;
    let goal_client = match await_with_timeout(requested, timeout)? {
        Some(gc) => gc,
        None => return Err("ROS action server rejected Fibonacci goal".into()),
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
            let bus_fb = action::fibonacci_ros_feedback_to_bus(&fb);
            replies.push(("FEEDBACK".into(), bus_fb.encode_to_vec()));
        }
        match poll_once(&mut result_fut) {
            Poll::Ready((_status, res)) => {
                let bus_res = action::fibonacci_ros_result_to_bus(&res);
                replies.push(("RESULT".into(), bus_res.encode_to_vec()));
                return Ok(replies);
            }
            Poll::Pending => {
                if Instant::now() >= deadline {
                    return Err("timed out waiting for ROS Fibonacci result".into());
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

/// Wire `example_interfaces/action/Fibonacci` (library-owned typed path).
pub fn attach_fibonacci(ctx: ActionWireContext<'_>) -> Result<()> {
    match ctx.direction {
        Direction::Ros2ToBus => {
            let bus_client = Arc::new(Mutex::new(
                ctx.bus_node
                    .create_action_client::<BusFibonacci>(ctx.bus_action)?,
            ));
            let timeout = ctx.timeout;
            let srv = ctx
                .ros_node
                .create_action_server::<ros_act::Fibonacci, _>(
                    ctx.ros_action,
                    move |requested| {
                        let bus_client = Arc::clone(&bus_client);
                        async move {
                            let goal = (**requested.goal()).clone();
                            let accepted = requested.accept();
                            let executing = match accepted.begin() {
                                BeginAcceptedGoal::Execute(e) => e,
                                BeginAcceptedGoal::Cancel(c) => {
                                    return c.cancelled_with(ros_act::Fibonacci_Result {
                                        sequence: Vec::new(),
                                    });
                                }
                            };
                            let bus_goal = action::fibonacci_ros_goal_to_bus(&goal);
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
                                Ok(Ok(outcome)) => {
                                    for fb in &outcome.feedbacks {
                                        executing.publish_feedback(
                                            action::fibonacci_bus_feedback_to_ros(fb),
                                        );
                                    }
                                    executing.succeeded_with(action::fibonacci_bus_result_to_ros(
                                        &outcome.result,
                                    ))
                                }
                                Ok(Err(e)) => {
                                    log::warn!("ros→bus Fibonacci goal failed: {e}");
                                    executing.aborted_with(ros_act::Fibonacci_Result {
                                        sequence: Vec::new(),
                                    })
                                }
                                Err(e) => {
                                    log::warn!("ros→bus Fibonacci join failed: {e}");
                                    executing.aborted_with(ros_act::Fibonacci_Result {
                                        sequence: Vec::new(),
                                    })
                                }
                            }
                        }
                    },
                )
                .map_err(|e| {
                    BusError::Protocol(format!("ros create_action_server Fibonacci: {e}"))
                })?;
            ctx.ros_entities.push(Box::new(srv));
        }
        Direction::BusToRos2 => {
            let ros_client = ctx
                .ros_node
                .create_action_client::<ros_act::Fibonacci>(ctx.ros_action)
                .map_err(|e| {
                    BusError::Protocol(format!("ros create_action_client Fibonacci: {e}"))
                })?;
            ctx.ros_entities.push(Box::new(Arc::clone(&ros_client)));
            let timeout = ctx.timeout;
            let handler: ActionGoalHandler = Arc::new(move |body| {
                let bus_goal = match BusFibonacciGoal::decode(body) {
                    Ok(g) => g,
                    Err(e) => {
                        log::warn!("decode FibonacciGoal: {e}");
                        return vec![(
                            "RESULT".into(),
                            BusFibonacciResult {
                                sequence: Vec::new(),
                            }
                            .encode_to_vec(),
                        )];
                    }
                };
                match call_ros_fibonacci(&ros_client, &bus_goal, timeout) {
                    Ok(replies) => replies,
                    Err(msg) => {
                        log::warn!("bus→ros Fibonacci failed: {msg}");
                        vec![(
                            "RESULT".into(),
                            BusFibonacciResult {
                                sequence: Vec::new(),
                            }
                            .encode_to_vec(),
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

/// Keep `ros_entities` type in scope for docs.
#[allow(dead_code)]
fn _entity_slot(_: &mut Vec<Box<dyn Any + Send + Sync>>) {}
