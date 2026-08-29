//! Shared pacing / goodput helpers for robot_bus_perf.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use robot_bus::Publisher;
use crate::support::{env_f64, env_usize, now_ns};

pub const PAYLOAD_LEN: usize = 64;
pub const WARMUP: usize = 50;
pub const MSG_HWM: i32 = 2_048;

pub fn svc_iters() -> usize {
    env_usize("ROBOT_BUS_PERF_SVC_ITERS", 10_000)
}

pub fn act_iters() -> usize {
    env_usize("ROBOT_BUS_PERF_ACT_ITERS", 5_000)
}

pub fn msg_latency_samples() -> usize {
    env_usize("ROBOT_BUS_PERF_MSG_LATENCY_SAMPLES", 5_000)
}

pub fn max_loss_pct() -> f64 {
    env_f64("ROBOT_BUS_PERF_MAX_LOSS_PCT", 1.0)
}

fn goodput_trial_msgs() -> usize {
    // Fallback when duration-based sizing is not used.
    env_usize("ROBOT_BUS_PERF_GOODPUT_TRIAL_MSGS", 0)
}

pub fn goodput_trial_secs() -> f64 {
    env_f64("ROBOT_BUS_PERF_GOODPUT_TRIAL_SECS", 1.0)
}

fn goodput_rate_lo() -> u64 {
    env_usize("ROBOT_BUS_PERF_GOODPUT_RATE_LO", 1_000) as u64
}

fn goodput_rate_hi() -> u64 {
    env_usize("ROBOT_BUS_PERF_GOODPUT_RATE_HI", 2_000_000) as u64
}

pub fn goodput_settle() -> Duration {
    Duration::from_millis(env_usize("ROBOT_BUS_PERF_GOODPUT_SETTLE_MS", 100) as u64)
}

/// Fixed message count when `ROBOT_BUS_PERF_GOODPUT_TRIAL_MSGS` is set (smoke).
/// Default path uses [`publish_paced_for`] (wall-clock duration), not a capped count —
/// a 50k cap made MHz targets finish in ~40ms and inflated reported goodput.
pub fn trial_msg_count(rate_hz: u64) -> Option<usize> {
    let fixed = goodput_trial_msgs();
    if fixed > 0 {
        Some(fixed)
    } else {
        let _ = rate_hz;
        None
    }
}
pub fn make_payload(ts_ns: u64) -> Vec<u8> {
    let mut buf = vec![0u8; PAYLOAD_LEN];
    buf[..8].copy_from_slice(&ts_ns.to_le_bytes());
    buf
}

pub fn read_ts(payload: &[u8]) -> Option<u64> {
    if payload.len() < 8 {
        return None;
    }
    let mut b = [0u8; 8];
    b.copy_from_slice(&payload[..8]);
    Some(u64::from_le_bytes(b))
}

pub fn wait_until(count: &AtomicUsize, target: usize, timeout: Duration) -> bool {
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
pub fn publish_paced_for(
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
pub fn publish_paced(publisher: &Publisher, topic: &str, n: usize, rate_hz: f64) -> (usize, Duration) {
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

pub fn wait_deadline(deadline: Instant) {
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

pub struct GoodputTrial {
    pub target_hz: u64,
    pub sent: usize,
    /// Receives observed by end of the send window (before settle).
    pub received_at_send_end: usize,
    /// Receives after settle (loss / delivery).
    pub received: usize,
    pub elapsed: Duration,
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

pub fn find_max_goodput(
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
