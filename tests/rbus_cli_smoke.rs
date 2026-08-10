//! Smoke: `rbus` CLI against an in-process broker console API.
//!
//! Topic list only shows topics with real subscriber traffic (same as metrics).

#![cfg(feature = "console")]

mod support;

use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use robot_bus::action_bus::ActionClient;
use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::broker::{
    ConsoleBrokerConfig, DiscoveryConfig, WsGatewayConfig, RobotBusBroker, RobotBusConfig,
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
            tank_enabled: false,
            listen: format!("127.0.0.1:{http}").parse().unwrap(),
            cors_origins: vec![],
        },
    }
}

fn rbus_bin() -> &'static str {
    env!("CARGO_BIN_EXE_rbus")
}

fn run_rbus(url: &str, args: &[&str]) -> (i32, String, String) {
    let output = Command::new(rbus_bin())
        .arg("--url")
        .arg(url)
        .args(args)
        .output()
        .expect("spawn rbus");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn rbus_lists_and_status() {
    let _guard = lock_brokers();
    let http_port = 27770u16;
    let broker = RobotBusBroker::start(test_broker_config(
        27560, 27561, 27662, 27663, 27664, 27665, http_port,
    ))
    .expect("start broker");
    thread::sleep(Duration::from_millis(200));

    let url = format!("http://127.0.0.1:{http_port}");

    // Empty topic list is fine (exit 0).
    let (code, stdout, stderr) = run_rbus(&url, &["topic", "list"]);
    assert_eq!(code, 0, "topic list empty: stderr={stderr}");
    assert!(
        stdout.trim().is_empty(),
        "expected empty topics, got {stdout}"
    );

    let pub_ = Publisher::new(Some("tcp://127.0.0.1:27560")).expect("publisher");
    thread::sleep(Duration::from_millis(100));
    let sub = Subscriber::new(Some("tcp://127.0.0.1:27561")).expect("subscriber");
    sub.subscribe("/rbus").expect("subscribe");
    thread::sleep(Duration::from_millis(500));

    for _ in 0..10 {
        pub_.publish("/rbus/cli", &[0u8; 4]).unwrap();
        thread::sleep(Duration::from_millis(5));
    }
    thread::sleep(Duration::from_millis(200));

    let svc_handler: Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|body| [b"ok:", body].concat());
    let svc_worker = WorkerThread::spawn_service("svc.rbus", svc_handler, "tcp://127.0.0.1:27663")
        .expect("service worker");

    let act_handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> =
        Arc::new(|body| vec![("RESULT".into(), [b"done:", body].concat())]);
    let act_worker = WorkerThread::spawn_action("act.rbus", act_handler, "tcp://127.0.0.1:27665")
        .expect("action worker");

    thread::sleep(Duration::from_millis(150));

    let svc_client = ServiceClient::new(Some("tcp://127.0.0.1:27662")).expect("svc client");
    svc_client
        .call("svc.rbus", b"ping", None, Some(Duration::from_secs(5)))
        .expect("service call");

    let act_client = ActionClient::new(Some("tcp://127.0.0.1:27664")).expect("act client");
    act_client
        .send_goal("act.rbus", b"go", None, Some(Duration::from_secs(10)))
        .expect("action goal");

    thread::sleep(Duration::from_millis(150));

    let (code, stdout, stderr) = run_rbus(&url, &["topic", "list"]);
    assert_eq!(code, 0, "topic list: stderr={stderr}");
    assert!(
        stdout
            .lines()
            .any(|l| l.split('\t').next() == Some("/rbus/cli")),
        "topics stdout={stdout}"
    );

    let (code, stdout, stderr) = run_rbus(&url, &["service", "list"]);
    assert_eq!(code, 0, "service list: stderr={stderr}");
    assert!(
        stdout.lines().any(|l| l == "svc.rbus"),
        "services stdout={stdout}"
    );

    let (code, stdout, stderr) = run_rbus(&url, &["action", "list"]);
    assert_eq!(code, 0, "action list: stderr={stderr}");
    assert!(
        stdout.lines().any(|l| l == "act.rbus"),
        "actions stdout={stdout}"
    );

    let (code, stdout, stderr) = run_rbus(&url, &["status"]);
    assert_eq!(code, 0, "status: stderr={stderr}");
    assert!(stdout.contains("status: ONLINE"), "status stdout={stdout}");
    assert!(stdout.contains("version:"), "status stdout={stdout}");

    drop(sub);
    svc_worker.stop();
    act_worker.stop();
    broker.stop().expect("stop");
}
