//! Shared pacing / goodput helpers for robot_bus_perf.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use crate::support::{LatencyStats, ScenarioResult, env_f64, env_usize};

pub const WARMUP: usize = 50;
const CDR_LATENCY_STRIDE: usize = 16;
const DEFAULT_PAYLOAD_LEN: usize = 64;
const DEFAULT_MSG_HWM: i32 = 2_048;

/// Application payload bytes. `ROBOT_BUS_PERF_PAYLOAD_BYTES` overrides the 64-byte default.
pub fn payload_len() -> usize {
    static LEN: OnceLock<usize> = OnceLock::new();
    *LEN.get_or_init(|| {
        let len = env_usize("ROBOT_BUS_PERF_PAYLOAD_BYTES", DEFAULT_PAYLOAD_LEN);
        assert!(
            (24..=32 * 1024 * 1024).contains(&len),
            "ROBOT_BUS_PERF_PAYLOAD_BYTES must be 24..=32MiB"
        );
        len
    })
}

/// Message queue depth. `ROBOT_BUS_PERF_MSG_HWM` overrides the 2048 default.
pub fn msg_hwm() -> i32 {
    static HWM: OnceLock<i32> = OnceLock::new();
    *HWM.get_or_init(|| {
        let hwm = env_usize("ROBOT_BUS_PERF_MSG_HWM", DEFAULT_MSG_HWM as usize);
        assert!(hwm > 0, "ROBOT_BUS_PERF_MSG_HWM must be positive");
        hwm as i32
    })
}

/// CDR prefix for `std_msgs/UInt8MultiArray`: encapsulation, no dimensions, length.
pub fn cdr_prefix() -> [u8; 16] {
    let mut prefix = [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    prefix[12..16].copy_from_slice(&(payload_len() as u32).to_le_bytes());
    prefix
}

pub fn svc_iters() -> usize {
    env_usize("ROBOT_BUS_PERF_SVC_ITERS", 10_000)
}

pub fn act_iters() -> usize {
    env_usize("ROBOT_BUS_PERF_ACT_ITERS", 5_000)
}

pub fn rpc_warmup(default: usize) -> usize {
    env_usize("ROBOT_BUS_PERF_RPC_WARMUP", default).max(1)
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

pub fn make_payload(ts_ns: u64) -> Vec<u8> {
    let mut buf = vec![0u8; payload_len()];
    buf[..8].copy_from_slice(&ts_ns.to_le_bytes());
    buf
}

fn clock_ns() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_nanos() as u64
}

#[derive(Default, Clone, Copy)]
struct Counts {
    received: usize,
    duplicates: usize,
    stale: usize,
    invalid: usize,
}

#[derive(Default)]
pub struct TrialReceiver {
    issued: AtomicUsize,
    window: Mutex<ReceiveWindow>,
    cdr: bool,
}

#[derive(Default)]
struct ReceiveWindow {
    epoch: u64,
    seen: Vec<bool>,
    counts: Counts,
    latency_stride: usize,
    latency_ns: Vec<u64>,
}

impl TrialReceiver {
    pub fn with_cdr(cdr: bool) -> Self {
        Self {
            cdr,
            ..Self::default()
        }
    }

    fn start(&self, latency_stride: usize) -> u64 {
        let mut window = self.window.lock().unwrap();
        self.issued.store(0, Ordering::Relaxed);
        window.epoch += 1;
        window.seen.clear();
        window.counts = Counts::default();
        window.latency_stride = latency_stride;
        window.latency_ns.clear();
        window.epoch
    }

    fn new_payload(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        if self.cdr {
            payload.extend_from_slice(&cdr_prefix());
        }
        payload.resize(payload.len() + payload_len(), 0);
        payload
    }

    fn stamp(&self, payload: &mut [u8], epoch: u64, sequence: usize, timestamp: u64) {
        let offset = if self.cdr { cdr_prefix().len() } else { 0 };
        let payload = &mut payload[offset..];
        payload[..8].copy_from_slice(&timestamp.to_le_bytes());
        payload[8..16].copy_from_slice(&epoch.to_le_bytes());
        payload[16..24].copy_from_slice(&(sequence as u64).to_le_bytes());
        self.issued.store(sequence + 1, Ordering::Release);
    }

    fn payload(&self, epoch: u64, sequence: usize) -> Vec<u8> {
        let timestamp = clock_ns();
        let mut payload = self.new_payload();
        self.stamp(&mut payload, epoch, sequence, timestamp);
        payload
    }

    pub fn receive(&self, payload: &[u8]) {
        let mut window = self.window.lock().unwrap();
        let payload = if self.cdr {
            let prefix = cdr_prefix();
            let Some(body) = payload.strip_prefix(&prefix) else {
                window.counts.invalid += 1;
                return;
            };
            body
        } else {
            payload
        };
        if payload.len() != payload_len() {
            window.counts.invalid += 1;
            return;
        }
        let epoch = u64::from_le_bytes(payload[8..16].try_into().unwrap());
        if epoch != window.epoch {
            window.counts.stale += 1;
            return;
        }
        let sequence = u64::from_le_bytes(payload[16..24].try_into().unwrap());
        if sequence >= self.issued.load(Ordering::Acquire) as u64 {
            window.counts.invalid += 1;
            return;
        }
        let sequence = sequence as usize;
        if sequence >= window.seen.len() {
            window.seen.resize(sequence + 1, false);
        }
        if std::mem::replace(&mut window.seen[sequence], true) {
            window.counts.duplicates += 1;
            return;
        }
        window.counts.received += 1;
        if window.latency_stride > 0 && sequence.is_multiple_of(window.latency_stride) {
            let sent = u64::from_le_bytes(payload[..8].try_into().unwrap());
            if let Some(latency) = clock_ns().checked_sub(sent) {
                window.latency_ns.push(latency);
            }
        }
    }

    fn snapshot(&self) -> (u64, Counts) {
        let window = self.window.lock().unwrap();
        (clock_ns(), window.counts)
    }

    fn finish(&self) -> (u64, Counts, Vec<u64>) {
        let mut window = self.window.lock().unwrap();
        let end = clock_ns();
        window.latency_stride = 0;
        (end, window.counts, std::mem::take(&mut window.latency_ns))
    }
}

/// Publisher and subscriber run independently. Keep latency, warmup and each
/// throughput trial in separate epochs even when the transport queues overlap.
pub fn measure_pubsub<E: std::fmt::Display>(
    transport: &str,
    scenario: &str,
    receiver: &TrialReceiver,
    mut publish: impl FnMut(&[u8]) -> Result<(), E>,
) -> ScenarioResult {
    let mut payload = receiver.new_payload();
    for sequence in 0..WARMUP {
        if receiver.cdr {
            receiver.stamp(&mut payload, 0, sequence, clock_ns());
            let _ = publish(&payload);
        } else {
            let _ = publish(&make_payload(clock_ns()));
        }
    }
    thread::sleep(Duration::from_millis(100));

    let mut publish_one = |epoch, sequence| {
        if receiver.cdr {
            receiver.stamp(&mut payload, epoch, sequence, clock_ns());
            publish(&payload)
        } else {
            publish(&receiver.payload(epoch, sequence))
        }
    };

    let latency_epoch = receiver.start(1);
    let latency_start = clock_ns();
    for sequence in 0..msg_latency_samples() {
        if let Err(err) = publish_one(latency_epoch, sequence) {
            return ScenarioResult::skipped(transport, scenario, format!("latency publish: {err}"));
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while receiver.snapshot().1.received <= sequence {
            if Instant::now() >= deadline {
                let window = receiver.window.lock().unwrap();
                println!(
                    "RAW latency {}",
                    serde_json::json!({
                        "transport": transport, "scenario": scenario,
                        "start_ns": latency_start, "end_ns": clock_ns(),
                        "sent": sequence + 1, "received": window.counts.received,
                        "samples_ns": window.latency_ns, "error": "latency sample timed out",
                    })
                );
                return ScenarioResult::skipped(
                    transport,
                    scenario,
                    format!("latency sample {sequence} timed out"),
                );
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
    let latency = {
        let (end, counts, samples) = receiver.finish();
        println!(
            "RAW latency {}",
            serde_json::json!({
                "transport": transport, "scenario": scenario,
                "start_ns": latency_start, "end_ns": end,
                "sent": msg_latency_samples(), "received": counts.received,
                "duplicates": counts.duplicates, "stale": counts.stale,
                "invalid": counts.invalid, "samples_ns": samples,
            })
        );
        LatencyStats::from_ns(samples)
    };

    let result = find_max_goodput(transport, |rate_hz| {
        let stride = if receiver.cdr { CDR_LATENCY_STRIDE } else { 0 };
        let epoch = receiver.start(stride);
        let trial_duration = Duration::from_secs_f64(goodput_trial_secs());
        let interval = Duration::from_secs_f64(1.0 / rate_hz.max(1) as f64);
        let fixed = goodput_trial_msgs();
        let start_ns = clock_ns();
        let start = Instant::now();
        let mut next = start;
        let mut sent = 0;
        while if fixed > 0 {
            sent < fixed
        } else {
            start.elapsed() < trial_duration
        } {
            publish_one(epoch, sent).map_err(|err| format!("publish: {err}"))?;
            sent += 1;
            next += interval;
            wait_deadline(next);
        }
        let (send_end_ns, at_send_end) = receiver.snapshot();
        thread::sleep(goodput_settle());
        let (settled_ns, final_counts, samples) = receiver.finish();
        let trial = GoodputTrial {
            target_hz: rate_hz,
            sent,
            received_at_send_end: at_send_end.received,
            received: final_counts.received,
            elapsed: Duration::from_nanos(send_end_ns - start_ns),
        };
        let valid = trial.received_at_send_end <= trial.received
            && trial.received <= trial.sent
            && final_counts.invalid == 0;
        println!(
            "RAW trial {}",
            serde_json::json!({
                "transport": transport, "scenario": scenario, "epoch": epoch,
                "target_hz": rate_hz, "sent": sent,
                "recv_send": trial.received_at_send_end, "recv_final": trial.received,
                "duplicates": final_counts.duplicates, "stale": final_counts.stale, "invalid": final_counts.invalid,
                "start_ns": start_ns, "send_end_ns": send_end_ns, "settled_ns": settled_ns,
                "valid": valid, "latency_stride": stride, "samples_ns": samples,
            })
        );
        if !valid {
            return Err(format!("invalid receive counts in trial {epoch}"));
        }
        Ok(trial)
    });

    match result {
        Ok(goodput) => ScenarioResult::ok_message(
            transport,
            scenario,
            goodput.sent,
            goodput.received_at_send_end,
            goodput.received,
            goodput.elapsed,
            latency,
        ),
        Err(note) => ScenarioResult::skipped(transport, scenario, note),
    }
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

fn loss_pct(sent: usize, received: usize) -> f64 {
    if sent == 0 {
        100.0
    } else {
        100.0 * (sent - received) as f64 / sent as f64
    }
}

struct GoodputTrial {
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

fn find_max_goodput(
    label: &str,
    mut trial: impl FnMut(u64) -> Result<GoodputTrial, String>,
) -> Result<GoodputTrial, String> {
    let max_loss = max_loss_pct();
    let step = env_usize("ROBOT_BUS_PERF_GOODPUT_RATE_STEP", 1).max(1) as u64;
    let rate_lo = goodput_rate_lo();
    let mut lo = 0;
    let mut hi = (goodput_rate_hi().max(rate_lo) - rate_lo) / step;
    let rate_hi = rate_lo + hi * step;
    let mut best: Option<GoodputTrial> = None;

    println!(
        "  … {label} max goodput: binary search {rate_lo}..={rate_hi} Hz (step {step}), loss≤{max_loss:.1}%, trial≈{:.1}s (or fixed msgs)",
        goodput_trial_secs()
    );

    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let t = trial(rate_lo + mid * step)?;
        let loss = loss_pct(t.sent, t.received);
        let sustained = trial_sustains_rate(&t);
        println!(
            "  …   try {} Hz → sent={} recv_send={} recv_final={} loss={loss:.2}% pub={:.0}/s sub_send={:.0}/s sustained={}",
            t.target_hz,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delayed_and_duplicate_messages_cannot_count_as_current_deliveries() {
        let receiver = TrialReceiver::default();
        let warmup = make_payload(clock_ns());
        let old_epoch = receiver.start(0);
        let queued = receiver.payload(old_epoch, 0);
        let epoch = receiver.start(1);
        let first = receiver.payload(epoch, 0);
        receiver.receive(&warmup);
        receiver.receive(&queued);
        receiver.receive(&first);
        let at_send_end = receiver.snapshot().1;
        assert_eq!(at_send_end.received, 1);
        assert_eq!(at_send_end.stale, 2);

        receiver.receive(&first);
        assert_eq!(receiver.snapshot().1.duplicates, 1);
        let second = receiver.payload(epoch, 1);
        receiver.receive(&second);
        let settled = receiver.snapshot().1;
        assert_eq!(settled.received, 2);
        assert_eq!(receiver.window.lock().unwrap().latency_ns.len(), 2);
    }

    #[test]
    fn websocket_backlog_does_not_inflate_the_next_trial() {
        use std::sync::Arc;

        use robot_bus::{HighWaterMark, Node, Publisher, RobotBusBroker};

        let _guard = crate::support::lock_broker();
        let mut config = crate::support::perf_broker_config();
        config.ws.listen = "127.0.0.1:0".parse().unwrap();
        let broker = RobotBusBroker::start(config).unwrap();
        let mut node = Node::ws_at("perf-ws-late-packets", &broker.api_url());
        let receiver = Arc::new(TrialReceiver::default());
        let topic = "perf/ws/late-packets";
        node.create_subscription_raw(
            topic,
            Arc::new({
                let receiver = Arc::clone(&receiver);
                move |payload| receiver.receive(payload)
            }),
            None,
        )
        .unwrap();
        let publisher =
            Publisher::with_hwm(Some(&broker.discover.message_xsub), HighWaterMark::STREAM)
                .unwrap();

        let previous = receiver.start(0);
        let probe = receiver.payload(previous, 0);
        let deadline = Instant::now() + Duration::from_secs(3);
        while receiver.snapshot().1.received == 0 {
            assert!(
                Instant::now() < deadline,
                "WS subscription did not become ready"
            );
            publisher.publish(topic, &probe).unwrap();
            node.spin_once(Some(Duration::from_millis(20))).unwrap();
        }

        publisher
            .publish(topic, &receiver.payload(previous, 1))
            .unwrap();
        // Leave the WS callback queue undrained when moving to the next trial.
        let epoch = receiver.start(0);
        let sample = receiver.payload(epoch, 0);
        publisher.publish(topic, &sample).unwrap();
        publisher.publish(topic, &sample).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let counts = receiver.snapshot().1;
            if counts.stale > 0 && counts.duplicates > 0 {
                assert_eq!(counts.received, 1);
                assert_eq!(counts.invalid, 0);
                println!(
                    "WS trial sent=1 raw_arrivals={} unique={} stale={} duplicates={}",
                    counts.received + counts.stale + counts.duplicates,
                    counts.received,
                    counts.stale,
                    counts.duplicates,
                );
                break;
            }
            assert!(
                Instant::now() < deadline,
                "expected late and duplicate WS frames"
            );
            node.spin_once(Some(Duration::from_millis(20))).unwrap();
        }
        drop(node);
        broker.stop().unwrap();
    }
}
