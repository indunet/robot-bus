//! robot-bus performance harness — writes `docs/zh/perf-report.md` and `docs/en/perf-report.md`.
//!
//! Sources live under `benches/robot_bus_perf/`.
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
    LatencyStats, ScenarioResult, env_f64, env_summary, env_usize, lock_broker, node_for, now_ns,
    options_for, perf_broker_config, write_report,
};

const PAYLOAD_LEN: usize = 64;
const WARMUP: usize = 50;
/// Bench-only depth. Keep modest vs trial length so queues cannot hide overload.
const MSG_HWM: i32 = 2_048;

fn svc_iters() -> usize {
    env_usize("ROBOT_BUS_PERF_SVC_ITERS", 10_000)
}

fn act_iters() -> usize {
    env_usize("ROBOT_BUS_PERF_ACT_ITERS", 5_000)
}

fn msg_latency_samples() -> usize {
    env_usize("ROBOT_BUS_PERF_MSG_LATENCY_SAMPLES", 5_000)
}

fn max_loss_pct() -> f64 {
    env_f64("ROBOT_BUS_PERF_MAX_LOSS_PCT", 1.0)
}

fn goodput_trial_msgs() -> usize {
    // Fallback when duration-based sizing is not used.
    env_usize("ROBOT_BUS_PERF_GOODPUT_TRIAL_MSGS", 0)
}

fn goodput_trial_secs() -> f64 {
    env_f64("ROBOT_BUS_PERF_GOODPUT_TRIAL_SECS", 1.0)
}

fn goodput_rate_lo() -> u64 {
    env_usize("ROBOT_BUS_PERF_GOODPUT_RATE_LO", 1_000) as u64
}

fn goodput_rate_hi() -> u64 {
    env_usize("ROBOT_BUS_PERF_GOODPUT_RATE_HI", 2_000_000) as u64
}

fn goodput_settle() -> Duration {
    Duration::from_millis(env_usize("ROBOT_BUS_PERF_GOODPUT_SETTLE_MS", 100) as u64)
}

/// Fixed message count when `ROBOT_BUS_PERF_GOODPUT_TRIAL_MSGS` is set (smoke).
/// Default path uses [`publish_paced_for`] (wall-clock duration), not a capped count —
/// a 50k cap made MHz targets finish in ~40ms and inflated reported goodput.
fn trial_msg_count(rate_hz: u64) -> Option<usize> {
    let fixed = goodput_trial_msgs();
    if fixed > 0 {
        Some(fixed)
    } else {
        let _ = rate_hz;
        None
    }
}

fn main() {
    let _guard = lock_broker();
    let only_message = matches!(
        std::env::var("ROBOT_BUS_PERF_ONLY")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "message" | "msg" | "pubsub" | "pub/sub"
    );
    println!("starting RobotBusBroker (bind_all + ws)…");
    let svc_iters = svc_iters();
    let act_iters = act_iters();
    if only_message {
        println!("ROBOT_BUS_PERF_ONLY=message — skipping service/action");
    } else {
        println!("service iters={svc_iters} action iters={act_iters}");
    }
    let ctx = Context::new();
    let broker =
        RobotBusBroker::start_with_context(&ctx, perf_broker_config()).expect("start broker");
    thread::sleep(Duration::from_millis(300));

    let mut results: Vec<ScenarioResult> = Vec::new();

    for transport in ["tcp", "ipc", "inproc"] {
        println!("=== {transport} ===");
        results.push(bench_pubsub(&ctx, transport));
        if !only_message {
            results.push(bench_service(&ctx, transport, svc_iters));
            results.push(bench_action(&ctx, transport, act_iters));
        }
    }

    let ws_url = broker.api_url();
    println!("=== ws ({ws_url}) ===");
    results.push(bench_ws_subscribe(&broker, &ws_url));
    if !only_message {
        results.push(bench_ws_service(&broker, &ws_url, svc_iters));
        results.push(bench_ws_action(&broker, &ws_url, act_iters));
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
    println!("wrote {} (and docs/en/perf-report.md)", path.display());
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

fn loss_pct(sent: usize, received: usize) -> f64 {
    if sent == 0 {
        100.0
    } else if received >= sent {
        0.0
    } else {
        100.0 * (sent - received) as f64 / sent as f64
    }
}

/// Publish aiming for `rate_hz` until `duration` elapses (sustained trial).
fn publish_paced_for(
    publisher: &Publisher,
    topic: &str,
    rate_hz: f64,
    duration: Duration,
) -> (usize, Duration) {
    let interval = Duration::from_secs_f64(1.0 / rate_hz.max(1.0));
    let t0 = Instant::now();
    let deadline = t0 + duration;
    let mut next = t0;
    let mut sent = 0usize;
    while Instant::now() < deadline {
        if publisher.publish(topic, &make_payload(now_ns())).is_err() {
            break;
        }
        sent += 1;
        next += interval;
        wait_deadline(next);
    }
    (sent, t0.elapsed())
}

/// Publish `n` messages aiming for `rate_hz`. Uses sleep for coarse gaps and
/// busy-wait for sub-ms deadlines (macOS sleep granularity is ~1ms).
fn publish_paced(publisher: &Publisher, topic: &str, n: usize, rate_hz: f64) -> (usize, Duration) {
    let interval = Duration::from_secs_f64(1.0 / rate_hz.max(1.0));
    let t0 = Instant::now();
    let mut next = t0;
    let mut sent = 0usize;
    for _ in 0..n {
        if publisher.publish(topic, &make_payload(now_ns())).is_err() {
            break;
        }
        sent += 1;
        next += interval;
        wait_deadline(next);
    }
    (sent, t0.elapsed())
}

fn wait_deadline(deadline: Instant) {
    loop {
        let now = Instant::now();
        if now >= deadline {
            return;
        }
        let remain = deadline - now;
        if remain > Duration::from_millis(2) {
            thread::sleep(remain - Duration::from_millis(1));
        } else {
            std::hint::spin_loop();
        }
    }
}

struct GoodputTrial {
    target_hz: u64,
    sent: usize,
    /// Receives observed by end of the send window (before settle).
    received_at_send_end: usize,
    /// Receives after settle (loss / delivery).
    received: usize,
    elapsed: Duration,
}

fn trial_sustains_rate(t: &GoodputTrial) -> bool {
    let secs = t.elapsed.as_secs_f64().max(1e-9);
    let pub_rate = t.sent as f64 / secs;
    // Subscriber must keep up *during* the send window — settle must not be what
    // "saves" a burst that only fit in the ZMQ HWM.
    let sub_rate = t.received_at_send_end as f64 / secs;
    let target = t.target_hz as f64;
    pub_rate >= 0.90 * target && sub_rate >= 0.90 * target
}

fn find_max_goodput(
    label: &str,
    mut trial: impl FnMut(u64) -> Result<GoodputTrial, String>,
) -> Result<GoodputTrial, String> {
    let max_loss = max_loss_pct();
    let mut lo = goodput_rate_lo();
    let mut hi = goodput_rate_hi().max(lo);
    let mut best: Option<GoodputTrial> = None;
    let rate_lo = lo;
    let rate_hi = hi;

    println!(
        "  … {label} max goodput: binary search {lo}..={hi} Hz, loss≤{max_loss:.1}%, trial≈{:.1}s (or fixed msgs)",
        goodput_trial_secs()
    );

    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let t = trial(mid)?;
        let loss = loss_pct(t.sent, t.received);
        let sustained = trial_sustains_rate(&t);
        println!(
            "  …   try {mid} Hz → sent={} recv_send={} recv_final={} loss={loss:.2}% pub={:.0}/s sub_send={:.0}/s sustained={}",
            t.sent,
            t.received_at_send_end,
            t.received,
            t.sent as f64 / t.elapsed.as_secs_f64().max(1e-9),
            t.received_at_send_end as f64 / t.elapsed.as_secs_f64().max(1e-9),
            sustained,
        );
        // Pass only if loss is within budget AND we actually kept the target pace.
        if t.received > 0 && loss <= max_loss && sustained {
            best = Some(t);
            lo = mid.saturating_add(1);
        } else if mid == 0 {
            break;
        } else {
            hi = mid - 1;
        }
    }

    best.ok_or_else(|| {
        format!(
            "no rate in {rate_lo}..={rate_hi} Hz met loss≤{max_loss:.1}% at ≥90% of target pace"
        )
    })
}

fn bench_pubsub(ctx: &Context, transport: &str) -> ScenarioResult {
    let scenario = "message pub/sub";
    let transport = transport.to_string();
    let options = options_for(&transport);
    let topic = format!("perf/{transport}/msg");

    let xsub = match options.message_xsub_endpoint() {
        Ok(ep) => ep,
        Err(err) => {
            return ScenarioResult::skipped(&transport, scenario, format!("xsub endpoint: {err}"));
        }
    };

    let latencies = Arc::new(Mutex::new(Vec::<u64>::with_capacity(msg_latency_samples())));
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
            return ScenarioResult::skipped(
                &transport,
                scenario,
                format!("shutdown handle: {err}"),
            );
        }
    };

    let publisher = match Publisher::with_shared_context(ctx, Some(&xsub), hwm) {
        Ok(p) => p,
        Err(err) => {
            return ScenarioResult::skipped(
                &transport,
                scenario,
                format!("publisher failed: {err}"),
            );
        }
    };

    let count_pub = Arc::clone(&count);
    let lat_pub = Arc::clone(&latencies);
    let rec_pub = Arc::clone(&record_latency);
    let topic_pub = topic.clone();
    let transport_pub = transport.clone();
    let latency_samples = msg_latency_samples();
    let settle = goodput_settle();
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
        for _ in 0..latency_samples {
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

        // Phase 2: binary-search max goodput at loss ≤ threshold.
        rec_pub.store(false, Ordering::Relaxed);
        let goodput = match find_max_goodput(&transport_pub, |rate_hz| {
            count_pub.store(0, Ordering::Relaxed);
            let trial_secs = Duration::from_secs_f64(goodput_trial_secs());
            let (sent, send_elapsed) = match trial_msg_count(rate_hz) {
                Some(n) => publish_paced(&publisher, &topic_pub, n, rate_hz as f64),
                None => publish_paced_for(&publisher, &topic_pub, rate_hz as f64, trial_secs),
            };
            let received_at_send_end = count_pub.load(Ordering::Relaxed);
            thread::sleep(settle);
            let received = count_pub.load(Ordering::Relaxed);
            Ok(GoodputTrial {
                target_hz: rate_hz,
                sent,
                received_at_send_end,
                received,
                elapsed: send_elapsed,
            })
        }) {
            Ok(g) => g,
            Err(note) => {
                shutdown.shutdown();
                return Ok(ScenarioResult::skipped(&transport_pub, scenario, note));
            }
        };
        shutdown.shutdown();

        println!(
            "  … {transport_pub} max goodput ≈ {} Hz target (loss≤{:.1}%)",
            goodput.target_hz,
            max_loss_pct()
        );

        Ok(ScenarioResult::ok_message(
            &transport_pub,
            scenario,
            goodput.sent,
            goodput.received_at_send_end,
            goodput.received,
            goodput.elapsed,
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
            return ScenarioResult::skipped(
                &transport,
                scenario,
                format!("shutdown handle: {err}"),
            );
        }
    };

    let transport_cli = transport.clone();
    let ctx_cli = ctx.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let mut client_node = node_for(
            &ctx_cli,
            format!("perf-svc-cli-{transport_cli}"),
            &transport_cli,
        );
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
            return ScenarioResult::skipped(
                &transport,
                scenario,
                format!("shutdown handle: {err}"),
            );
        }
    };

    let transport_cli = transport.clone();
    let ctx_cli = ctx.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let mut client_node = node_for(
            &ctx_cli,
            format!("perf-act-cli-{transport_cli}"),
            &transport_cli,
        );
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
        if let Err(err) =
            client.send_goal_and_wait(&payload, None, Some(Duration::from_millis(800)))
        {
            shutdown.shutdown();
            return ScenarioResult::skipped(
                &transport_cli,
                scenario,
                format!("send_goal failed: {err}"),
            );
        }
        for _ in 1..WARMUP.min(10) {
            let _ = client.send_goal_and_wait(&payload, None, Some(Duration::from_secs(2)));
        }

        let mut samples = Vec::with_capacity(n);
        let t0 = Instant::now();
        let mut received = 0usize;
        for _ in 0..n {
            let start = Instant::now();
            match client.send_goal_and_wait(&payload, None, Some(Duration::from_secs(5))) {
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

fn bench_ws_subscribe(broker: &RobotBusBroker, url: &str) -> ScenarioResult {
    let transport = "ws";
    let scenario = "message Subscribe";
    let topic = "perf/ws/msg";

    let latencies = Arc::new(Mutex::new(Vec::<u64>::with_capacity(msg_latency_samples())));
    let count = Arc::new(AtomicUsize::new(0));
    let record_latency = Arc::new(AtomicBool::new(true));

    let mut node = Node::ws_at("perf-ws-sub", url);
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
            return ScenarioResult::skipped(transport, scenario, format!("publisher: {err}"));
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
    for _ in 0..msg_latency_samples() {
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

    // Phase 2: binary-search max goodput (pace publish + spin_once on this thread).
    record_latency.store(false, Ordering::Relaxed);
    let settle = goodput_settle();

    let goodput = match find_max_goodput(transport, |rate_hz| {
        count.store(0, Ordering::Relaxed);
        let trial_secs = Duration::from_secs_f64(goodput_trial_secs());
        let interval = Duration::from_secs_f64(1.0 / (rate_hz as f64).max(1.0));
        let t0 = Instant::now();
        let mut next = t0;
        let mut sent = 0usize;
        let fixed_n = trial_msg_count(rate_hz);
        loop {
            let done = match fixed_n {
                Some(n) => sent >= n,
                None => Instant::now() >= t0 + trial_secs,
            };
            if done {
                break;
            }
            if publisher.publish(topic, &make_payload(now_ns())).is_err() {
                break;
            }
            sent += 1;
            next += interval;
            // Busy-wait the remainder so spin_once can still drain.
            while Instant::now() < next {
                let remain = next - Instant::now();
                if remain > Duration::from_millis(2) {
                    let _ = node.spin_once(Some(remain - Duration::from_millis(1)));
                } else {
                    let _ = node.spin_once(Some(Duration::from_micros(50)));
                }
            }
        }
        let send_elapsed = t0.elapsed();
        let received_at_send_end = count.load(Ordering::Relaxed);
        let settle_deadline = Instant::now() + settle;
        while Instant::now() < settle_deadline {
            let _ = node.spin_once(Some(Duration::from_millis(1)));
        }
        let received = count.load(Ordering::Relaxed);
        Ok(GoodputTrial {
            target_hz: rate_hz,
            sent,
            received_at_send_end,
            received,
            elapsed: send_elapsed,
        })
    }) {
        Ok(g) => g,
        Err(note) => return ScenarioResult::skipped(transport, scenario, note),
    };

    println!(
        "  … ws max goodput ≈ {} Hz target (loss≤{:.1}%)",
        goodput.target_hz,
        max_loss_pct()
    );

    ScenarioResult::ok_message(
        transport,
        scenario,
        goodput.sent,
        goodput.received_at_send_end,
        goodput.received,
        goodput.elapsed,
        latency,
    )
}

fn bench_ws_service(broker: &RobotBusBroker, url: &str, n: usize) -> ScenarioResult {
    let transport = "ws";
    let scenario = "service Call";
    let name = "perf.ws.echo";

    let handler: Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync> = Arc::new(|body| body.to_vec());
    let worker = match WorkerThread::spawn_service(name, handler, &broker.service.backend_bind) {
        Ok(w) => w,
        Err(err) => return ScenarioResult::skipped(transport, scenario, format!("worker: {err}")),
    };
    thread::sleep(Duration::from_millis(150));

    let mut node = Node::ws_at("perf-ws-cli", url);
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

fn bench_ws_action(broker: &RobotBusBroker, url: &str, n: usize) -> ScenarioResult {
    let transport = "ws";
    let scenario = "action SendGoal";
    let name = "perf.ws.act";

    let handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> = Arc::new(|body| {
        vec![
            ("FEEDBACK".into(), b"f".to_vec()),
            ("RESULT".into(), body.to_vec()),
        ]
    });
    let worker = match WorkerThread::spawn_action(name, handler, &broker.action.backend_bind) {
        Ok(w) => w,
        Err(err) => return ScenarioResult::skipped(transport, scenario, format!("worker: {err}")),
    };
    thread::sleep(Duration::from_millis(150));

    let mut node = Node::ws_at("perf-ws-act", url);
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
        let _ = client.send_goal_and_wait(&payload, None, Some(Duration::from_secs(2)));
    }

    let mut samples = Vec::with_capacity(n);
    let t0 = Instant::now();
    let mut received = 0usize;
    for _ in 0..n {
        let start = Instant::now();
        match client.send_goal_and_wait(&payload, None, Some(Duration::from_secs(5))) {
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
