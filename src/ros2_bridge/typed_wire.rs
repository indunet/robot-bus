//! Generic ROS↔bus wiring driven by [`TypedServiceMapper`] / [`TypedActionMapper`].

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread;
use std::time::{Duration, Instant};

use rclrs::{BeginAcceptedGoal, GoalClient, IntoActionClientOptions, IntoActionServerOptions};
use rosidl_runtime_rs::{Action as ActionIdl, Service as ServiceIdl};

use crate::errors::{BusError, Result};
use crate::ros2_bridge::mapper::{
    ros_action_feedback_qos_profile, ros_service_qos_profile, ros_topic_options, ActionWireContext,
    Direction, ServiceWireContext, TopicQos, TopicWireContext, TypedActionMapper,
    TypedServiceMapper, TypedTopicMapper,
};
use crate::action_bus::ActionMessage;
use crate::runtime::{
    ActionGoalLiveHandler, MessageCallback, QosProfile, RawActionFeedbackCallback, ServiceHandler,
    TopicPublisherRaw,
};
use crate::ActionKind;

/// Typed ROS→bus subscription: `create_subscription<Ros>` then convert + publish.
pub fn create_typed_ros2_to_bus_sub<M>(
    mapper: &M,
    ros_node: &rclrs::Node,
    bus_pub: TopicPublisherRaw,
    ros_topic: &str,
    qos: TopicQos,
) -> Result<Box<dyn Any + Send + Sync>>
where
    M: TypedTopicMapper,
{
    use prost::Message as _;

    let mapper = mapper.clone();
    let type_name = mapper.type_name().to_string();
    let opts = ros_topic_options(ros_topic, qos);
    let type_name_cb = type_name.clone();
    let sub = ros_node
        .create_subscription(opts, move |msg: M::Ros| {
            let payload = match mapper.ros_to_bus(msg) {
                Ok(bus) => bus.encode_to_vec(),
                Err(e) => {
                    log::warn!("ros→bus {type_name_cb} convert: {e}");
                    return;
                }
            };
            if let Err(e) = bus_pub.publish(&payload) {
                log::warn!("ros→bus {type_name_cb} publish: {e}");
            }
        })
        .map_err(|e| BusError::Protocol(format!("ros typed subscription {type_name}: {e}")))?;
    Ok(Box::new(sub))
}

/// Typed bus→ROS: `create_publisher<Ros>` plus a bus raw subscription.
pub fn attach_typed_bus_to_ros<M>(mapper: &M, ctx: TopicWireContext<'_>) -> Result<()>
where
    M: TypedTopicMapper,
{
    let mapper = mapper.clone();
    let type_name = mapper.type_name().to_string();
    let opts = ros_topic_options(ctx.ros_topic, ctx.ros_qos);
    let ros_pub = ctx
        .ros_node
        .create_publisher::<M::Ros>(opts)
        .map_err(|e| BusError::Protocol(format!("ros typed publisher {type_name}: {e}")))?;
    let ros_pub_cb = ros_pub.clone();
    ctx.ros_entities.push(Box::new(ros_pub));
    let cb: MessageCallback = Arc::new(move |payload| {
        use prost::Message as _;
        let bus = match M::Bus::decode(payload) {
            Ok(b) => b,
            Err(e) => {
                log::warn!("bus→ros {type_name} decode: {e}");
                return;
            }
        };
        match mapper.bus_to_ros(bus) {
            Ok(ros_msg) => {
                if let Err(e) = ros_pub_cb.publish(ros_msg) {
                    log::warn!("bus→ros {type_name} publish: {e}");
                }
            }
            Err(e) => log::warn!("bus→ros {type_name} convert: {e}"),
        }
    });
        ctx.bus_node.create_subscription_raw_with_qos(
            ctx.bus_topic,
            QosProfile::keep_last(ctx.bus_qos.depth()),
            cb,
            None,
        )?;
    Ok(())
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
            let bus_client = Arc::new(Mutex::new(ctx.bus_node.create_client_raw_with_qos(
                ctx.bus_service,
                QosProfile::keep_last(ctx.bus_qos.depth()),
            )?));
            let timeout = ctx.timeout;
            let type_name = mapper.type_name().to_string();
            let cb_mapper = mapper.clone();
            let srv = ctx
                .ros_node
                .create_service::<M::Ros, _>(
                    ros_topic_options(ctx.ros_service, ctx.ros_qos),
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
                            Ok(bus_resp) => {
                                cb_mapper.bus_resp_to_ros(&bus_resp).unwrap_or_else(|e| {
                                    cb_mapper.error_response(&format!("decode: {e}"))
                                })
                            }
                            Err(e) => cb_mapper.error_response(&format!("bus call failed: {e}")),
                        }
                    },
                )
                .map_err(|e| BusError::Protocol(format!("ros create_service {type_name}: {e}")))?;
            ctx.ros_entities.push(Box::new(srv));
        }
        Direction::BusToRos2 => {
            let type_name = mapper.type_name().to_string();
            let ros_client = ctx
                .ros_node
                .create_client::<M::Ros>(ros_topic_options(ctx.ros_service, ctx.ros_qos))
                .map_err(|e| BusError::Protocol(format!("ros create_client {type_name}: {e}")))?;
            ctx.ros_entities.push(Box::new(Arc::clone(&ros_client)));
            let timeout = ctx.timeout;
            let handler: ServiceHandler = Arc::new(move |body| {
                let ros_req = match mapper.bus_req_to_ros(body) {
                    Ok(r) => r,
                    Err(e) => {
                        return mapper
                            .ros_resp_to_bus(
                                &mapper.error_response(&format!("decode request: {e}")),
                            )
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
            let _ = ctx.bus_node.create_service_raw_with_qos(
                ctx.bus_service,
                QosProfile::keep_last(ctx.bus_qos.depth()),
                handler,
                None,
            )?;
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
                ctx.bus_node.create_action_client_raw_with_qos(
                    ctx.bus_action,
                    QosProfile::keep_last(ctx.bus_qos.depth()),
                )?,
            ));
            let timeout = ctx.timeout;
            let type_name = mapper.type_name().to_string();
            let srv_qos = ros_service_qos_profile(ctx.ros_qos);
            let fb_qos = ros_action_feedback_qos_profile(ctx.ros_qos);
            let srv = ctx
                .ros_node
                .create_action_server::<M::Ros, _>(
                    IntoActionServerOptions::goal_service_qos(ctx.ros_action, srv_qos)
                        .result_service_qos(srv_qos)
                        .cancel_service_qos(srv_qos)
                        .feedback_topic_qos(fb_qos),
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
                        let fb_pub = executing.feedback_publisher();
                        let fb_mapper = mapper.clone();
                        let feedback_cb: RawActionFeedbackCallback = Arc::new(move |msg: &ActionMessage| {
                            if msg.kind != ActionKind::Feedback {
                                return;
                            }
                            match fb_mapper.bus_feedback_to_ros(&msg.body) {
                                Ok(fb) => {
                                    let _ = fb_pub.publish(fb);
                                }
                                Err(e) => log::warn!("ros→bus decode feedback failed: {e}"),
                            }
                        });
                        let bus_handle = {
                            let guard = match bus_client.lock() {
                                Ok(g) => g,
                                Err(e) => {
                                    log::warn!("ros→bus action client lock poisoned: {e}");
                                    return executing.aborted_with(Default::default());
                                }
                            };
                            match guard.send_goal(&bus_goal, None, Some(timeout), Some(feedback_cb))
                            {
                                Ok(h) => h,
                                Err(e) => {
                                    log::warn!("ros→bus send_goal failed: {e}");
                                    return executing.aborted_with(Default::default());
                                }
                            }
                        };
                        let wait_handle = bus_handle.clone();
                        let wait_fut =
                            tokio::task::spawn_blocking(move || wait_handle.wait_result());
                        match executing.unless_cancel_requested(wait_fut).await {
                            Ok(Ok(Ok(result_msg))) => {
                                match mapper.bus_result_to_ros(&result_msg.body) {
                                    Ok(result) => executing.succeeded_with(result),
                                    Err(e) => {
                                        log::warn!("ros→bus decode result failed: {e}");
                                        executing.aborted_with(Default::default())
                                    }
                                }
                            }
                            Ok(Ok(Err(e))) => {
                                log::warn!("ros→bus action goal failed: {e}");
                                executing.aborted_with(Default::default())
                            }
                            Ok(Err(e)) => {
                                log::warn!("ros→bus action join failed: {e}");
                                executing.aborted_with(Default::default())
                            }
                            Err(()) => {
                                let _ = bus_handle.cancel();
                                executing
                                    .begin_cancelling()
                                    .cancelled_with(Default::default())
                            }
                        }
                    }
                })
                .map_err(|e| {
                    BusError::Protocol(format!("ros create_action_server {type_name}: {e}"))
                })?;
            ctx.ros_entities.push(Box::new(srv));
        }
        Direction::BusToRos2 => {
            let type_name = mapper.type_name().to_string();
            let srv_qos = ros_service_qos_profile(ctx.ros_qos);
            let fb_qos = ros_action_feedback_qos_profile(ctx.ros_qos);
            let ros_client = ctx
                .ros_node
                .create_action_client::<M::Ros>(
                    IntoActionClientOptions::goal_service_qos(ctx.ros_action, srv_qos)
                        .result_service_qos(srv_qos)
                        .cancel_service_qos(srv_qos)
                        .feedback_topic_qos(fb_qos),
                )
                .map_err(|e| {
                    BusError::Protocol(format!("ros create_action_client {type_name}: {e}"))
                })?;
            ctx.ros_entities.push(Box::new(Arc::clone(&ros_client)));
            let timeout = ctx.timeout;
            let handler: ActionGoalLiveHandler = Arc::new(move |body, ctx| {
                let ros_goal = match mapper.bus_goal_to_ros(body) {
                    Ok(g) => g,
                    Err(e) => {
                        log::warn!("decode action goal: {e}");
                        return mapper
                            .ros_result_to_bus(&Default::default())
                            .unwrap_or_default();
                    }
                };
                match call_ros_action_mapped_live(&ros_client, &mapper, ros_goal, timeout, ctx) {
                    Ok(result) => result,
                    Err(msg) => {
                        log::warn!("bus→ros action failed: {msg}");
                        mapper
                            .ros_result_to_bus(&Default::default())
                            .unwrap_or_default()
                    }
                }
            });
            let _ = ctx.bus_node.create_action_server_raw_live_with_qos(
                ctx.bus_action,
                QosProfile::keep_last(ctx.bus_qos.depth()),
                handler,
                None,
            )?;
        }
    }
    Ok(())
}

fn call_ros_action_mapped_live<M: TypedActionMapper>(
    client: &rclrs::ActionClient<M::Ros>,
    mapper: &M,
    ros_goal: <M::Ros as ActionIdl>::Goal,
    timeout: Duration,
    ctx: &crate::runtime::ActionGoalContext,
) -> std::result::Result<Vec<u8>, String>
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
        cancellation,
        ..
    } = goal_client;
    let deadline = Instant::now() + timeout;
    let mut result_fut = result;
    let mut cancel_fut = None;
    loop {
        if ctx.cancel_requested() && cancel_fut.is_none() {
            cancel_fut = Some(cancellation.cancel());
        }
        if let Some(fut) = cancel_fut.as_mut() {
            let _ = poll_once(fut);
        }
        while let Ok(fb) = feedback.try_recv() {
            let bus_fb = mapper.ros_feedback_to_bus(&fb).map_err(|e| e.to_string())?;
            ctx.publish_feedback(&bus_fb);
        }
        match poll_once(&mut result_fut) {
            Poll::Ready((_status, res)) => {
                return mapper.ros_result_to_bus(&res).map_err(|e| e.to_string());
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
