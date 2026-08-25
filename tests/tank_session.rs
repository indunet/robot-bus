//! TANK session API: acquire starts in-process tank; release + grace stops it.

#![cfg(all(feature = "console", feature = "ws"))]

mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::broker::{
    ConsoleBrokerConfig, DiscoveryConfig, WsGatewayConfig, RobotBusBroker, RobotBusConfig,
};
use robot_bus::geometry_msgs::msg::v1::{Pose2D, Twist, Vector3};
use robot_bus::{CMD_VEL_TOPIC, Node, NodeOptions, POSE_TOPIC};
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
            bind_opts: Default::default(),
            ..BusConfig::default()
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{svc_fe}"),
            backend_bind: format!("tcp://127.0.0.1:{svc_be}"),
            bind_all_transports: false,
            bind_opts: Default::default(),
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{act_fe}"),
            backend_bind: format!("tcp://127.0.0.1:{act_be}"),
            bind_all_transports: false,
            bind_opts: Default::default(),
            ..ActionBusConfig::default()
        },
        discovery: DiscoveryConfig {
            enabled: false,
            ..DiscoveryConfig::default()
        },
        ws: WsGatewayConfig {
            listen: format!("127.0.0.1:{http}").parse().unwrap(),
            ..WsGatewayConfig::default()
        },
        console: ConsoleBrokerConfig {
            enabled: true,
            tank_enabled: true,
            docs_enabled: true,
            listen: format!("127.0.0.1:{http}").parse().unwrap(),
            cors_origins: vec![],
        },
    }
}

#[test]
fn tank_session_starts_physics_and_stops_after_release() {
    let _guard = lock_brokers();
    let http = 26770u16;
    let broker = RobotBusBroker::start(test_broker_config(
        26560, 26561, 26662, 26663, 26664, 26665, http,
    ))
    .expect("start broker");
    thread::sleep(Duration::from_millis(200));

    let base = format!("http://127.0.0.1:{http}");
    let idle: serde_json::Value = ureq::get(&format!("{base}/api/v1/tank"))
        .call()
        .expect("status idle")
        .into_json()
        .expect("json");
    assert_eq!(idle["running"], false);
    assert_eq!(idle["viewers"], 0);

    let ui: serde_json::Value = ureq::get(&format!("{base}/api/v1/console"))
        .call()
        .expect("console ui")
        .into_json()
        .expect("json");
    assert_eq!(ui["tankEnabled"], true);
    assert_eq!(ui["docsEnabled"], true);

    let session: serde_json::Value = ureq::post(&format!("{base}/api/v1/tank/session"))
        .call()
        .expect("acquire")
        .into_json()
        .expect("json");
    let session_id = session["sessionId"]
        .as_str()
        .expect("sessionId")
        .to_string();
    assert!(session["viewers"].as_u64().unwrap_or(0) >= 1);

    let seen = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&seen);
    let mut opts = NodeOptions::tcp();
    opts.message_xsub = Some("tcp://127.0.0.1:26560".into());
    opts.message_xpub = Some("tcp://127.0.0.1:26561".into());
    let mut viewer = Node::with_options("tank_test_viewer", opts);
    let cmd = viewer
        .create_publisher::<Twist>(CMD_VEL_TOPIC)
        .expect("pub cmd");
    viewer
        .create_subscription::<Pose2D, _>(
            POSE_TOPIC,
            move |_t, _pose| {
                flag.store(true, Ordering::Relaxed);
            },
            None,
        )
        .expect("sub pose");

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && !seen.load(Ordering::Relaxed) {
        let _ = cmd.publish(&Twist {
            linear: Some(Vector3 {
                x: 0.5,
                y: 0.0,
                z: 0.0,
            }),
            angular: None,
        });
        let _ = viewer.spin_once(Some(Duration::from_millis(20)));
    }
    assert!(
        seen.load(Ordering::Relaxed),
        "expected /robot_bus/tank/pose from in-process tank"
    );

    let running: serde_json::Value = ureq::get(&format!("{base}/api/v1/tank"))
        .call()
        .expect("status running")
        .into_json()
        .expect("json");
    assert_eq!(running["running"], true);

    ureq::delete(&format!("{base}/api/v1/tank/session/{session_id}"))
        .call()
        .expect("release");

    // Default stop grace is 2s; sweep on status.
    thread::sleep(Duration::from_millis(2500));
    let after: serde_json::Value = ureq::get(&format!("{base}/api/v1/tank"))
        .call()
        .expect("status after")
        .into_json()
        .expect("json");
    assert_eq!(after["running"], false);
    assert_eq!(after["viewers"], 0);

    let _ = viewer.shutdown();
    broker.stop().expect("stop broker");
}
