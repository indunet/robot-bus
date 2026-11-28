//! Smoke: `rbus` CLI against the broker's discovery and monitoring APIs.
//!
//! Topic list only shows topics with real subscriber traffic (same as metrics).

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::{Command, Output};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::common::{ephemeral_robot_bus_config, lock_brokers};
use robot_bus::action_bus::ActionClient;
use robot_bus::broker::RobotBusBroker;
use robot_bus::message_bus::{Publisher, Subscriber};
use robot_bus::service_bus::ServiceClient;
use robot_bus::worker_thread::WorkerThread;
use serde_json::Value;

fn rbus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rbus"))
}

fn run_rbus(url: &str, args: &[&str]) -> (i32, String, String) {
    let output = rbus()
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

fn returned_json(output: Output) -> Value {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).expect("parse rbus JSON output")
}

fn rbus_json(url: &str, args: &[&str]) -> Value {
    returned_json(
        rbus()
            .args(["--url", url])
            .args(args)
            .arg("--json")
            .output()
            .unwrap(),
    )
}

fn assert_failure((code, stdout, stderr): (i32, String, String), diagnostic: &str) {
    assert_ne!(code, 0, "unexpected success with stdout={stdout}");
    assert!(stdout.is_empty(), "unexpected stdout={stdout}");
    assert!(stderr.contains(diagnostic), "stderr={stderr}");
}

#[test]
fn rbus_lists_and_status() {
    let _guard = lock_brokers();
    let mut config = ephemeral_robot_bus_config();
    config.console.enabled = true;
    let broker = RobotBusBroker::start(config).expect("start broker");
    thread::sleep(Duration::from_millis(200));

    let url = format!("http://{}", broker.api_listen());

    let (code, stdout, stderr) = run_rbus(&url, &["discover"]);
    assert_eq!(code, 0, "discover: stderr={stderr}");
    assert!(
        stdout.lines().any(|line| line == "TCP host: 127.0.0.1"),
        "{stdout}"
    );
    assert!(
        stdout.contains(&format!("Topic XSUB: {}", broker.message.xsub_bind)),
        "{stdout}"
    );
    assert_eq!(
        rbus_json(&url, &["discover"]),
        serde_json::to_value(&broker.discover).unwrap()
    );

    // Empty topic list is fine (exit 0).
    let (code, stdout, stderr) = run_rbus(&url, &["topic", "list"]);
    assert_eq!(code, 0, "topic list empty: stderr={stderr}");
    assert!(
        stdout.trim().is_empty(),
        "expected empty topics, got {stdout}"
    );

    let pub_ = Publisher::new(Some(&broker.message.xsub_bind)).expect("publisher");
    thread::sleep(Duration::from_millis(100));
    let sub = Subscriber::new(Some(&broker.message.xpub_bind)).expect("subscriber");
    sub.subscribe("/rbus").expect("subscribe");
    thread::sleep(Duration::from_millis(500));

    for _ in 0..10 {
        pub_.publish("/rbus/cli", &[0u8; 4]).unwrap();
        thread::sleep(Duration::from_millis(5));
    }
    thread::sleep(Duration::from_millis(200));

    let svc_handler: Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|body| [b"ok:", body].concat());
    let svc_worker =
        WorkerThread::spawn_service("svc.rbus", svc_handler, &broker.service.backend_bind)
            .expect("service worker");

    let act_handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> =
        Arc::new(|body| vec![("RESULT".into(), [b"done:", body].concat())]);
    let act_worker =
        WorkerThread::spawn_action("act.rbus", act_handler, &broker.action.backend_bind)
            .expect("action worker");

    thread::sleep(Duration::from_millis(150));

    let svc_client = ServiceClient::new(Some(&broker.service.frontend_bind)).expect("svc client");
    svc_client
        .call("svc.rbus", b"ping", None, Some(Duration::from_secs(5)))
        .expect("service call");

    let act_client = ActionClient::new(Some(&broker.action.frontend_bind)).expect("act client");
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

    let (code, stdout, stderr) = run_rbus(&url, &["topic", "info", "/rbus/cli"]);
    assert_eq!(code, 0, "topic info: stderr={stderr}");
    assert!(
        stdout.starts_with("Topic: /rbus/cli\nType: -\n"),
        "{stdout}"
    );
    assert!(stdout.contains("Total msgs: "), "{stdout}");

    let topics = rbus_json(&url, &["topic", "list"]);
    let topic = topics["topics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == "/rbus/cli")
        .unwrap();
    assert!(topic["sparkline"].is_array(), "{topic}");
    let detail = rbus_json(&url, &["topic", "info", "/rbus/cli"]);
    assert_eq!(detail["name"], "/rbus/cli");
    assert!(detail["totalMsgs"].is_u64(), "{detail}");
    let services = rbus_json(&url, &["service", "list"]);
    assert!(
        services["services"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "svc.rbus" && item["workers"] == 1),
        "{services}"
    );
    let actions = rbus_json(&url, &["action", "list"]);
    assert!(
        actions["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "act.rbus" && item["runs"] == 1),
        "{actions}"
    );
    let status = rbus_json(&url, &["status"]);
    assert_eq!(status["status"], "ONLINE");
    assert_eq!(status["svcFE"], broker.service.frontend_bind);
    assert_failure(
        run_rbus(&url, &["topic", "info", "/rbus/absent", "--json"]),
        "topic not found",
    );

    drop(sub);
    svc_worker.stop();
    act_worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn rbus_discover_without_monitoring() {
    let _guard = lock_brokers();
    let mut config = ephemeral_robot_bus_config();
    config.discovery.advertise_host = Some("data.example.test".into());
    config.discovery.domain_id = 7;
    let broker = RobotBusBroker::start(config).expect("start broker");
    let address = broker.api_listen().to_string();

    let (code, stdout, stderr) = run_rbus(&address, &["discover"]);
    assert_eq!(code, 0, "discover: stderr={stderr}");
    for line in [
        format!("Broker ID: {}", broker.discover.broker_id),
        "Domain ID: 7".into(),
        "TCP host: data.example.test".into(),
        format!(
            "Topic XSUB: tcp://data.example.test:{}",
            broker.discover.message_xsub_port
        ),
        format!(
            "Topic XPUB: tcp://data.example.test:{}",
            broker.discover.message_xpub_port
        ),
        format!(
            "Service frontend: tcp://data.example.test:{}",
            broker.discover.service_frontend_port
        ),
        format!(
            "Service backend: tcp://data.example.test:{}",
            broker.discover.service_backend_port
        ),
        format!(
            "Action frontend: tcp://data.example.test:{}",
            broker.discover.action_frontend_port
        ),
        format!(
            "Action backend: tcp://data.example.test:{}",
            broker.discover.action_backend_port
        ),
        format!(
            "Topic federation: tcp://data.example.test:{}",
            broker.discover.message_fed_port
        ),
        "Federation capabilities: join".into(),
    ] {
        assert!(
            stdout.lines().any(|actual| actual == line),
            "missing {line:?} in {stdout}"
        );
    }

    let expected = serde_json::to_value(&broker.discover).unwrap();
    let by_env = rbus()
        .env("ROBOT_BUS_BROKER_URL", format!(" {address}/ "))
        .args(["--json", "discover"])
        .output()
        .unwrap();
    assert_eq!(returned_json(by_env), expected);
    let by_flag = rbus()
        .env("ROBOT_BUS_BROKER_URL", "127.0.0.1:0")
        .args(["discover", "--url", &address, "--json"])
        .output()
        .unwrap();
    assert_eq!(returned_json(by_flag), expected);

    for args in [
        &["status", "--json"][..],
        &["topic", "info", "/rbus/absent"],
    ] {
        let failure = run_rbus(&address, args);
        assert!(failure.2.contains("rbus discover"), "stderr={}", failure.2);
        assert_failure(failure, "monitoring API unavailable (HTTP 404)");
    }
    broker.stop().expect("stop");
}

fn rbus_with_http_response(args: &[&str], response: &'static str) -> (i32, String, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = BufReader::new(&mut stream);
        let mut line = String::new();
        loop {
            line.clear();
            if request.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                break;
            }
        }
        stream.write_all(response.as_bytes()).unwrap();
    });
    let output = run_rbus(&address, args);
    server.join().unwrap();
    output
}

#[test]
fn rbus_rejects_missing_discovery_and_invalid_responses() {
    assert_failure(
        rbus_with_http_response(
            &["discover"],
            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n",
        ),
        "discovery endpoint unavailable",
    );
    assert_failure(
        rbus_with_http_response(
            &["status", "--json"],
            "HTTP/1.1 200 OK\r\nContent-Length: 8\r\n\r\nnot JSON",
        ),
        "invalid JSON response",
    );
    assert_failure(
        rbus_with_http_response(
            &["topic", "list"],
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}",
        ),
        "missing field `topics`",
    );
}

#[test]
fn rbus_reports_unreachable_api_and_enforces_timeout() {
    assert_failure(
        run_rbus("127.0.0.1:0", &["discover"]),
        "could not reach broker API",
    );
    assert_failure(
        run_rbus("127.0.0.1:0", &["status", "--timeout-ms", "0"]),
        "invalid value '0'",
    );

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let server = thread::spawn(move || {
        let (_stream, _) = listener.accept().unwrap();
        // Close after a delay so a client that ignores the timeout still exits.
        thread::sleep(Duration::from_millis(800));
    });
    let output = run_rbus(&address, &["--timeout-ms", "100", "discover", "--json"]);
    server.join().unwrap();
    assert_failure(output, "timed out");
}
