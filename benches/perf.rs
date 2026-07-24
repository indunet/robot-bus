//! robot-bus performance harness — writes `docs/perf-report.md`.
//!
//! Run: `just perf` or `cargo run --release --bin robot_bus_perf`

#[path = "support.rs"]
mod support;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use robot_bus::worker_thread::WorkerThread;
use robot_bus::{Context, HighWaterMark, Node, Publisher, RobotBusBroker};
use support::{
    env_summary, lock_broker, node_for, now_ns, options_for, perf_broker_config, write_report,
    LatencyStats, ScenarioResult,
};

const PAYLOAD_LEN: usize = 64;
const WARMUP: usize = 50;
/// Paced one-way samples for message latency (firehose backlog is not latency).
const MSG_LATENCY_SAMPLES: usize = 5_000;
/// Deep-queue throughput: send this many; end stats when subscriber reaches MSG_RECV_TARGET.
const MSG_ITERS: usize = 100_000;
/// Early-complete threshold for throughput (best-effort may not deliver every send).
const MSG_RECV_TARGET: usize = 80_000;
const SVC_ITERS: usize = 100_000;
const ACT_ITERS: usize = 100_000;
/// Bench-only depth (≥ MSG_ITERS) so firehose is not dominated by HWM drops.
const MSG_HWM: i32 = 100_000;

fn main() {
    let _guard = lock_broker();
    let only_message = matches!(
        std::env::var("ROBOT_BUS_PERF_ONLY")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "message" | "msg" | "pubsub" | "pub/sub"
    );
    println!("starting RobotBusBroker (bind_all + grpc)…");
    if only_message {
        println!("ROBOT_BUS_PERF_ONLY=message — skipping service/action");
    }
    let ctx = Context::new();
    let broker =
        RobotBusBroker::start_with_context(ctx.clone(), perf_broker_config()).expect("start broker");
    thread::sleep(Duration::from_millis(300));

    let mut results: Vec<ScenarioResult> = Vec::new();

    for transport in ["tcp", "ipc", "inproc"] {
        println!("=== {transport} ===");
        results.push(bench_pubsub(&ctx, transport));
        if !only_message {
            results.push(bench_service(&ctx, transport, SVC_ITERS));
            results.push(bench_action(&ctx, transport, ACT_ITERS));
        }
    }

    let grpc_url = broker.grpc_url();
    println!("=== grpc ({grpc_url}) ===");
    results.push(bench_grpc_subscribe(&broker, &grpc_url));
    if !only_message {
        results.push(bench_grpc_service(&broker, &grpc_url, SVC_ITERS));
        results.push(bench_grpc_action(&broker, &grpc_url, ACT_ITERS));
    }

    broker.stop().expect("stop broker");

    for r in &results {
        if let Some(note) = &r.note {
            println!("[{}/{}] SKIP: {note}", r.transport, r.scenario);
        } else if r.kind == support::ScenarioKind::Message {
            println!(
                "[{}/{}] sent={} recv={} pub={:.0}/s sub={:.0}/s delivery={:.1}% p50={:.0}µs p99={:.0}µs",
                r.transport,
                r.scenario,
                r.sent,
                r.received,
                r.publish_per_s,
                r.subscribe_per_s,
                r.delivery_pct,
                r.latency.p50_us,
                r.latency.p99_us,
            );
        } else {
            println!(
                "[{}/{}] n={} got={} {:.0}/s p50={:.0}µs p99={:.0}µs",
                r.transport,
                r.scenario,
                r.sent,
                r.received,
                r.subscribe_per_s,
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

fn bench_pubsub(ctx: &Context, transport: &str) -> ScenarioResult {
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

    let latencies = Arc::new(Mutex::new(Vec::<u64>::with_capacity(MSG_LATENCY_SAMPLES)));
    let count = Arc::new(AtomicUsize::new(0));
    let record_latency = Arc::new(AtomicBool::new(true));

    let mut sub = node_for(ctx, format!("perf-sub-{transport}"), &transport);
    let hwm = HighWaterMark {
        snd: MSG_HWM,
        rcv: MSG_HWM,
    };
    if let Err(err) = sub.set_stream_hwm(hwm) {
        return ScenarioResult::skipped(&transport, scenario, format!("set_stream_hwm: {err}"));
    }
    let lat_cb = Arc::clone(&latencies);
    let cnt_cb = Arc::clone(&count);
    let rec_cb = Arc::clone(&record_latency);
    if let Err(err) = sub.create_subscription_raw(
        &topic,
        Arc::new(move |_topic, payload| {
            if rec_cb.load(Ordering::Relaxed) {
                if let Some(sent) = read_ts(payload) {
                    let now = now_ns();
                    if now >= sent {
                        lat_cb.lock().unwrap().push(now - sent);
                    }
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

    let publisher = match Publisher::with_shared_context(ctx, Some(&xsub), hwm) {
        Ok(p) => p,
        Err(err) => {
            return ScenarioResult::skipped(&transport, scenario, format!("publisher failed: {err}"))
        }
    };

    let count_pub = Arc::clone(&count);
    let lat_pub = Arc::clone(&latencies);
    let rec_pub = Arc::clone(&record_latency);
    let topic_pub = topic.clone();
    let transport_pub = transport.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(250));
        for _ in 0..WARMUP {
            let _ = publisher.publish(&topic_pub, &make_payload(now_ns()));
        }
        // Warmup may drop under modest HWM; do not require full receive.
        thread::sleep(Duration::from_millis(100));
        count_pub.store(0, Ordering::Relaxed);
        lat_pub.lock().unwrap().clear();

        // Phase 1: paced one-way latency (send → wait for that receive).
        rec_pub.store(true, Ordering::Relaxed);
        for _ in 0..MSG_LATENCY_SAMPLES {
            let before = count_pub.load(Ordering::Relaxed);
            if publisher
                .publish(&topic_pub, &make_payload(now_ns()))
                .is_err()
            {
                shutdown.shutdown();
                return Err("publish failed (latency phase)".to_string());
            }
            if !wait_until(&count_pub, before + 1, Duration::from_secs(2)) {
                shutdown.shutdown();
                return Err("latency sample timed out".to_string());
            }
        }
        let latency = LatencyStats::from_ns(lat_pub.lock().unwrap().clone());

        // Phase 2: deep-queue throughput — send MSG_ITERS, end when recv >= MSG_RECV_TARGET.
        rec_pub.store(false, Ordering::Relaxed);
        count_pub.store(0, Ordering::Relaxed);
        lat_pub.lock().unwrap().clear();

        println!(
            "  … {transport_pub} throughput: send {MSG_ITERS}, end at recv>={MSG_RECV_TARGET} (HWM={MSG_HWM})"
        );
        let t0 = Instant::now();
        let mut sent = 0usize;
        for _ in 0..MSG_ITERS {
            if publisher
                .publish(&topic_pub, &make_payload(now_ns()))
                .is_err()
            {
                break;
            }
            sent += 1;
        }
        let ok = wait_until(&count_pub, MSG_RECV_TARGET, Duration::from_secs(120));
        let elapsed = t0.elapsed();
        let received = count_pub.load(Ordering::Relaxed);
        shutdown.shutdown();

        if received == 0 {
            return Ok(ScenarioResult::skipped(
                &transport_pub,
                scenario,
                "timed out with 0 messages",
            ));
        }
        if !ok {
            println!(
                "  … {transport_pub} incomplete: recv={received}/{MSG_RECV_TARGET} sent={sent}"
            );
        }

        Ok(ScenarioResult::ok_message(
            &transport_pub,
            scenario,
            sent,
            received,
            elapsed,
            latency,
        ))
    });

    let _ = sub.spin();
    match worker.join().expect("publisher thread") {
        Ok(result) => result,
        Err(note) => ScenarioResult::skipped(&transport, scenario, note),
    }
}

fn bench_service(ctx: &Context, transport: &str, n: usize) -> ScenarioResult {
    let scenario = "service call";
    let transport = transport.to_string();
    let name = format!("perf.{transport}.echo");

    let mut server = node_for(ctx, format!("perf-svc-srv-{transport}"), &transport);
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
    let ctx_cli = ctx.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let mut client_node =
            node_for(&ctx_cli, format!("perf-svc-cli-{transport_cli}"), &transport_cli);
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
        // Fail fast: one probe before full warmup.
        if let Err(err) = client.call(&payload, Some(Duration::from_millis(500))) {
            shutdown.shutdown();
            return ScenarioResult::skipped(
                &transport_cli,
                scenario,
                format!("call failed: {err}"),
            );
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

        ScenarioResult::ok_rpc(
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

fn bench_action(ctx: &Context, transport: &str, n: usize) -> ScenarioResult {
    let scenario = "action send_goal";
    let transport = transport.to_string();
    let name = format!("perf.{transport}.act");

    let mut server = node_for(ctx, format!("perf-act-srv-{transport}"), &transport);
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
    let ctx_cli = ctx.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let mut client_node =
            node_for(&ctx_cli, format!("perf-act-cli-{transport_cli}"), &transport_cli);
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
            return ScenarioResult::skipped(
                &transport_cli,
                scenario,
                format!("send_goal failed: {err}"),
            );
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

        ScenarioResult::ok_rpc(
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

fn bench_grpc_subscribe(broker: &RobotBusBroker, url: &str) -> ScenarioResult {
    let transport = "grpc";
    let scenario = "message Subscribe";
    let topic = "perf/grpc/msg";

    let latencies = Arc::new(Mutex::new(Vec::<u64>::with_capacity(MSG_LATENCY_SAMPLES)));
    let count = Arc::new(AtomicUsize::new(0));
    let record_latency = Arc::new(AtomicBool::new(true));

    let mut node = Node::grpc_at("perf-grpc-sub", url);
    if let Err(err) = node.create_subscription_raw(
        topic,
        Arc::new({
            let latencies = Arc::clone(&latencies);
            let count = Arc::clone(&count);
            let record_latency = Arc::clone(&record_latency);
            move |_t, payload| {
                if record_latency.load(Ordering::Relaxed) {
                    if let Some(sent) = read_ts(payload) {
                        let now = now_ns();
                        if now >= sent {
                            latencies.lock().unwrap().push(now - sent);
                        }
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

    // Phase 1: paced one-way latency.
    record_latency.store(true, Ordering::Relaxed);
    for _ in 0..MSG_LATENCY_SAMPLES {
        let before = count.load(Ordering::Relaxed);
        let _ = publisher.publish(topic, &make_payload(now_ns()));
        let deadline = Instant::now() + Duration::from_secs(2);
        while count.load(Ordering::Relaxed) <= before && Instant::now() < deadline {
            let _ = node.spin_once(Some(Duration::from_millis(1)));
        }
        if count.load(Ordering::Relaxed) <= before {
            return ScenarioResult::skipped(transport, scenario, "latency sample timed out");
        }
    }
    let latency = LatencyStats::from_ns(latencies.lock().unwrap().clone());

    // Phase 2: deep-queue throughput — send N, spin until recv N.
    record_latency.store(false, Ordering::Relaxed);
    count.store(0, Ordering::Relaxed);
    latencies.lock().unwrap().clear();

    let topic_pub = topic.to_string();
    let publisher_thread = thread::spawn(move || {
        let mut sent = 0usize;
        for _ in 0..MSG_ITERS {
            if publisher
                .publish(&topic_pub, &make_payload(now_ns()))
                .is_err()
            {
                break;
            }
            sent += 1;
        }
        sent
    });

    println!(
        "  … grpc throughput: send {MSG_ITERS}, end at recv>={MSG_RECV_TARGET} (HWM={MSG_HWM})"
    );
    let t0 = Instant::now();
    let deadline = t0 + Duration::from_secs(120);
    while count.load(Ordering::Relaxed) < MSG_RECV_TARGET && Instant::now() < deadline {
        let _ = node.spin_once(Some(Duration::from_millis(1)));
    }
    let elapsed = t0.elapsed();
    let sent_n = publisher_thread.join().unwrap_or(0);
    let received = count.load(Ordering::Relaxed);

    if received == 0 {
        return ScenarioResult::skipped(transport, scenario, "timed out with 0 messages");
    }
    if received < MSG_RECV_TARGET {
        println!(
            "  … grpc incomplete: recv={received}/{MSG_RECV_TARGET} sent={sent_n}"
        );
    }

    ScenarioResult::ok_message(
        transport,
        scenario,
        sent_n,
        received,
        elapsed,
        latency,
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

    ScenarioResult::ok_rpc(
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

    ScenarioResult::ok_rpc(
        transport,
        scenario,
        n,
        received,
        elapsed,
        LatencyStats::from_ns(samples),
    )
}
