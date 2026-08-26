//! Builtin Trigger service bridge smoke (ROS ↔ bus) under sourced ROS 2.
//!
//! Run: `cargo test --features ws,ros2 --test ros2_bridge_service_smoke -- --nocapture`

#![cfg(feature = "ros2")]

mod support;

use std::thread;
use std::time::{Duration, Instant};

use rclrs::CreateBasicExecutor;
use robot_bus::ros2_bridge::vendor::std_srvs::srv as ros_srv;
use robot_bus::ros2_bridge::{Direction, Ros2Bridge, TriggerServiceMapper};
use robot_bus::std_srvs::srv::v1::{Trigger, TriggerRequest, TriggerResponse};
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

#[test]
fn trigger_ros2_to_bus_roundtrip() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let opts = node_options_from_broker(&broker);

    let ros_svc = format!("/robot_bus_trigger_smoke_{}", std::process::id());
    let bus_svc = format!("trigger_smoke_{}", std::process::id());

    let mut bus_server = Node::with_options("trigger_bus_server", opts.clone());
    bus_server
        .create_service::<Trigger, _>(
            &bus_svc,
            |_req: TriggerRequest| TriggerResponse {
                success: true,
                message: "from-bus".into(),
            },
            None,
        )
        .expect("bus Trigger server");

    let mut bridge = Ros2Bridge::new(format!("trigger_bridge_{}", std::process::id()))
        .bus_options(opts)
        .service(&ros_svc, &bus_svc)
        .mapper(TriggerServiceMapper)
        .timeout(Duration::from_secs(3))
        .direction(Direction::Ros2ToBus)
        .add()
        .expect("add service")
        .build()
        .expect("build bridge");

    let client_name = format!("trigger_client_{}", std::process::id());
    let ctx = rclrs::Context::default_from_env().expect("ROS context");
    let mut exec = ctx.create_basic_executor();
    let node = exec
        .create_node(client_name.as_str())
        .expect("ros client node");
    let client = node
        .create_client::<ros_srv::Trigger>(ros_svc.as_str())
        .expect("ros Trigger client");

    let deadline = Instant::now() + Duration::from_secs(5);
    while !client.service_is_ready().unwrap_or(false) {
        if Instant::now() > deadline {
            panic!("ROS Trigger service not ready: {ros_svc}");
        }
        let _ = bus_server.spin_once(Some(Duration::from_millis(5)));
        let _ = bridge.spin_once(Duration::from_millis(5));
        let _ = exec.spin(rclrs::SpinOptions::spin_once().timeout(Duration::from_millis(20)));
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = client
        .call_then(
            ros_srv::Trigger_Request {
                structure_needs_at_least_one_member: 0,
            },
            move |resp: ros_srv::Trigger_Response| {
                let _ = tx.send(resp);
            },
        )
        .expect("call_then");

    let resp_deadline = Instant::now() + Duration::from_secs(5);
    let resp = loop {
        if let Ok(r) = rx.try_recv() {
            break r;
        }
        if Instant::now() > resp_deadline {
            panic!("timed out waiting for Trigger response");
        }
        let _ = bus_server.spin_once(Some(Duration::from_millis(5)));
        let _ = bridge.spin_once(Duration::from_millis(5));
        let _ = exec.spin(rclrs::SpinOptions::spin_once().timeout(Duration::from_millis(20)));
    };

    assert!(resp.success, "resp={resp:?}");
    assert_eq!(resp.message, "from-bus");
}

#[test]
fn trigger_bus_to_ros2_roundtrip() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("broker");
    let opts = node_options_from_broker(&broker);

    let ros_svc = format!("/robot_bus_trigger_b2r_{}", std::process::id());
    let bus_svc = format!("trigger_b2r_{}", std::process::id());

    let server_name = format!("trigger_ros_server_{}", std::process::id());
    let ros_svc_for_server = ros_svc.clone();
    let ros_spin = thread::spawn(move || {
        let ctx = rclrs::Context::default_from_env().expect("ROS context");
        let mut exec = ctx.create_basic_executor();
        let node = exec
            .create_node(server_name.as_str())
            .expect("ros server node");
        let _srv = node
            .create_service::<ros_srv::Trigger, _>(ros_svc_for_server.as_str(), |_req| {
                ros_srv::Trigger_Response {
                    success: true,
                    message: "from-ros".into(),
                }
            })
            .expect("ros Trigger server");
        for _ in 0..800 {
            let _ = exec.spin(rclrs::SpinOptions::spin_once().timeout(Duration::from_millis(20)));
        }
    });

    thread::sleep(Duration::from_millis(200));

    let mut bridge = Ros2Bridge::new(format!("trigger_b2r_bridge_{}", std::process::id()))
        .bus_options(opts.clone())
        .service(&ros_svc, &bus_svc)
        .mapper(TriggerServiceMapper)
        .timeout(Duration::from_secs(5))
        .direction(Direction::BusToRos2)
        .add()
        .expect("add service")
        .build()
        .expect("build bridge");

    let mut bus_client_node = Node::with_options("trigger_bus_client", opts);
    let client = bus_client_node
        .create_client::<Trigger>(&bus_svc)
        .expect("bus Trigger client");

    for _ in 0..50 {
        let _ = bridge.spin_once(Duration::from_millis(20));
    }

    let handle = thread::spawn(move || {
        client
            .call(&TriggerRequest {}, Some(Duration::from_secs(8)))
            .expect("bus call")
    });

    let deadline = Instant::now() + Duration::from_secs(10);
    while !handle.is_finished() {
        if Instant::now() > deadline {
            panic!("timed out spinning for bus→ros Trigger");
        }
        let _ = bridge.spin_once(Duration::from_millis(10));
    }

    let resp = handle.join().expect("join call thread");
    assert!(resp.success, "resp={resp:?}");
    assert_eq!(resp.message, "from-ros");
    let _ = ros_spin.join();
}
