//! Builtin [`ActionMapper`] implementation (Fibonacci).

use std::future::Future;
use std::pin::Pin;
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
use crate::runtime::ActionGoalHandler;
use crate::ros2_bridge::mapper::{ActionMapper, ActionWireContext, Direction};

use super::action;

pub struct FibonacciActionMapper;

pub fn lookup_action_mapper(type_name: &str) -> Result<Arc<dyn ActionMapper>> {
    match type_name {
        "example_interfaces/action/Fibonacci" => Ok(Arc::new(FibonacciActionMapper)),
        other => Err(BusError::Protocol(format!(
            "unsupported ros2 bridge action type {other:?}; \
             builtin: example_interfaces/action/Fibonacci; \
             for custom types use .mapper(...) on the action route"
        ))),
    }
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

impl ActionMapper for FibonacciActionMapper {
    fn type_name(&self) -> &'static str {
        "example_interfaces/action/Fibonacci"
    }

    fn wire(&self, ctx: ActionWireContext<'_>) -> Result<()> {
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
                                        executing.succeeded_with(
                                            action::fibonacci_bus_result_to_ros(&outcome.result),
                                        )
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
}
