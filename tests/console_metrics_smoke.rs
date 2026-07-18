//! Console message-metrics smoke (starts an in-process broker).
//!
//! Metrics only observe traffic that real subscribers receive (no internal
//! blanket SUB — communication efficiency comes first).

#![cfg(feature = "console")]

use std::thread;
use std::time::Duration;

use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::broker::{ConsoleBrokerConfig, GrpcBrokerConfig, RobotBusBroker, RobotBusConfig};
use robot_bus::message_bus::{Publisher, Subscriber};

#[test]
fn message_metrics_count_published_topics() {
    let broker = RobotBusBroker::start(RobotBusConfig {
        message: BusConfig {
            xsub_bind: "tcp://127.0.0.1:25560".into(),
            xpub_bind: "tcp://127.0.0.1:25561".into(),
            bind_all_transports: false,
            ..BusConfig::default()
        },
        service: ServiceBusConfig {
            frontend_bind: "tcp://127.0.0.1:25662".into(),
            backend_bind: "tcp://127.0.0.1:25663".into(),
            bind_all_transports: false,
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: "tcp://127.0.0.1:25664".into(),
            backend_bind: "tcp://127.0.0.1:25665".into(),
            bind_all_transports: false,
            ..ActionBusConfig::default()
        },
        grpc: GrpcBrokerConfig {
            listen: "127.0.0.1:25770".parse().unwrap(),
            ..GrpcBrokerConfig::default()
        },
        console: ConsoleBrokerConfig {
            enabled: true,
            listen: "127.0.0.1:25771".parse().unwrap(),
        },
    })
    .expect("start broker");

    thread::sleep(Duration::from_millis(200));

    // Real subscriber required — otherwise ZMQ PUB drops and metrics stay empty.
    let sub = Subscriber::new(Some("tcp://127.0.0.1:25561")).expect("subscriber");
    sub.subscribe("/demo").expect("subscribe");
    thread::sleep(Duration::from_millis(200));

    let pub_ = Publisher::new(Some("tcp://127.0.0.1:25560")).expect("publisher");
    thread::sleep(Duration::from_millis(100));
    for _ in 0..20 {
        pub_.publish("/demo/imu", &[0u8; 8]).unwrap();
        pub_.publish("/demo/odom", &[1u8; 4]).unwrap();
        thread::sleep(Duration::from_millis(5));
    }
    thread::sleep(Duration::from_millis(200));

    let snap = broker.message.metrics.snapshot();
    assert!(
        snap.total_msgs >= 40,
        "expected >=40 msgs, got {} topics={:?}",
        snap.total_msgs,
        snap.topics
    );
    assert!(
        snap.topics.iter().any(|t| t.name == "/demo/imu"),
        "missing /demo/imu in {:?}",
        snap.topics
    );
    assert!(
        snap.topics.iter().any(|t| t.name == "/demo/odom"),
        "missing /demo/odom in {:?}",
        snap.topics
    );

    let status = std::process::Command::new("curl")
        .args(["-s", "http://127.0.0.1:25771/api/v1/status"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    assert!(status.contains("ONLINE"), "status={status}");

    // Keep sub alive until after snapshot (drop order).
    drop(sub);
    broker.stop().expect("stop");
}
