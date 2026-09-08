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

use crate::ActionKind;
use crate::action_bus::ActionMessage;
use crate::errors::{BusError, Result, rpc_error_body};
use crate::ros2_bridge::deadline::Deadline;
use crate::ros2_bridge::drop_stats::{DropStats, RouteHealth};
use crate::ros2_bridge::mapper::{
    ActionWireContext, Direction, ServiceWireContext, TopicQos, TopicWireContext,
    TypedActionMapper, TypedServiceMapper, TypedTopicMapper, ros_action_feedback_qos_profile,
    ros_service_qos_profile, ros_topic_options,
};
use crate::runtime::CallbackGroup;
use crate::runtime::{
    ActionGoalLiveHandler, MessageCallback, QosProfile, RawActionFeedbackCallback, ServiceHandler,
    TopicPublisherRaw,
};

/// Typed ROS→bus subscription: `create_subscription<Ros>` then convert + publish.
pub fn create_typed_ros2_to_bus_sub<M>(
    mapper: &M,
    ros_node: &rclrs::Node,
    bus_pub: TopicPublisherRaw,
    ros_topic: &str,
    qos: TopicQos,
    drop_stats: Arc<DropStats>,
    route_health: Arc<RouteHealth>,
) -> Result<Box<dyn Any + Send + Sync>>
where
    M: TypedTopicMapper,
{
    use prost::Message as _;

    let mapper = mapper.clone();
    let topic = ros_topic.to_string();
    let opts = ros_topic_options(ros_topic, qos);
    let topic_cb = topic.clone();
    let sub = ros_node
        .create_subscription(opts, move |msg: M::Ros| {
            route_health.record_rx();
            let payload = match mapper.ros_to_bus(msg) {
                Ok(bus) => bus.encode_to_vec(),
                Err(e) => {
                    drop_stats.record_convert_fail();
                    route_health.record_convert_fail();
                    if route_health.should_log_warn() {
                        log::warn!("ros→bus {topic_cb} convert: {e}");
                    }
                    return;
                }
            };
            if let Err(e) = bus_pub.publish(&payload) {
                drop_stats.record_publish_fail();
                route_health.record_publish_fail();
                if route_health.should_log_warn() {
                    log::warn!("ros→bus {topic_cb} publish: {e}");
                }
            } else {
                route_health.record_tx();
            }
        })
        .map_err(|e| BusError::Protocol(format!("ros typed subscription {topic}: {e}")))?;
    Ok(Box::new(sub))
}

/// Typed bus→ROS: `create_publisher<Ros>` plus a bus raw subscription.
pub fn attach_typed_bus_to_ros<M>(mapper: &M, ctx: TopicWireContext<'_>) -> Result<()>
where
    M: TypedTopicMapper,
{
    let mapper = mapper.clone();
    let topic = ctx.ros_topic.to_string();
    let opts = ros_topic_options(ctx.ros_topic, ctx.ros_qos);
    let ros_pub = ctx
        .ros_node
        .create_publisher::<M::Ros>(opts)
        .map_err(|e| BusError::Protocol(format!("ros typed publisher {topic}: {e}")))?;
    let ros_pub_cb = ros_pub.clone();
    ctx.ros_entities.push(Box::new(ros_pub));
    let drop_stats = Arc::clone(&ctx.drop_stats);
    let route_health = Arc::clone(&ctx.route_health);
    let cb: MessageCallback = Arc::new(move |payload| {
        use prost::Message as _;
        route_health.record_rx();
        let bus = match M::Bus::decode(payload) {
            Ok(b) => b,
            Err(e) => {
                drop_stats.record_decode_fail();
                route_health.record_decode_fail();
                if route_health.should_log_warn() {
                    log::warn!("bus→ros {topic} decode: {e}");
                }
                return;
            }
        };
        match mapper.bus_to_ros(bus) {
            Ok(ros_msg) => {
                if let Err(e) = ros_pub_cb.publish(ros_msg) {
                    drop_stats.record_publish_fail();
                    route_health.record_publish_fail();
                    if route_health.should_log_warn() {
                        log::warn!("bus→ros {topic} publish: {e}");
                    }
                } else {
                    route_health.record_tx();
                }
            }
            Err(e) => {
                drop_stats.record_convert_fail();
                route_health.record_convert_fail();
                if route_health.should_log_warn() {
                    log::warn!("bus→ros {topic} convert: {e}");
                }
            }
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
    let deadline = Deadline::new(timeout);
    wait_service_ready(
        || client.service_is_ready().unwrap_or(false),
        deadline.remaining().map_err(|e| e.to_string())?,
    )?;
    let (tx, rx) = mpsc::sync_channel(1);
    let _promise = client
        .call_then(ros_req, move |resp: S::Response| {
            let _ = tx.send(resp);
        })
        .map_err(|e| format!("ros service call: {e}"))?;
    match rx.recv_timeout(deadline.remaining().map_err(|e| e.to_string())?) {
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
    let health = ctx.route_health;
    match ctx.direction {
        Direction::Ros2ToBus => {
            let bus_client = Arc::new(Mutex::new(ctx.bus_node.create_client_raw_with_qos(
                ctx.bus_service,
                QosProfile::keep_last(ctx.bus_qos.depth()),
            )?));
            let timeout = ctx.timeout;
            let pool = crate::runtime::WorkerPool::with_queue_capacity(2, 64);
            let srv = ctx
                .ros_node
                .create_async_service::<M::Ros, _>(
                    ros_topic_options(ctx.ros_service, ctx.ros_qos),
                    move |req: <M::Ros as ServiceIdl>::Request| {
                        health.rpc_start();
                        let deadline = Deadline::new(timeout);
                        let mapper = mapper.clone();
                        let worker_mapper = mapper.clone();
                        let client = Arc::clone(&bus_client);
                        let health = Arc::clone(&health);
                        let (tx, rx) = futures_channel::oneshot::channel();
                        let submitted = pool.try_submit(move || {
                            let result = (|| {
                                let body = worker_mapper.ros_req_to_bus(&req)?;
                                let guard = deadline.lock(&client)?;
                                let response = guard.call(&body, Some(deadline.remaining()?))?;
                                worker_mapper.bus_resp_to_ros(&response)
                            })();
                            let _ = tx.send(result);
                        });
                        async move {
                            let result = match submitted {
                                Ok(()) => rx.await.unwrap_or_else(|_| {
                                    Err(BusError::Protocol("bridge service worker stopped".into()))
                                }),
                                Err(e) => {
                                    Err(BusError::Protocol(format!("bridge service queue: {e}")))
                                }
                            };
                            health.rpc_finish(result.as_ref().err());
                            result.unwrap_or_else(|e| mapper.error_response(&e.to_string()))
                        }
                    },
                )
                .map_err(|e| BusError::Protocol(format!("ros create_async_service: {e}")))?;
            ctx.ros_entities.push(Box::new(srv));
        }
        Direction::BusToRos2 => {
            let client = ctx
                .ros_node
                .create_client::<M::Ros>(ros_topic_options(ctx.ros_service, ctx.ros_qos))
                .map_err(|e| BusError::Protocol(format!("ros create_client: {e}")))?;
            ctx.ros_entities.push(Box::new(Arc::clone(&client)));
            let timeout = ctx.timeout;
            let handler: ServiceHandler = Arc::new(move |body| {
                health.rpc_start();
                let result = (|| {
                    let req = mapper.bus_req_to_ros(body)?;
                    let response =
                        call_ros_service(&client, req, timeout).map_err(rpc_call_error)?;
                    mapper.ros_resp_to_bus(&response)
                })();
                health.rpc_finish(result.as_ref().err());
                result.unwrap_or_else(|e| rpc_error_body(&e))
            });
            let callback_group = CallbackGroup::mutually_exclusive();
            ctx.bus_node.create_service_raw_with_qos(
                ctx.bus_service,
                QosProfile::keep_last(ctx.bus_qos.depth()),
                handler,
                Some(&callback_group),
            )?;
        }
    }
    Ok(())
}

fn rpc_call_error(message: String) -> BusError {
    if message.contains("timed out") {
        BusError::Timeout(message)
    } else {
        BusError::Protocol(message)
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
    let health = ctx.route_health;
    match ctx.direction {
        Direction::Ros2ToBus => {
            let bus_client = Arc::new(Mutex::new(ctx.bus_node.create_action_client_raw_with_qos(
                ctx.bus_action,
                QosProfile::keep_last(ctx.bus_qos.depth()),
            )?));
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
                        let health = Arc::clone(&health);
                        health.rpc_start();
                        let deadline = Deadline::new(timeout);
                        async move {
                            let goal = (**requested.goal()).clone();
                            let accepted = requested.accept();
                            let executing = match accepted.begin() {
                                BeginAcceptedGoal::Execute(e) => e,
                                BeginAcceptedGoal::Cancel(c) => {
                                    health.rpc_finish(Some(&BusError::Cancelled {
                                        name: "ROS goal".into(),
                                    }));
                                    return c.cancelled_with(Default::default());
                                }
                            };
                            let bus_goal = match mapper.ros_goal_to_bus(&goal) {
                                Ok(b) => b,
                                Err(e) => {
                                    log::warn!("ros→bus encode goal failed: {e}");
                                    health.rpc_finish(Some(&e));
                                    return executing.aborted_with(Default::default());
                                }
                            };
                            let fb_pub = executing.feedback_publisher();
                            let fb_mapper = mapper.clone();
                            let feedback_cb: RawActionFeedbackCallback =
                                Arc::new(move |msg: &ActionMessage| {
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
                                let guard = match deadline.lock(&bus_client) {
                                    Ok(g) => g,
                                    Err(e) => {
                                        log::warn!("ros→bus action client lock poisoned: {e}");
                                        health.rpc_finish(Some(&BusError::Protocol(e.to_string())));
                                        return executing.aborted_with(Default::default());
                                    }
                                };
                                match guard.send_goal(
                                    &bus_goal,
                                    None,
                                    Some(match deadline.remaining() {
                                        Ok(t) => t,
                                        Err(e) => {
                                            health.rpc_finish(Some(&e));
                                            return executing.aborted_with(Default::default());
                                        }
                                    }),
                                    Some(feedback_cb),
                                ) {
                                    Ok(h) => h,
                                    Err(e) => {
                                        log::warn!("ros→bus send_goal failed: {e}");
                                        health.rpc_finish(Some(&e));
                                        return executing.aborted_with(Default::default());
                                    }
                                }
                            };
                            let wait_handle = bus_handle.clone();
                            // rclrs uses its own executor; no Tokio runtime is required.
                            let (tx, wait_fut) = futures_channel::oneshot::channel();
                            if let Err(e) = thread::Builder::new()
                                .name("ros_bridge_result".into())
                                .spawn(move || {
                                    let _ = tx.send(wait_handle.wait_result());
                                })
                            {
                                health.rpc_finish(Some(&BusError::Protocol(e.to_string())));
                                return executing.aborted_with(Default::default());
                            }
                            match executing.unless_cancel_requested(wait_fut).await {
                                Ok(Ok(Ok(result_msg))) => {
                                    match mapper.bus_result_to_ros(&result_msg.body) {
                                        Ok(result) => {
                                            health.rpc_finish(None);
                                            executing.succeeded_with(result)
                                        }
                                        Err(e) => {
                                            log::warn!("ros→bus decode result failed: {e}");
                                            health.rpc_finish(Some(&e));
                                            executing.aborted_with(Default::default())
                                        }
                                    }
                                }
                                Ok(Ok(Err(e))) => {
                                    log::warn!("ros→bus action goal failed: {e}");
                                    health.rpc_finish(Some(&e));
                                    if matches!(e, BusError::Cancelled { .. }) {
                                        executing
                                            .begin_cancelling()
                                            .cancelled_with(Default::default())
                                    } else {
                                        executing.aborted_with(Default::default())
                                    }
                                }
                                Ok(Err(e)) => {
                                    log::warn!("ros→bus action join failed: {e}");
                                    health.rpc_finish(Some(&BusError::Protocol(e.to_string())));
                                    executing.aborted_with(Default::default())
                                }
                                Err(()) => {
                                    health.rpc_finish(Some(&BusError::Cancelled {
                                        name: "ROS goal".into(),
                                    }));
                                    let _ = bus_handle.cancel();
                                    executing
                                        .begin_cancelling()
                                        .cancelled_with(Default::default())
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
                health.rpc_start();
                let result = mapper.bus_goal_to_ros(body).and_then(|goal| {
                    call_ros_action_mapped_live(&ros_client, &mapper, goal, timeout, ctx)
                });
                health.rpc_finish(result.as_ref().err());
                result.unwrap_or_else(|e| rpc_error_body(&e))
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
) -> Result<Vec<u8>>
where
    <M::Ros as ActionIdl>::Goal: Clone + Send + Sync + 'static,
    <M::Ros as ActionIdl>::Feedback: Clone + Send + Sync + 'static,
    <M::Ros as ActionIdl>::Result: Clone + Send + Sync + 'static,
{
    let budget = Deadline::new(timeout);
    let requested = client
        .try_request_goal(ros_goal)
        .map_err(|e| BusError::Protocol(format!("ros action request_goal: {e}")))?;
    let goal_client =
        match await_with_timeout(requested, budget.remaining()?).map_err(rpc_call_error)? {
            Some(gc) => gc,
            None => {
                return Err(BusError::ActionRejected(
                    "ROS action server rejected goal".into(),
                ));
            }
        };
    let GoalClient {
        mut feedback,
        result,
        cancellation,
        ..
    } = goal_client;
    let mut result_fut = result;
    let mut cancel_fut = None;
    let mut cancel_sent = false;
    loop {
        if ctx.cancel_requested() && !cancel_sent {
            cancel_fut = Some(cancellation.cancel());
            cancel_sent = true;
        }
        if let Some(fut) = cancel_fut.as_mut() {
            if poll_once(fut).is_ready() {
                cancel_fut = None;
            }
        }
        while let Ok(fb) = feedback.try_recv() {
            let bus_fb = mapper.ros_feedback_to_bus(&fb)?;
            ctx.publish_feedback(&bus_fb);
        }
        match poll_once(&mut result_fut) {
            Poll::Ready((status, res)) => {
                check_action_status(status)?;
                return mapper.ros_result_to_bus(&res);
            }
            Poll::Pending => {
                if budget.remaining().is_err() {
                    if !cancel_sent {
                        let mut cancel = cancellation.cancel();
                        let _ = poll_once(&mut cancel);
                    }
                    return Err(BusError::Timeout(
                        "timed out waiting for ROS action result".into(),
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

/// Keep `ros_entities` type in scope for docs / unused import silence.
#[allow(dead_code)]
fn _entity_slot(_: &mut Vec<Box<dyn Any + Send + Sync>>) {}

fn check_action_status(status: rclrs::GoalStatusCode) -> Result<()> {
    use rclrs::GoalStatusCode;
    match status {
        GoalStatusCode::Succeeded => Ok(()),
        GoalStatusCode::Cancelled => Err(BusError::Cancelled {
            name: "ROS action".into(),
        }),
        GoalStatusCode::Aborted => Err(BusError::ActionAborted("ROS action".into())),
        other => Err(BusError::Protocol(format!(
            "unexpected ROS action status: {other:?}"
        ))),
    }
}
