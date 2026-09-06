//! WebSocket RPC scenarios.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::support::{LatencyStats, ScenarioResult, now_ns};
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{HighWaterMark, Node, Publisher, RobotBusBroker, Subscriber};

use crate::pacing::{
    GoodputTrial, MSG_HWM, WARMUP, find_max_goodput, goodput_settle, goodput_trial_secs,
    make_payload, max_loss_pct, msg_latency_samples, read_ts, trial_msg_count, wait_deadline,
};

pub fn bench_ws_publish(broker: &RobotBusBroker, url: &str) -> ScenarioResult {
    let transport = "ws";
    let scenario = "message Publish";
    let topic = "perf/ws/pub";

    let mut node = Node::ws_at("perf-ws-pub", url);
    let publisher = match node.create_publisher_raw(topic) {
        Ok(p) => p,
        Err(err) => {
            return ScenarioResult::skipped(transport, scenario, format!("publisher: {err}"));
        }
    };

    let hwm = HighWaterMark {
        snd: MSG_HWM,
        rcv: MSG_HWM,
    };
    let sub = match Subscriber::with_hwm(Some(&broker.message.xpub_bind), hwm) {
        Ok(s) => s,
        Err(err) => {
            return ScenarioResult::skipped(transport, scenario, format!("subscriber: {err}"));
        }
    };
    if let Err(err) = sub.subscribe(topic) {
        return ScenarioResult::skipped(transport, scenario, format!("subscribe: {err}"));
    }

    let count = Arc::new(AtomicUsize::new(0));
    let latencies = Arc::new(Mutex::new(Vec::<u64>::with_capacity(msg_latency_samples())));
    let running = Arc::new(AtomicBool::new(true));
    let record_latency = Arc::new(AtomicBool::new(true));
    let recv = thread::spawn({
        let count = Arc::clone(&count);
        let latencies = Arc::clone(&latencies);
        let running = Arc::clone(&running);
        let record_latency = Arc::clone(&record_latency);
        move || {
            while running.load(Ordering::Relaxed) {
                match sub.receive(Some(Duration::from_millis(50))) {
                    Ok((_, payload)) => {
                        if record_latency.load(Ordering::Relaxed) {
                            if let Some(sent) = read_ts(&payload) {
                                let now = now_ns();
                                if now >= sent {
                                    latencies.lock().unwrap().push(now - sent);
                                }
                            }
                        }
                        count.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {}
                }
            }
        }
    });
    thread::sleep(Duration::from_millis(400));

    for _ in 0..WARMUP {
        let _ = publisher.publish(&make_payload(now_ns()));
    }
    thread::sleep(Duration::from_millis(50));
    count.store(0, Ordering::Relaxed);
    latencies.lock().unwrap().clear();

    record_latency.store(true, Ordering::Relaxed);
    for _ in 0..msg_latency_samples() {
        let before = count.load(Ordering::Relaxed);
        if publisher.publish(&make_payload(now_ns())).is_err() {
            running.store(false, Ordering::Relaxed);
            let _ = recv.join();
            return ScenarioResult::skipped(transport, scenario, "publish failed");
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while count.load(Ordering::Relaxed) <= before && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }
        if count.load(Ordering::Relaxed) <= before {
            running.store(false, Ordering::Relaxed);
            let _ = recv.join();
            return ScenarioResult::skipped(transport, scenario, "latency sample timed out");
        }
    }
    let latency = LatencyStats::from_ns(latencies.lock().unwrap().clone());

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
            if publisher.publish(&make_payload(now_ns())).is_err() {
                break;
            }
            sent += 1;
            next += interval;
            wait_deadline(next);
        }
        let send_elapsed = t0.elapsed();
        let received_at_send_end = count.load(Ordering::Relaxed);
        thread::sleep(settle);
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
        Err(note) => {
            running.store(false, Ordering::Relaxed);
            let _ = recv.join();
            return ScenarioResult::skipped(transport, scenario, note);
        }
    };

    running.store(false, Ordering::Relaxed);
    let _ = recv.join();

    println!(
        "  … ws publish max goodput ≈ {} Hz target (loss≤{:.1}%)",
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

pub fn bench_ws_subscribe(broker: &RobotBusBroker, url: &str) -> ScenarioResult {
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
            move |payload| {
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

pub fn bench_ws_service(broker: &RobotBusBroker, url: &str, n: usize) -> ScenarioResult {
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

pub fn bench_ws_action(broker: &RobotBusBroker, url: &str, n: usize) -> ScenarioResult {
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
