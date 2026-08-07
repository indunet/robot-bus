//! Console message / service / action metrics smoke (in-process broker).
//!
//! Message metrics only observe traffic that real subscribers receive (no
//! internal blanket SUB — communication efficiency comes first).

#![cfg(feature = "console")]

mod support;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use robot_bus::action_bus::ActionClient;
use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::broker::{
    ConsoleBrokerConfig, DiscoveryConfig, GrpcBrokerConfig, RobotBusBroker, RobotBusConfig,
};
use robot_bus::message_bus::{Publisher, Subscriber};
use robot_bus::service_bus::ServiceClient;
use robot_bus::worker_thread::WorkerThread;
use support::lock_brokers;

fn test_broker_config(
    msg_xsub: u16,
    msg_xpub: u16,
    svc_fe: u16,
    svc_be: u16,
    act_fe: u16,
    act_be: u16,
    http: u16,
) -> RobotBusConfig {
    RobotBusConfig {
        message: BusConfig {
            xsub_bind: format!("tcp://127.0.0.1:{msg_xsub}"),
            xpub_bind: format!("tcp://127.0.0.1:{msg_xpub}"),
            bind_all_transports: false,
            ..BusConfig::default()
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{svc_fe}"),
            backend_bind: format!("tcp://127.0.0.1:{svc_be}"),
            bind_all_transports: false,
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{act_fe}"),
            backend_bind: format!("tcp://127.0.0.1:{act_be}"),
            bind_all_transports: false,
            ..ActionBusConfig::default()
        },
        discovery: DiscoveryConfig {
            enabled: false,
            ..DiscoveryConfig::default()
        },
        grpc: GrpcBrokerConfig {
            listen: format!("127.0.0.1:{http}").parse().unwrap(),
            ..GrpcBrokerConfig::default()
        },
        console: ConsoleBrokerConfig {
            enabled: true,
            listen: format!("127.0.0.1:{http}").parse().unwrap(),
            cors_origins: vec![],
        },
    }
}

#[test]
fn message_metrics_count_published_topics() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(test_broker_config(
        25560, 25561, 25662, 25663, 25664, 25665, 25770,
    ))
    .expect("start broker");

    thread::sleep(Duration::from_millis(200));

    // Connect the publisher before subscribing so the XSUB can forward the
    // subscription to every currently connected upstream publisher.
    let pub_ = Publisher::new(Some("tcp://127.0.0.1:25560")).expect("publisher");
    thread::sleep(Duration::from_millis(100));

    // Real subscriber required — otherwise ZMQ PUB drops and metrics stay empty.
    let sub = Subscriber::new(Some("tcp://127.0.0.1:25561")).expect("subscriber");
    sub.subscribe("/demo").expect("subscribe");
    thread::sleep(Duration::from_millis(500));

    for _ in 0..20 {
        pub_.publish("/demo/imu", &[0u8; 8]).unwrap();
        pub_.publish("/demo/odom", &[1u8; 4]).unwrap();
        thread::sleep(Duration::from_millis(5));
    }
    thread::sleep(Duration::from_millis(200));

    let snap = broker.message.metrics.snapshot();
    assert!(
        snap.total_msgs >= 38,
        "expected nearly all 40 msgs, got {} topics={:?}",
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
        .args(["-s", "http://127.0.0.1:25770/api/v1/status"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    assert!(status.contains("ONLINE"), "status={status}");

    let index = std::process::Command::new("curl")
        .args(["-s", "http://127.0.0.1:25770/"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    assert!(index.contains("<!DOCTYPE html>"), "index={index}");

    // Keep sub alive until after snapshot (drop order).
    drop(sub);
    broker.stop().expect("stop");
}

#[test]
fn service_and_action_metrics_via_console_api() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(test_broker_config(
        26560, 26561, 26662, 26663, 26664, 26665, 26770,
    ))
    .expect("start broker");
    thread::sleep(Duration::from_millis(200));

    let svc_handler: Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|body| [b"echo:", body].concat());
    let svc_worker =
        WorkerThread::spawn_service("svc.console", svc_handler, "tcp://127.0.0.1:26663")
            .expect("service worker");

    let act_handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> =
        Arc::new(|body| vec![("RESULT".into(), [b"done:", body].concat())]);
    let act_worker =
        WorkerThread::spawn_action("act.console", act_handler, "tcp://127.0.0.1:26665")
            .expect("action worker");

    thread::sleep(Duration::from_millis(150));

    let svc_client = ServiceClient::new(Some("tcp://127.0.0.1:26662")).expect("svc client");
    let reply = svc_client
        .call("svc.console", b"ping", None, Some(Duration::from_secs(5)))
        .expect("service call");
    assert_eq!(reply, b"echo:ping");

    let act_client = ActionClient::new(Some("tcp://127.0.0.1:26664")).expect("act client");
    let messages = act_client
        .send_goal("act.console", b"go", None, Some(Duration::from_secs(10)))
        .expect("action goal");
    assert!(!messages.is_empty());

    thread::sleep(Duration::from_millis(100));

    let svc_snap = broker.service.metrics.snapshot();
    assert!(
        svc_snap
            .services
            .iter()
            .any(|s| s.name == "svc.console" && s.calls >= 1),
        "service metrics={:?}",
        svc_snap.services
    );

    let act_snap = broker.action.metrics.snapshot();
    assert!(
        act_snap
            .actions
            .iter()
            .any(|a| a.name == "act.console" && a.runs >= 1),
        "action metrics={:?}",
        act_snap.actions
    );

    let services_json = std::process::Command::new("curl")
        .args(["-s", "http://127.0.0.1:26770/api/v1/services"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    assert!(
        services_json.contains("svc.console"),
        "services api={services_json}"
    );

    let actions_json = std::process::Command::new("curl")
        .args(["-s", "http://127.0.0.1:26770/api/v1/actions"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    assert!(
        actions_json.contains("act.console"),
        "actions api={actions_json}"
    );

    svc_worker.stop();
    act_worker.stop();
    broker.stop().expect("stop");
}
