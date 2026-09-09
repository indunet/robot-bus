//! Pacing, goodput search, and bus spin helper.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use robot_bus::{Node, ShutdownHandle};

use crate::config::{
    WARMUP, goodput_rate_hi, goodput_rate_lo, goodput_settle, goodput_trial_secs, max_loss_pct,
    msg_latency_samples,
};
use crate::support::{LatencyStats, ScenarioResult, now_ns};

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
    received_at_send_end: usize,
    received: usize,
    elapsed: Duration,
}

fn trial_sustains_rate(t: &GoodputTrial) -> bool {
    let secs = t.elapsed.as_secs_f64().max(1e-9);
    let pub_rate = t.sent as f64 / secs;
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
    println!("  … {label} max goodput: binary search {lo}..={hi} Hz, loss≤{max_loss:.1}%");
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let t = trial(mid)?;
        let loss = loss_pct(t.sent, t.received);
        let sustained = trial_sustains_rate(&t);
        println!(
            "  …   try {mid} Hz → sent={} recv_send={} recv_final={} loss={loss:.2}% sustained={sustained}",
            t.sent, t.received_at_send_end, t.received
        );
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

pub(crate) fn spin_bus(mut node: Node) -> (thread::JoinHandle<()>, ShutdownHandle) {
    let shutdown = node.shutdown_handle().expect("shutdown handle");
    let handle = thread::spawn(move || {
        let _ = node.spin();
    });
    (handle, shutdown)
}

pub(crate) fn run_pub_trial(
    scenario: &str,
    count: Arc<AtomicUsize>,
    latencies: Arc<Mutex<Vec<u64>>>,
    record: Arc<AtomicBool>,
    shutdown: ShutdownHandle,
    publish: impl Fn(u64) -> Result<(), String>,
) -> ScenarioResult {
    thread::sleep(Duration::from_millis(250));
    for _ in 0..WARMUP {
        let _ = publish(now_ns());
    }
    thread::sleep(Duration::from_millis(100));
    count.store(0, Ordering::Relaxed);
    latencies.lock().unwrap().clear();

    record.store(true, Ordering::Relaxed);
    let samples = msg_latency_samples();
    for _ in 0..samples {
        let before = count.load(Ordering::Relaxed);
        if publish(now_ns()).is_err() {
            shutdown.shutdown();
            return ScenarioResult::skipped("inproc", scenario, "publish failed (latency)");
        }
        if !wait_until(&count, before + 1, Duration::from_secs(5)) {
            shutdown.shutdown();
            return ScenarioResult::skipped("inproc", scenario, "latency sample timed out");
        }
    }
    let latency = LatencyStats::from_ns(latencies.lock().unwrap().clone());

    record.store(false, Ordering::Relaxed);
    let settle = goodput_settle();
    let trial_secs = Duration::from_secs_f64(goodput_trial_secs());
    let goodput = match find_max_goodput(scenario, |rate_hz| {
        count.store(0, Ordering::Relaxed);
        let interval = Duration::from_secs_f64(1.0 / (rate_hz as f64).max(1.0));
        let t0 = Instant::now();
        let deadline = t0 + trial_secs;
        let mut next = t0;
        let mut sent = 0usize;
        while Instant::now() < deadline {
            if publish(now_ns()).is_err() {
                break;
            }
            sent += 1;
            next += interval;
            wait_deadline(next);
        }
        let send_elapsed = t0.elapsed();
        let received_at_send_end = count.load(Ordering::Relaxed);
        thread::sleep(settle);
        Ok(GoodputTrial {
            target_hz: rate_hz,
            sent,
            received_at_send_end,
            received: count.load(Ordering::Relaxed),
            elapsed: send_elapsed,
        })
    }) {
        Ok(g) => g,
        Err(err) => {
            shutdown.shutdown();
            return ScenarioResult::skipped("inproc", scenario, err);
        }
    };
    shutdown.shutdown();
    ScenarioResult::ok_message(
        "inproc",
        scenario,
        goodput.sent,
        goodput.received_at_send_end,
        goodput.received,
        goodput.elapsed,
        latency,
    )
}
