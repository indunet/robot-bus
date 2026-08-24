//! Opt-in `.lazy()` ROS2→bus topic subscribe (requires sourced ROS 2).
//!
//! Run: `cargo test --features ws,ros2 --test ros2_bridge_lazy -- --nocapture`

#![cfg(feature = "ros2")]

mod support;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rclrs::CreateBasicExecutor;
use prost::Message as ProstMessage;
use robot_bus::lazy_subscribe::CONSOLE_DETECT_TIMEOUT;
use robot_bus::ros2_bridge::{Ros2Bridge, StdMsgsStringMapper, TopicMapper};
use robot_bus::std_msgs::msg::v1::String as BusString;
use robot_bus::{Node, NodeOptions, RobotBusBroker};
use support::{ephemeral_robot_bus_config, lock_brokers};

fn node_options_from_broker(broker: &RobotBusBroker) -> NodeOptions {
    NodeOptions {
        message_xsub: Some(broker.message.xsub_bind.clone()),
        message_xpub: Some(broker.message.xpub_bind.clone()),
        service_frontend: Some(broker.service.frontend_bind.clone()),
        service_backend: Some(broker.service.backend_bind.clone()),
        action_frontend: Some(broker.action.frontend_bind.clone()),
        action_backend: Some(broker.action.backend_bind.clone()),
        ..NodeOptions::default()
    }
}

fn console_on_config() -> robot_bus::RobotBusConfig {
    let mut config = ephemeral_robot_bus_config();
    config.console.enabled = true;
    config.console.tank_enabled = false;
    config
}

fn drain_bridge(bridge: &mut Ros2Bridge, n: usize) {
    for _ in 0..n {
        let _ = bridge.spin_once(Duration::from_millis(20));
    }
}

fn wait_until(bridge: &mut Ros2Bridge, timeout: Duration, pred: impl Fn(&Ros2Bridge) -> bool) {
    let deadline = Instant::now() + timeout;
    loop {
        if pred(bridge) {
            return;
        }
        assert!(Instant::now() < deadline, "timeout waiting for lazy state");
        let _ = bridge.spin_once(Duration::from_millis(20));
    }
}

#[test]
fn eager_has_ros_subscription_at_build() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let opts = node_options_from_broker(&broker);
    let topic = format!("/lazy_eager_{}", std::process::id());

    let bridge = Ros2Bridge::new(format!("eager_bridge_{}", std::process::id()))
        .bus_options(opts)
        .route(&topic, &topic)
        .mapper(StdMsgsStringMapper)
        .add()
        .expect("add")
        .build()
        .expect("build");

    assert!(
        bridge.has_ros_subscription(&topic),
        "eager ROS2→bus must subscribe at build"
    );

    broker.stop().expect("stop");
}

#[test]
fn lazy_waits_for_bus_subscriber_then_tears_down() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(console_on_config()).expect("broker");
    let opts = node_options_from_broker(&broker);
    let topic = format!("/lazy_cam_{}", std::process::id());

    let mut bridge = Ros2Bridge::new(format!("lazy_bridge_{}", std::process::id()))
        .bus_options(opts.clone())
        .route(&topic, &topic)
        .mapper(StdMsgsStringMapper)
        .lazy()
        .add()
        .expect("add")
        .build()
        .expect("build");

    assert!(
        !bridge.has_ros_subscription(&topic),
        "lazy route must not subscribe at build"
    );
    drain_bridge(&mut bridge, 10);
    assert!(
        !bridge.has_ros_subscription(&topic),
        "lazy route stays off with zero bus subscribers"
    );

    let received: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
    let received_cb = Arc::clone(&received);
    let mut listener = Node::with_options("lazy_listener", opts.clone());
    let sub = listener
        .create_subscription_raw(
            &topic,
            Arc::new(move |_t, payload| {
                received_cb.lock().expect("lock").push(payload.to_vec());
            }),
            None,
        )
        .expect("bus sub");

    wait_until(&mut bridge, Duration::from_secs(3), |b| {
        b.has_ros_subscription(&topic)
    });

    let mut listener2 = Node::with_options("lazy_listener_2", opts);
    let sub2 = listener2
        .create_subscription_raw(&topic, Arc::new(|_, _| {}), None)
        .expect("bus sub 2");
    drain_bridge(&mut bridge, 15);
    let _ = listener.spin_once(Some(Duration::from_millis(20)));
    let _ = listener2.spin_once(Some(Duration::from_millis(20)));
    assert!(
        bridge.has_ros_subscription(&topic),
        "second subscriber must keep the ROS sub alive"
    );

    listener.destroy_subscription(sub).expect("destroy 1");
    drain_bridge(&mut bridge, 20);
    assert!(
        bridge.has_ros_subscription(&topic),
        "remaining subscriber still needs the ROS sub"
    );

    listener2.destroy_subscription(sub2).expect("destroy 2");
    wait_until(&mut bridge, Duration::from_secs(3), |b| {
        !b.has_ros_subscription(&topic)
    });

    let _ = received.lock().expect("lock");
    broker.stop().expect("stop");
}

#[test]
fn lazy_and_eager_routes_independent_at_runtime() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(console_on_config()).expect("broker");
    let opts = node_options_from_broker(&broker);
    let eager = format!("/lazy_mix_eager_{}", std::process::id());
    let lazy = format!("/lazy_mix_lazy_{}", std::process::id());

    let mut bridge = Ros2Bridge::new(format!("mix_bridge_{}", std::process::id()))
        .bus_options(opts)
        .route(&eager, &eager)
        .mapper(StdMsgsStringMapper)
        .add()
        .unwrap()
        .route(&lazy, &lazy)
        .mapper(StdMsgsStringMapper)
        .lazy()
        .add()
        .unwrap()
        .build()
        .expect("build");

    assert!(bridge.has_ros_subscription(&eager));
    assert!(!bridge.has_ros_subscription(&lazy));
    drain_bridge(&mut bridge, 5);
    assert!(bridge.has_ros_subscription(&eager));
    assert!(!bridge.has_ros_subscription(&lazy));

    broker.stop().expect("stop");
}

#[test]
fn no_console_lazy_falls_back_eager() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let opts = node_options_from_broker(&broker);
    let topic = format!("/lazy_noconsole_{}", std::process::id());

    let mut bridge = Ros2Bridge::new(format!("noconsole_bridge_{}", std::process::id()))
        .bus_options(opts)
        .route(&topic, &topic)
        .mapper(StdMsgsStringMapper)
        .lazy()
        .add()
        .expect("add")
        .build()
        .expect("build");

    assert!(!bridge.has_ros_subscription(&topic));
    wait_until(
        &mut bridge,
        CONSOLE_DETECT_TIMEOUT + Duration::from_secs(1),
        |b| b.has_ros_subscription(&topic),
    );

    broker.stop().expect("stop");
}

#[test]
fn lazy_forwards_after_bus_subscribe() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(console_on_config()).expect("broker");
    let opts = node_options_from_broker(&broker);
    let topic = format!("/lazy_fwd_{}", std::process::id());

    let mut bridge = Ros2Bridge::new(format!("fwd_bridge_{}", std::process::id()))
        .bus_options(opts.clone())
        .route(&topic, &topic)
        .mapper(StdMsgsStringMapper)
        .lazy()
        .add()
        .expect("add")
        .build()
        .expect("build");

    let got: Arc<Mutex<Option<BusString>>> = Arc::new(Mutex::new(None));
    let got_cb = Arc::clone(&got);
    let mut listener = Node::with_options("fwd_listener", opts);
    let _sub = listener
        .create_subscription::<BusString, _>(
            &topic,
            move |_t, msg: BusString| {
                *got_cb.lock().expect("lock") = Some(msg);
            },
            None,
        )
        .expect("typed sub");

    wait_until(&mut bridge, Duration::from_secs(3), |b| {
        b.has_ros_subscription(&topic)
    });

    let ctx = rclrs::Context::default_from_env().expect("ROS context");
    let mut exec = ctx.create_basic_executor();
    let node = exec
        .create_node(format!("fwd_talker_{}", std::process::id()).as_str())
        .expect("ros talker");
    let mapper = StdMsgsStringMapper;
    let publisher = node
        .create_dynamic_publisher(mapper.ros_type(), topic.as_str())
        .expect("ros dynamic pub");
    let payload = BusString {
        data: "hello-lazy".into(),
    }
    .encode_to_vec();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let dyn_msg = mapper.bus_to_ros(&payload).expect("bus_to_ros");
        let _ = publisher.publish(dyn_msg);
        let _ = exec.spin(rclrs::SpinOptions::spin_once().timeout(Duration::from_millis(20)));
        let _ = bridge.spin_once(Duration::from_millis(20));
        let _ = listener.spin_once(Some(Duration::from_millis(20)));
        if got
            .lock()
            .expect("lock")
            .as_ref()
            .is_some_and(|m| m.data == "hello-lazy")
        {
            break;
        }
        assert!(Instant::now() < deadline, "lazy bridge never forwarded");
    }

    broker.stop().expect("stop");
}
