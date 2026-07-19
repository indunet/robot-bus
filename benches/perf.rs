//! robot-bus performance harness — writes `docs/perf-report.md`.
//!
//! Run: `just perf` or `cargo run --release --bin robot_bus_perf`

#[path = "support.rs"]
mod support;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use robot_bus::worker_thread::WorkerThread;
use robot_bus::{HighWaterMark, Node, Publisher, RobotBusBroker};
use support::{
    env_summary, lock_broker, now_ns, options_for, perf_broker_config, write_report, LatencyStats,
    ScenarioResult,
};

const PAYLOAD_LEN: usize = 64;
const WARMUP: usize = 50;

// Same iteration counts for tcp / ipc / inproc / grpc so results are comparable.
// Message: firehose 1万条，收到 1万条即结束（不等待逐条 ACK）。
const MSG_ITERS: usize = 10_000;
const SVC_ITERS: usize = 5_000;
const ACT_ITERS: usize = 1_000;
/// Deep queues so a 10k firehose is less likely to drop before the subscriber drains.
const MSG_HWM: i32 = 100_000;

fn main() {
    let _guard = lock_broker();
    println!("starting RobotBusBroker (bind_all + grpc)…");
    let broker = RobotBusBroker::start(perf_broker_config()).expect("start broker");
    thread::sleep(Duration::from_millis(300));

    let mut results: Vec<ScenarioResult> = Vec::new();

    for transport in ["tcp", "ipc", "inproc"] {
        println!("=== {transport} ===");
        results.push(bench_pubsub(transport, MSG_ITERS));
        results.push(bench_service(transport, SVC_ITERS));
        results.push(bench_action(transport, ACT_ITERS));
    }

    let grpc_url = format!("http://{}", broker.grpc_listen());
    println!("=== grpc ({grpc_url}) ===");
    results.push(bench_grpc_subscribe(&broker, &grpc_url, MSG_ITERS));
    results.push(bench_grpc_service(&broker, &grpc_url, SVC_ITERS));
    results.push(bench_grpc_action(&broker, &grpc_url, ACT_ITERS));

    broker.stop().expect("stop broker");

    for r in &results {
        if let Some(note) = &r.note {
            println!("[{}/{}] SKIP: {note}", r.transport, r.scenario);
        } else {
            println!(
                "[{}/{}] n={} got={} {:.0}/s p50={:.1}µs p99={:.1}µs",
                r.transport,
                r.scenario,
                r.iterations,
                r.received,
                r.throughput_per_s,
                r.latency.p50_us,
                r.latency.p99_us,
            );
        }
    }

    let path = write_report(&results, &env_summary()).expect("write report");
    println!("wrote {}", path.display());
}

fn make_payload(ts_ns: u64) -> Vec<u8> {
    let mut buf = vec![0u8; PAYLOAD_LEN];
    buf[..8].copy_from_slice(&ts_ns.to_le_bytes());
    buf
}

fn read_ts(payload: &[u8]) -> Option<u64> {
    if payload.len() < 8 {
        return None;
    }
    let mut b = [0u8; 8];
    b.copy_from_slice(&payload[..8]);
    Some(u64::from_le_bytes(b))
}

fn wait_until(count: &AtomicUsize, target: usize, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while count.load(Ordering::Relaxed) < target {
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(1));
    }
    true
}

fn bench_pubsub(transport: &str, n: usize) -> ScenarioResult {
    let scenario = "message pub/sub";
    let transport = transport.to_string();
    let options = options_for(&transport);
    let topic = format!("perf/{transport}/msg");

    let xsub = match options.message_xsub_endpoint() {
        Ok(ep) => ep,
        Err(err) => {
            return ScenarioResult::skipped(&transport, scenario, format!("xsub endpoint: {err}"))
        }
    };

    let latencies = Arc::new(Mutex::new(Vec::<u64>::with_capacity(n)));
    let count = Arc::new(AtomicUsize::new(0));

    let mut sub = Node::with_options(format!("perf-sub-{transport}"), options);
    let lat_cb = Arc::clone(&latencies);
    let cnt_cb = Arc::clone(&count);
    if let Err(err) = sub.create_subscription_raw(
        &topic,
        Arc::new(move |_topic, payload| {
            if let Some(sent) = read_ts(payload) {
                let now = now_ns();
                if now >= sent {
                    lat_cb.lock().unwrap().push(now - sent);
                }
            }
            cnt_cb.fetch_add(1, Ordering::Relaxed);
        }),
        None,
    ) {
        return ScenarioResult::skipped(&transport, scenario, format!("subscribe failed: {err}"));
    }

    let shutdown = match sub.shutdown_handle() {
        Ok(h) => h,
        Err(err) => {
            return ScenarioResult::skipped(&transport, scenario, format!("shutdown handle: {err}"))
        }
    };

    let hwm = HighWaterMark {
        snd: MSG_HWM,
        rcv: MSG_HWM,
    };
    let publisher = match Publisher::with_hwm(Some(&xsub), hwm) {
        Ok(p) => p,
        Err(err) => {
            return ScenarioResult::skipped(&transport, scenario, format!("publisher failed: {err}"))
        }
    };

    let count_pub = Arc::clone(&count);
    let lat_pub = Arc::clone(&latencies);
    let topic_pub = topic.clone();
    let transport_pub = transport.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(250));
        for _ in 0..WARMUP {
            let _ = publisher.publish(&topic_pub, &make_payload(now_ns()));
        }
        thread::sleep(Duration::from_millis(100));
        count_pub.store(0, Ordering::Relaxed);
        lat_pub.lock().unwrap().clear();

        // Firehose: blast all N publishes, then wait until the subscriber has N.
        let t0 = Instant::now();
        for _ in 0..n {
            if publisher
                .publish(&topic_pub, &make_payload(now_ns()))
                .is_err()
            {
                shutdown.shutdown();
                return Err("publish failed".to_string());
            }
        }
        let wait = if transport_pub == "inproc" {
            Duration::from_secs(3)
        } else {
            Duration::from_secs(120)
        };
        let ok = wait_until(&count_pub, n, wait);
        let elapsed = t0.elapsed();
        let received = count_pub.load(Ordering::Relaxed);
        let samples = lat_pub.lock().unwrap().clone();
        shutdown.shutdown();

        if !ok && received == 0 {
            let note = if transport_pub == "inproc" {
                "no messages received (ZMQ inproc is context-local; SDK nodes use separate contexts from the broker)"
                    .to_string()
            } else {
                "timed out with 0 messages".to_string()
            };
            return Ok(ScenarioResult::skipped(&transport_pub, scenario, note));
        }

        Ok(ScenarioResult::ok(
            &transport_pub,
            scenario,
            n,
            received,
            elapsed,
            LatencyStats::from_ns(samples),
        ))
    });

    let _ = sub.spin();
    match worker.join().expect("publisher thread") {
        Ok(result) => result,
        Err(note) => ScenarioResult::skipped(&transport, scenario, note),
    }
}

fn bench_service(transport: &str, n: usize) -> ScenarioResult {
    let scenario = "service call";
    let transport = transport.to_string();
    let options = options_for(&transport);
    let name = format!("perf.{transport}.echo");

    let mut server = Node::with_options(format!("perf-svc-srv-{transport}"), options.clone());
    if let Err(err) = server.create_service_raw(&name, Arc::new(|body| body.to_vec()), None) {
        return ScenarioResult::skipped(&transport, scenario, format!("create_service: {err}"));
    }

    let shutdown = match server.shutdown_handle() {
        Ok(h) => h,
        Err(err) => {
            return ScenarioResult::skipped(&transport, scenario, format!("shutdown handle: {err}"))
        }
    };

    let transport_cli = transport.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let mut client_node =
            Node::with_options(format!("perf-svc-cli-{transport_cli}"), options);
        let client = match client_node.create_client_raw(&name) {
            Ok(c) => c,
            Err(err) => {
                shutdown.shutdown();
                return ScenarioResult::skipped(
                    &transport_cli,
                    scenario,
                    format!("create_client: {err}"),
                );
            }
        };

        let payload = make_payload(0);
        // Fail fast: one probe before full warmup (avoids stacking timeouts on inproc).
        if let Err(err) = client.call(&payload, Some(Duration::from_millis(500))) {
            shutdown.shutdown();
            let note = if transport_cli == "inproc" {
                format!("call failed (likely inproc context isolation): {err}")
            } else {
                format!("call failed: {err}")
            };
            return ScenarioResult::skipped(&transport_cli, scenario, note);
        }
        for _ in 1..WARMUP {
            let _ = client.call(&payload, Some(Duration::from_secs(2)));
        }

        let mut samples = Vec::with_capacity(n);
        let t0 = Instant::now();
        let mut received = 0usize;
        for _ in 0..n {
            let start = Instant::now();
            match client.call(&payload, Some(Duration::from_secs(5))) {
                Ok(_) => {
                    samples.push(start.elapsed().as_nanos() as u64);
                    received += 1;
                }
                Err(err) => {
                    shutdown.shutdown();
                    if received == 0 {
                        return ScenarioResult::skipped(
                            &transport_cli,
                            scenario,
                            format!("call failed: {err}"),
                        );
                    }
                    break;
                }
            }
        }
        let elapsed = t0.elapsed();
        shutdown.shutdown();

        if received == 0 {
            return ScenarioResult::skipped(&transport_cli, scenario, "0 successful calls");
        }

        ScenarioResult::ok(
            &transport_cli,
            scenario,
            n,
            received,
            elapsed,
            LatencyStats::from_ns(samples),
        )
    });

    let _ = server.spin();
    worker.join().expect("service client thread")
}

fn bench_action(transport: &str, n: usize) -> ScenarioResult {
    let scenario = "action send_goal";
    let transport = transport.to_string();
    let options = options_for(&transport);
    let name = format!("perf.{transport}.act");

    let mut server = Node::with_options(format!("perf-act-srv-{transport}"), options.clone());
    if let Err(err) = server.create_action_server_raw(
        &name,
        Arc::new(|body| {
            vec![
                ("FEEDBACK".into(), b"f".to_vec()),
                ("RESULT".into(), body.to_vec()),
            ]
        }),
        None,
    ) {
        return ScenarioResult::skipped(
            &transport,
            scenario,
            format!("create_action_server: {err}"),
        );
    }

    let shutdown = match server.shutdown_handle() {
        Ok(h) => h,
        Err(err) => {
            return ScenarioResult::skipped(&transport, scenario, format!("shutdown handle: {err}"))
        }
    };

    let transport_cli = transport.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let mut client_node =
            Node::with_options(format!("perf-act-cli-{transport_cli}"), options);
        let client = match client_node.create_action_client_raw(&name) {
            Ok(c) => c,
            Err(err) => {
                shutdown.shutdown();
                return ScenarioResult::skipped(
                    &transport_cli,
                    scenario,
                    format!("create_action_client: {err}"),
                );
            }
        };

        let payload = make_payload(0);
        if let Err(err) = client.send_goal(&payload, None, Some(Duration::from_millis(800))) {
            shutdown.shutdown();
            let note = if transport_cli == "inproc" {
                format!("send_goal failed (likely inproc context isolation): {err}")
            } else {
                format!("send_goal failed: {err}")
            };
            return ScenarioResult::skipped(&transport_cli, scenario, note);
        }
        for _ in 1..WARMUP.min(10) {
            let _ = client.send_goal(&payload, None, Some(Duration::from_secs(2)));
        }

        let mut samples = Vec::with_capacity(n);
        let t0 = Instant::now();
        let mut received = 0usize;
        for _ in 0..n {
            let start = Instant::now();
            match client.send_goal(&payload, None, Some(Duration::from_secs(5))) {
                Ok(_) => {
                    samples.push(start.elapsed().as_nanos() as u64);
                    received += 1;
                }
                Err(err) => {
                    shutdown.shutdown();
                    if received == 0 {
                        return ScenarioResult::skipped(
                            &transport_cli,
                            scenario,
                            format!("send_goal failed: {err}"),
                        );
                    }
                    break;
                }
            }
        }
        let elapsed = t0.elapsed();
        shutdown.shutdown();

        if received == 0 {
            return ScenarioResult::skipped(&transport_cli, scenario, "0 successful goals");
        }

        ScenarioResult::ok(
            &transport_cli,
            scenario,
            n,
            received,
            elapsed,
            LatencyStats::from_ns(samples),
        )
    });

    let _ = server.spin();
    worker.join().expect("action client thread")
}

fn bench_grpc_subscribe(broker: &RobotBusBroker, url: &str, n: usize) -> ScenarioResult {
    let transport = "grpc";
    let scenario = "message Subscribe";
    let topic = "perf/grpc/msg";

    let latencies = Arc::new(Mutex::new(Vec::<u64>::with_capacity(n)));
    let count = Arc::new(AtomicUsize::new(0));

    let mut node = Node::grpc_at("perf-grpc-sub", url);
    if let Err(err) = node.create_subscription_raw(
        topic,
        Arc::new({
            let latencies = Arc::clone(&latencies);
            let count = Arc::clone(&count);
            move |_t, payload| {
                if let Some(sent) = read_ts(payload) {
                    let now = now_ns();
                    if now >= sent {
                        latencies.lock().unwrap().push(now - sent);
                    }
                }
                count.fetch_add(1, Ordering::Relaxed);
            }
        }),
        None,
    ) {
        return ScenarioResult::skipped(transport, scenario, format!("subscribe: {err}"));
    }

    thread::sleep(Duration::from_millis(400));

    let hwm = HighWaterMark {
        snd: MSG_HWM,
        rcv: MSG_HWM,
    };
    let publisher = match Publisher::with_hwm(Some(&broker.message.xsub_bind), hwm) {
        Ok(p) => p,
        Err(err) => {
            return ScenarioResult::skipped(transport, scenario, format!("publisher: {err}"))
        }
    };

    for _ in 0..WARMUP {
        let _ = publisher.publish(topic, &make_payload(now_ns()));
        let _ = node.spin_once(Some(Duration::from_millis(5)));
    }
    thread::sleep(Duration::from_millis(50));
    count.store(0, Ordering::Relaxed);
    latencies.lock().unwrap().clear();

    // Firehose on a side thread; this thread only spins until N arrivals.
    let topic_pub = topic.to_string();
    let publisher_thread = thread::spawn(move || {
        for _ in 0..n {
            let _ = publisher.publish(&topic_pub, &make_payload(now_ns()));
        }
    });

    let t0 = Instant::now();
    let deadline = Instant::now() + Duration::from_secs(120);
    while count.load(Ordering::Relaxed) < n && Instant::now() < deadline {
        let _ = node.spin_once(Some(Duration::from_millis(5)));
    }
    let elapsed = t0.elapsed();
    let _ = publisher_thread.join();
    let received = count.load(Ordering::Relaxed);
    let samples = latencies.lock().unwrap().clone();

    if received == 0 {
        return ScenarioResult::skipped(transport, scenario, "timed out with 0 messages");
    }

    ScenarioResult::ok(
        transport,
        scenario,
        n,
        received,
        elapsed,
        LatencyStats::from_ns(samples),
    )
}

fn bench_grpc_service(broker: &RobotBusBroker, url: &str, n: usize) -> ScenarioResult {
    let transport = "grpc";
    let scenario = "service Call";
    let name = "perf.grpc.echo";

    let handler: Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync> = Arc::new(|body| body.to_vec());
    let worker = match WorkerThread::spawn_service(name, handler, &broker.service.backend_bind) {
        Ok(w) => w,
        Err(err) => {
            return ScenarioResult::skipped(transport, scenario, format!("worker: {err}"))
        }
    };
    thread::sleep(Duration::from_millis(150));

    let mut node = Node::grpc_at("perf-grpc-cli", url);
    let client = match node.create_client_raw(name) {
        Ok(c) => c,
        Err(err) => {
            worker.stop();
            return ScenarioResult::skipped(transport, scenario, format!("create_client: {err}"));
        }
    };

    let payload = make_payload(0);
    for _ in 0..WARMUP {
        let _ = client.call(&payload, Some(Duration::from_secs(2)));
    }

    let mut samples = Vec::with_capacity(n);
    let t0 = Instant::now();
    let mut received = 0usize;
    for _ in 0..n {
        let start = Instant::now();
        match client.call(&payload, Some(Duration::from_secs(5))) {
            Ok(_) => {
                samples.push(start.elapsed().as_nanos() as u64);
                received += 1;
            }
            Err(err) => {
                if received == 0 {
                    worker.stop();
                    return ScenarioResult::skipped(
                        transport,
                        scenario,
                        format!("call failed: {err}"),
                    );
                }
                break;
            }
        }
    }
    let elapsed = t0.elapsed();
    worker.stop();

    ScenarioResult::ok(
        transport,
        scenario,
        n,
        received,
        elapsed,
        LatencyStats::from_ns(samples),
    )
}

fn bench_grpc_action(broker: &RobotBusBroker, url: &str, n: usize) -> ScenarioResult {
    let transport = "grpc";
    let scenario = "action Run";
    let name = "perf.grpc.act";

    let handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> = Arc::new(|body| {
        vec![
            ("FEEDBACK".into(), b"f".to_vec()),
            ("RESULT".into(), body.to_vec()),
        ]
    });
    let worker = match WorkerThread::spawn_action(name, handler, &broker.action.backend_bind) {
        Ok(w) => w,
        Err(err) => {
            return ScenarioResult::skipped(transport, scenario, format!("worker: {err}"))
        }
    };
    thread::sleep(Duration::from_millis(150));

    let mut node = Node::grpc_at("perf-grpc-act", url);
    let client = match node.create_action_client_raw(name) {
        Ok(c) => c,
        Err(err) => {
            worker.stop();
            return ScenarioResult::skipped(
                transport,
                scenario,
                format!("create_action_client: {err}"),
            );
        }
    };

    let payload = make_payload(0);
    for _ in 0..WARMUP.min(10) {
        let _ = client.send_goal(&payload, None, Some(Duration::from_secs(2)));
    }

    let mut samples = Vec::with_capacity(n);
    let t0 = Instant::now();
    let mut received = 0usize;
    for _ in 0..n {
        let start = Instant::now();
        match client.send_goal(&payload, None, Some(Duration::from_secs(5))) {
            Ok(_) => {
                samples.push(start.elapsed().as_nanos() as u64);
                received += 1;
            }
            Err(err) => {
                if received == 0 {
                    worker.stop();
                    return ScenarioResult::skipped(
                        transport,
                        scenario,
                        format!("send_goal failed: {err}"),
                    );
                }
                break;
            }
        }
    }
    let elapsed = t0.elapsed();
    worker.stop();

    ScenarioResult::ok(
        transport,
        scenario,
        n,
        received,
        elapsed,
        LatencyStats::from_ns(samples),
    )
}
