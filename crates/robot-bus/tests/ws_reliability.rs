//! Fault injection and bounded-load checks over the real TCP and WebSocket transports.

#[path = "common/mod.rs"]
mod common;

use std::net::{SocketAddr, TcpListener};
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::common::{ephemeral_robot_bus_config, free_ports, lock_brokers};
use robot_bus::RobotBusBroker;
use robot_bus::runtime::NodeServiceClientRaw;
use robot_bus::{
    BusError, ConnectionState, MultiThreadedExecutor, Node, NodeOptions, Publisher, QosProfile,
    SubscriptionHandle,
};
use tokio::net::TcpStream;
use tokio::sync::watch;

const TIMEOUT: Duration = Duration::from_secs(5);
const OP_TIMEOUT: Duration = Duration::from_secs(2);
const PULSE: Duration = Duration::from_millis(5);
const TOPIC: &str = "/reliability/telemetry";
const SERVICE: &str = "/reliability/echo";
const ACTION: &str = "/reliability/goal";

struct BrokerChild(Child);

impl BrokerChild {
    fn start(ports: &[u16]) -> Self {
        let flags = [
            "--api-listen",
            "--message-xsub-bind",
            "--message-xpub-bind",
            "--service-frontend-bind",
            "--service-backend-bind",
            "--action-frontend-bind",
            "--action-backend-bind",
        ];
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_robot_bus_broker"));
        cmd.args([
            "--tcp-only",
            "--no-discovery",
            "--no-tank",
            "--no-docs",
            "--advertise-host",
            "127.0.0.1",
        ]);
        for (flag, port) in flags.into_iter().zip(ports) {
            cmd.arg(flag).arg(format!("127.0.0.1:{port}"));
        }
        Self(cmd.spawn().expect("launch broker subprocess"))
    }

    fn crash(mut self) {
        self.0.kill().expect("kill broker subprocess");
        assert!(!self.0.wait().expect("reap crashed broker").success());
    }
}

impl Drop for BrokerChild {
    fn drop(&mut self) {
        if matches!(self.0.try_wait(), Ok(None)) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

/// Pause an existing TCP path without propagating FIN, and refuse new paths
/// until restored. The client must detect the outage through its own heartbeat.
struct Link {
    address: SocketAddr,
    online: watch::Sender<bool>,
    _runtime: tokio::runtime::Runtime,
}

impl Link {
    fn new(upstream: SocketAddr) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let (online, state) = watch::channel(true);
        runtime.spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            while let Ok((mut client, _)) = listener.accept().await {
                let mut state = state.clone();
                if !*state.borrow_and_update() {
                    continue;
                }
                tokio::spawn(async move {
                    let Ok(mut broker) = TcpStream::connect(upstream).await else {
                        return;
                    };
                    tokio::select! {
                        _ = tokio::io::copy_bidirectional(&mut client, &mut broker) => {}
                        _ = async { let _ = state.wait_for(|online| !*online).await; } => {
                            let _ = state.wait_for(|online| *online).await;
                        }
                    }
                });
            }
        });
        Self {
            address,
            online,
            _runtime: runtime,
        }
    }
}

fn wait_until(label: &str, timeout: Duration, mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + timeout;
    loop {
        if condition() {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {label}");
        thread::sleep(Duration::from_millis(20));
    }
}

fn broker_http(url: &str, path: &str) -> Option<serde_json::Value> {
    ureq::get(&format!("{url}{path}"))
        .timeout(Duration::from_millis(250))
        .call()
        .ok()?
        .into_json()
        .ok()
}

fn start_broker() -> RobotBusBroker {
    let mut config = ephemeral_robot_bus_config();
    config.discovery.advertise_host = Some("127.0.0.1".into());
    config.console.enabled = true;
    RobotBusBroker::start(config).unwrap()
}

fn ws_client(name: &str, url: &str, service_frontend: &str) -> Node {
    Node::with_options(
        name,
        NodeOptions {
            service_frontend: Some(service_frontend.into()),
            ..NodeOptions::ws_at(url)
        },
    )
}

fn subscribe(node: &mut Node, filter: &str, depth: i32) -> (SubscriptionHandle, Receiver<Vec<u8>>) {
    let (tx, rx) = mpsc::channel();
    let handle = node
        .create_subscription_raw_with_qos(
            filter,
            QosProfile::keep_last(depth),
            Arc::new(move |payload| {
                let _ = tx.send(payload.to_vec());
            }),
            None,
        )
        .unwrap();
    (handle, rx)
}

fn wait_for_topic(node: &mut Node, rx: &Receiver<Vec<u8>>, publisher: &Publisher, marker: &[u8]) {
    wait_until("subscription to deliver a fresh marker", TIMEOUT, || {
        publisher.publish(TOPIC, marker).unwrap();
        node.spin_once(Some(PULSE)).unwrap();
        // `any` stops at the first match and would leave later copies queued.
        let mut matched = false;
        while let Ok(payload) = rx.try_recv() {
            matched |= payload == marker;
        }
        matched
    });
}

fn echo_server(
    url: &str,
    on_request: robot_bus::runtime::ServiceHandler,
) -> (Node, MultiThreadedExecutor) {
    let opts = NodeOptions {
        console_url: Some(url.into()),
        ..NodeOptions::tcp()
    };
    let mut node = Node::with_options("reliability-backend", opts);
    assert!(
        node.wait_for_broker(Some(TIMEOUT)),
        "native worker did not discover broker"
    );
    let executor = MultiThreadedExecutor::new(2);
    executor.add_node(&mut node).unwrap();
    node.create_service_raw(SERVICE, on_request, None).unwrap();
    node.create_action_server_raw(
        ACTION,
        Arc::new(|payload| vec![("RESULT".into(), payload.to_vec())]),
        None,
    )
    .unwrap();
    executor.start().unwrap();
    (node, executor)
}

fn assert_echo(client: &NodeServiceClientRaw, message: &[u8]) {
    assert_eq!(client.call(message, Some(OP_TIMEOUT)).unwrap(), message);
}

fn interrupted_echo() -> (
    robot_bus::runtime::ServiceHandler,
    Receiver<()>,
    mpsc::Sender<()>,
) {
    let (entered, observed) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let released = Mutex::new(released);
    let handler = Arc::new(move |payload: &[u8]| {
        if payload == b"in-flight" {
            let _ = entered.send(());
            let _ = released
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(18));
        }
        payload.to_vec()
    });
    (handler, observed, release)
}

fn with_interrupted_call(
    client: &Arc<NodeServiceClientRaw>,
    entered: &Receiver<()>,
    interrupt: impl FnOnce(),
) {
    let client = Arc::clone(client);
    let (done, result) = mpsc::channel();
    let call = thread::spawn(move || {
        let _ = done.send(client.call_with_id(
            b"in-flight",
            Some("interrupted"),
            Some(Duration::from_secs(18)),
        ));
    });
    entered
        .recv_timeout(TIMEOUT)
        .expect("RPC reached the native worker");
    interrupt();
    let result = result
        .recv_timeout(Duration::from_secs(14))
        .expect("in-flight call must finish while the link is still down");
    call.join().unwrap();
    assert!(
        matches!(&result, Err(BusError::Protocol(reason)) if reason.contains("websocket closed")),
        "in-flight call must fail on socket loss: {result:?}",
    );
}

#[test]
fn broker_crash_recovers_existing_subscription_and_native_workers() {
    let _lock = lock_brokers();
    let ports = free_ports(7);
    let url = format!("http://127.0.0.1:{}", ports[0]);
    let broker = BrokerChild::start(&ports);
    let (handler, entered, release) = interrupted_echo();
    let (_worker, executor) = echo_server(&url, handler);
    let mut node = ws_client(
        "crash-client",
        &url,
        &format!("tcp://127.0.0.1:{}", ports[3]),
    );
    assert!(node.wait_for_broker(Some(TIMEOUT)));
    let (_, received) = subscribe(&mut node, TOPIC, 8);
    let client = Arc::new(node.create_client_raw(SERVICE).unwrap());
    let action = node.create_action_client_raw(ACTION).unwrap();
    assert!(client.wait_for_service(Some(TIMEOUT)));
    assert!(action.wait_for_action_server(Some(TIMEOUT)));
    assert_echo(&client, b"before crash");

    let publisher = Publisher::new(Some(&format!("tcp://127.0.0.1:{}", ports[1]))).unwrap();
    wait_for_topic(&mut node, &received, &publisher, b"before crash");
    with_interrupted_call(&client, &entered, || broker.crash());
    wait_until("socket loss to change connection state", TIMEOUT, || {
        node.connection_state() != ConnectionState::Connected
    });
    release.send(()).unwrap();

    let _replacement = BrokerChild::start(&ports);
    assert!(
        node.wait_for_broker(Some(TIMEOUT)),
        "WebSocket did not reconnect"
    );
    wait_for_topic(&mut node, &received, &publisher, b"after crash");
    assert!(
        client.wait_for_service(Some(TIMEOUT)),
        "existing service worker did not re-register"
    );
    assert!(
        action.wait_for_action_server(Some(TIMEOUT)),
        "existing action worker did not re-register"
    );
    assert_eq!(
        client
            .call_with_id(b"after crash", Some("new-request"), Some(OP_TIMEOUT))
            .unwrap(),
        b"after crash",
    );
    let result = action
        .send_goal(
            b"new goal",
            Some("goal-after-crash"),
            Some(OP_TIMEOUT),
            None,
        )
        .unwrap()
        .wait_result()
        .unwrap();
    assert_eq!(result.body, b"new goal");
    assert!(
        entered.try_recv().is_err(),
        "interrupted request was replayed"
    );
    executor.stop().unwrap();
}

#[test]
fn silent_tcp_outage_reconnects_without_restarting_broker() {
    let _lock = lock_brokers();
    let broker = start_broker();
    let (handler, entered, release) = interrupted_echo();
    let (_worker, executor) = echo_server(&broker.api_url(), handler);
    let link = Link::new(broker.api_listen());
    let mut node = ws_client(
        "isolated-client",
        &format!("http://{}", link.address),
        &broker.service.frontend_bind,
    );
    assert!(node.wait_for_broker(Some(TIMEOUT)));
    let (_, received) = subscribe(&mut node, TOPIC, 8);
    let client = Arc::new(node.create_client_raw(SERVICE).unwrap());
    assert!(client.wait_for_service(Some(TIMEOUT)));
    assert_echo(&client, b"before outage");
    let publisher = Publisher::new(Some(&broker.message.xsub_bind)).unwrap();
    wait_for_topic(&mut node, &received, &publisher, b"before outage");

    with_interrupted_call(&client, &entered, || {
        link.online.send(false).unwrap();
    });
    wait_until("heartbeat loss to change connection state", TIMEOUT, || {
        node.connection_state() != ConnectionState::Connected
    });
    assert!(
        broker_http(&broker.api_url(), "/api/v1/discover").is_some(),
        "broker should stay online"
    );
    release.send(()).unwrap();
    link.online.send(true).unwrap();

    assert!(
        node.wait_for_broker(Some(TIMEOUT)),
        "WebSocket did not reconnect after restoring TCP"
    );
    wait_for_topic(&mut node, &received, &publisher, b"after outage");
    assert_eq!(
        client
            .call_with_id(b"after outage", Some("new-request"), Some(OP_TIMEOUT))
            .unwrap(),
        b"after outage",
    );
    assert!(
        entered.try_recv().is_err(),
        "interrupted request was replayed"
    );
    drop(node);
    executor.stop().unwrap();
    broker.stop().unwrap();
}

#[test]
fn slow_subscription_retains_only_newest_and_other_client_keeps_receiving() {
    let _lock = lock_brokers();
    let broker = start_broker();
    let url = broker.api_url();
    let (_worker, executor) = echo_server(&url, Arc::new(<[u8]>::to_vec));
    let mut slow = ws_client("slow-client", &url, &broker.service.frontend_bind);
    let mut fast = ws_client("fast-client", &url, &broker.service.frontend_bind);
    let prefix = "/reliability/";
    let (handle, slow_rx) = subscribe(&mut slow, TOPIC, 3);
    let (prefix_handle, prefix_rx) = subscribe(&mut slow, prefix, 1);
    let (_, fast_rx) = subscribe(&mut fast, TOPIC, 64);
    let client = slow.create_client_raw(SERVICE).unwrap();
    assert!(client.wait_for_service(Some(TIMEOUT)));
    let publisher = Publisher::new(Some(&broker.message.xsub_bind)).unwrap();
    wait_for_topic(&mut fast, &fast_rx, &publisher, b"warm-up");
    wait_for_topic(&mut slow, &slow_rx, &publisher, b"warm-up");

    let dequeued = |filter: &str, depth: u64| -> Option<u64> {
        broker_http(&url, "/api/v1/subscriptions")?["subscriptions"]
            .as_array()?
            .iter()
            .find(|row| row["filter"] == filter && row["capacity"] == depth)?["dequeued"]
            .as_u64()
    };
    wait_for_topic(&mut slow, &prefix_rx, &publisher, b"prefix warm-up");
    // The marker loop can deliver several copies in one spin, and the last
    // publish can still be sitting in the broker or client queue. Drain both
    // before the burst so keep_last is measured only on the numbered sequence.
    let pending = |filter: &str, depth: u64| -> Option<u64> {
        broker_http(&url, "/api/v1/subscriptions")?["subscriptions"]
            .as_array()?
            .iter()
            .find(|row| row["filter"] == filter && row["capacity"] == depth)?["pending"]
            .as_u64()
    };
    let mut quiet_dequeued = None;
    wait_until("slow client to finish warm-up", TIMEOUT, || {
        slow.spin_once(Some(PULSE)).unwrap();
        let leftover = slow_rx.try_iter().count() + prefix_rx.try_iter().count();
        let counts = match (dequeued(TOPIC, 3), dequeued(prefix, 1)) {
            (Some(topic), Some(prefix_count)) => Some((topic, prefix_count)),
            _ => None,
        };
        let quiet = leftover == 0
            && counts.is_some()
            && pending(TOPIC, 3) == Some(0)
            && pending(prefix, 1) == Some(0);
        // One empty sample can race the last marker publish. Require the
        // broker counters to stay unchanged across another spin.
        let stable = quiet && quiet_dequeued == counts;
        quiet_dequeued = if quiet { counts } else { None };
        stable
    });
    let baseline = dequeued(TOPIC, 3).expect("slow subscription metrics");
    let prefix_baseline = dequeued(prefix, 1).expect("slow prefix metrics");
    for seq in 1u64..=128 {
        let payload = seq.to_le_bytes();
        publisher.publish(TOPIC, &payload).unwrap();
        wait_until(
            "fast subscriber to receive each sequence",
            OP_TIMEOUT,
            || {
                fast.spin_once(Some(PULSE)).unwrap();
                fast_rx.try_iter().any(|received| received == payload)
            },
        );
    }
    wait_until(
        "broker to forward burst to the non-spinning client",
        TIMEOUT,
        || {
            dequeued(TOPIC, 3).is_some_and(|count| count >= baseline + 128)
                && dequeued(prefix, 1).is_some_and(|count| count >= prefix_baseline + 128)
        },
    );
    // The echo response crosses the same socket after the queued topic writes.
    assert_echo(&client, b"responsive during backlog");
    assert!(
        slow_rx.try_recv().is_err(),
        "slow application has not dispatched callbacks"
    );
    slow.spin_once(Some(Duration::ZERO)).unwrap();
    assert_eq!(
        slow_rx.try_iter().collect::<Vec<_>>(),
        (126u64..=128)
            .map(|seq| seq.to_le_bytes().to_vec())
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        prefix_rx.try_iter().collect::<Vec<_>>(),
        [128u64.to_le_bytes().to_vec()]
    );
    slow.destroy_subscription(handle).unwrap();
    slow.destroy_subscription(prefix_handle).unwrap();
    wait_until(
        "slow subscription to be removed from the broker",
        TIMEOUT,
        || dequeued(TOPIC, 3).is_none() && dequeued(prefix, 1).is_none(),
    );
    drop(slow);
    drop(fast);
    executor.stop().unwrap();
    broker.stop().unwrap();
}

#[cfg(target_os = "linux")]
mod soak {
    use super::*;

    const CHURN: &str = "/reliability/churn";
    const WARMUP: Duration = Duration::from_secs(5);
    const MEASUREMENT_WINDOWS: u64 = 6;

    #[derive(Clone, Copy, Debug)]
    struct Resources {
        rss_kib: u64,
        fds: usize,
    }

    impl Resources {
        fn measure() -> Self {
            let status = std::fs::read_to_string("/proc/self/status").unwrap();
            Self {
                rss_kib: status
                    .lines()
                    .find_map(|line| line.strip_prefix("VmRSS:"))
                    .expect("Linux process RSS")
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap(),
                fds: std::fs::read_dir("/proc/self/fd")
                    .unwrap()
                    .map(|entry| entry.expect("read process descriptor entry"))
                    .count(),
            }
        }
    }

    fn p95_us(mut values: Vec<u64>) -> u64 {
        values.sort_unstable();
        values[(values.len() * 95).div_ceil(100) - 1]
    }

    fn time_topic(node: &mut Node, rx: &Receiver<Vec<u8>>, tx: &Publisher, payload: &[u8]) -> u64 {
        let start = Instant::now();
        tx.publish(TOPIC, payload).unwrap();
        loop {
            node.spin_once(Some(Duration::ZERO)).unwrap();
            for received in rx.try_iter() {
                if received == b"warm-up" {
                    continue;
                }
                assert_eq!(
                    received, payload,
                    "unexpected sequence in sustained traffic"
                );
                return start.elapsed().as_micros() as u64;
            }
            assert!(
                start.elapsed() < OP_TIMEOUT,
                "topic sample was lost: {payload:?}"
            );
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn churn_subscription(node: &mut Node, url: &str) {
        let active = || {
            let stats = broker_http(url, "/api/v1/subscriptions")?;
            Some(
                stats["subscriptions"]
                    .as_array()?
                    .iter()
                    .any(|row| row["filter"] == CHURN),
            )
        };
        let handle = node
            .create_subscription_raw(CHURN, Arc::new(|_| {}), None)
            .unwrap();
        wait_until("temporary subscription registration", TIMEOUT, || {
            active() == Some(true)
        });
        node.destroy_subscription(handle).unwrap();
        wait_until("temporary subscription cleanup", TIMEOUT, || {
            active() == Some(false)
        });
    }

    #[test]
    #[ignore = "Linux soak; run explicitly in the reliability CI step"]
    fn short_soak_tracks_rss_fds_and_latency_drift() {
        let _lock = lock_brokers();
        let window_secs = std::env::var("ROBOT_BUS_SOAK_WINDOW_SECONDS").map_or(10, |value| {
            value
                .parse::<u64>()
                .expect("ROBOT_BUS_SOAK_WINDOW_SECONDS must be a whole number")
        });
        assert!(
            window_secs >= 5,
            "each measurement window needs at least five seconds"
        );
        let window = Duration::from_secs(window_secs);
        let broker = start_broker();
        let url = broker.api_url();
        let (_worker, executor) = echo_server(&url, Arc::new(<[u8]>::to_vec));
        let mut node = ws_client("soak-client", &url, &broker.service.frontend_bind);
        let (_, rx) = subscribe(&mut node, TOPIC, 16);
        let client = node.create_client_raw(SERVICE).unwrap();
        assert!(client.wait_for_service(Some(TIMEOUT)));
        let tx = Publisher::new(Some(&broker.message.xsub_bind)).unwrap();
        wait_for_topic(&mut node, &rx, &tx, b"warm-up");

        let mut sequence = 0u64;
        let mut baseline_resources = Resources::measure();
        let mut topic_p95 = Vec::new();
        let mut rpc_p95 = Vec::new();
        let mut peak = baseline_resources;
        let mut churn_at = Instant::now();
        for index in 0..=MEASUREMENT_WINDOWS {
            let start = Instant::now();
            let deadline = start + if index == 0 { WARMUP } else { window };
            let mut next_sample = start;
            let mut topic_us = Vec::new();
            let mut rpc_us = Vec::new();
            while Instant::now() < deadline {
                let payload = sequence.to_le_bytes();
                topic_us.push(time_topic(&mut node, &rx, &tx, &payload));
                if sequence.is_multiple_of(5) {
                    let start = Instant::now();
                    assert_echo(&client, &payload);
                    rpc_us.push(start.elapsed().as_micros() as u64);
                }
                sequence += 1;
                let now = Instant::now();
                if now >= churn_at {
                    churn_subscription(&mut node, &url);
                    let sample = Resources::measure();
                    peak.rss_kib = peak.rss_kib.max(sample.rss_kib);
                    peak.fds = peak.fds.max(sample.fds);
                    churn_at = now + Duration::from_secs(1);
                }
                next_sample += Duration::from_millis(25);
                let sleep = next_sample.saturating_duration_since(Instant::now());
                thread::sleep(sleep);
            }
            let resources = Resources::measure();
            if index == 0 {
                baseline_resources = resources;
                peak = resources;
                println!(
                    "reliability {}",
                    serde_json::json!({
                        "phase": "warmup", "rss_kib": resources.rss_kib, "fds": resources.fds,
                    })
                );
                continue;
            }
            let topic_samples = topic_us.len();
            let rpc_samples = rpc_us.len();
            assert!(
                topic_samples >= 25 && rpc_samples >= 5,
                "insufficient traffic in window {index}"
            );
            topic_p95.push(p95_us(topic_us));
            rpc_p95.push(p95_us(rpc_us));
            peak.rss_kib = peak.rss_kib.max(resources.rss_kib);
            peak.fds = peak.fds.max(resources.fds);
            println!(
                "reliability {}",
                serde_json::json!({
                    "phase": "measurement", "window": index,
                    "seconds": start.elapsed().as_secs_f64(),
                    "topic_samples": topic_samples, "rpc_samples": rpc_samples,
                    "topic_p95_us": topic_p95.last(), "rpc_p95_us": rpc_p95.last(),
                    "rss_kib": resources.rss_kib, "fds": resources.fds,
                    "peak_rss_kib": peak.rss_kib, "peak_fds": peak.fds,
                })
            );
        }
        assert!(
            peak.rss_kib <= baseline_resources.rss_kib + 32 * 1024,
            "RSS grew beyond 32 MiB after warmup: {baseline_resources:?} -> {peak:?}"
        );
        assert!(
            peak.fds <= baseline_resources.fds + 6,
            "descriptor count grew beyond six after warmup: {baseline_resources:?} -> {peak:?}"
        );
        for (name, p95) in [("topic", topic_p95), ("service", rpc_p95)] {
            let baseline = p95[0].max(p95[1]);
            let ending = p95[p95.len() - 2].max(p95[p95.len() - 1]);
            assert!(
                ending <= (3 * baseline).max(baseline + 20_000),
                "{name} p95 drifted after warmup: {p95:?} μs"
            );
        }

        drop(node);
        executor.stop().unwrap();
        broker.stop().unwrap();
    }
}
