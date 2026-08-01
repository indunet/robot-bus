//! Shared helpers for `benches/robot_bus_perf`.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
use robot_bus::{Context, Node, NodeOptions, RobotBusConfig};

/// Only one bind_all broker at a time (fixed ipc / inproc names).
static BROKER_LOCK: Mutex<()> = Mutex::new(());

pub fn lock_broker() -> MutexGuard<'static, ()> {
    BROKER_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn perf_broker_config() -> RobotBusConfig {
    let mut config = RobotBusConfig::default();
    // Deep enough that paced goodput trials are not HWM-drop dominated.
    // Not a production default (STREAM default remains 2).
    config.message.snd_hwm = 2_048;
    config.message.rcv_hwm = 2_048;
    config.service.snd_hwm = 64;
    config.service.rcv_hwm = 64;
    config.action.snd_hwm = 64;
    config.action.rcv_hwm = 64;
    #[cfg(feature = "console")]
    {
        config.console = ConsoleBrokerConfig {
            enabled: false,
            ..ConsoleBrokerConfig::default()
        };
    }
    config
}

pub fn options_for(transport: &str) -> NodeOptions {
    match transport {
        "tcp" => NodeOptions::tcp(),
        "ipc" => NodeOptions::ipc(),
        "inproc" => NodeOptions::inproc(),
        other => panic!("unknown transport: {other}"),
    }
}

/// Node sharing `context` (required for inproc with the embedded broker).
pub fn node_for(context: &Context, name: impl Into<String>, transport: &str) -> Node {
    Node::with_context(context.clone(), name, options_for(transport))
}

pub fn now_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos() as u64
}

#[derive(Clone, Debug)]
pub struct LatencyStats {
    pub count: usize,
    pub p50_us: f64,
    pub p95_us: f64,
    pub p99_us: f64,
    pub mean_us: f64,
}

impl LatencyStats {
    pub fn from_ns(mut samples: Vec<u64>) -> Self {
        if samples.is_empty() {
            return Self {
                count: 0,
                p50_us: 0.0,
                p95_us: 0.0,
                p99_us: 0.0,
                mean_us: 0.0,
            };
        }
        samples.sort_unstable();
        let n = samples.len();
        let sum: u128 = samples.iter().map(|&v| v as u128).sum();
        Self {
            count: n,
            p50_us: percentile_us(&samples, 0.50),
            p95_us: percentile_us(&samples, 0.95),
            p99_us: percentile_us(&samples, 0.99),
            mean_us: (sum as f64 / n as f64) / 1_000.0,
        }
    }
}

fn percentile_us(sorted_ns: &[u64], p: f64) -> f64 {
    let n = sorted_ns.len();
    if n == 0 {
        return 0.0;
    }
    let idx = ((n as f64 - 1.0) * p).round() as usize;
    sorted_ns[idx.min(n - 1)] as f64 / 1_000.0
}

#[derive(Clone, Debug)]
pub struct ScenarioResult {
    pub transport: String,
    pub scenario: String,
    /// Messages / calls issued.
    pub sent: usize,
    /// Messages / replies observed.
    pub received: usize,
    pub elapsed: Duration,
    /// Issue rate (publish or call/s).
    pub publish_per_s: f64,
    /// Delivery / completion rate (subscribe or successful reply/s).
    pub subscribe_per_s: f64,
    /// `100 * received / sent` (message bus may be < 100% under HWM drops).
    pub delivery_pct: f64,
    pub latency: LatencyStats,
    pub note: Option<String>,
    pub kind: ScenarioKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenarioKind {
    Message,
    Rpc,
}

impl ScenarioResult {
    /// Pub/sub max-goodput trial (paced rate with loss ≤ threshold).
    ///
    /// `received_at_send_end` drives subscribe rate (keep-up during send window).
    /// `received_final` (after settle) drives delivery %.
    pub fn ok_message(
        transport: &str,
        scenario: &str,
        sent: usize,
        received_at_send_end: usize,
        received_final: usize,
        elapsed: Duration,
        latency: LatencyStats,
    ) -> Self {
        let secs = elapsed.as_secs_f64().max(1e-9);
        Self {
            transport: transport.into(),
            scenario: scenario.into(),
            sent,
            received: received_final,
            elapsed,
            publish_per_s: sent as f64 / secs,
            subscribe_per_s: received_at_send_end as f64 / secs,
            delivery_pct: if sent == 0 {
                0.0
            } else {
                100.0 * received_final as f64 / sent as f64
            },
            latency,
            note: None,
            kind: ScenarioKind::Message,
        }
    }

    /// Reliable RPC-style (service / action): one reply per call.
    pub fn ok_rpc(
        transport: &str,
        scenario: &str,
        n: usize,
        received: usize,
        elapsed: Duration,
        latency: LatencyStats,
    ) -> Self {
        let secs = elapsed.as_secs_f64().max(1e-9);
        let rate = received as f64 / secs;
        Self {
            transport: transport.into(),
            scenario: scenario.into(),
            sent: n,
            received,
            elapsed,
            publish_per_s: rate,
            subscribe_per_s: rate,
            delivery_pct: if n == 0 {
                0.0
            } else {
                100.0 * received as f64 / n as f64
            },
            latency,
            note: None,
            kind: ScenarioKind::Rpc,
        }
    }

    pub fn skipped(transport: &str, scenario: &str, note: impl Into<String>) -> Self {
        Self {
            transport: transport.into(),
            scenario: scenario.into(),
            sent: 0,
            received: 0,
            elapsed: Duration::ZERO,
            publish_per_s: 0.0,
            subscribe_per_s: 0.0,
            delivery_pct: 0.0,
            latency: LatencyStats::from_ns(Vec::new()),
            note: Some(note.into()),
            kind: ScenarioKind::Rpc,
        }
    }
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn write_report(results: &[ScenarioResult], env_lines: &[String]) -> std::io::Result<PathBuf> {
    let path = repo_root().join("docs/perf-report.md");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut md = String::new();
    md.push_str("# robot-bus 性能测试报告\n\n");
    md.push_str("由 `just perf`（`cargo run --release --bin robot_bus_perf`）生成。\n\n");

    md.push_str("## 环境\n\n");
    for line in env_lines {
        md.push_str("- ");
        md.push_str(line);
        md.push('\n');
    }
    md.push('\n');

    md.push_str("## 方法\n\n");
    md.push_str("- 进程内 `RobotBusBroker`，`bind_all_transports = true`（tcp + ipc + inproc + grpc）。\n");
    md.push_str("- Console HTTP 关闭；message HWM=2048（仅 bench）；service/action HWM=64。\n");
    md.push_str("- Payload：64 字节 raw（前 8 字节为发送端 Unix 纳秒时间戳，用于延迟）。\n");
    md.push_str(&format!(
        "- Message **吞吐（主指标）**：按目标速率限速发送约 {:.1}s（可用 `ROBOT_BUS_PERF_GOODPUT_TRIAL_MSGS` 改为固定条数），**二分搜索**丢包率 ≤ {:.1}% 且发送窗口内 pub/sub 均 ≥90% 目标速率的最大可持续速率（max goodput）。\n",
        env_f64("ROBOT_BUS_PERF_GOODPUT_TRIAL_SECS", 1.0),
        env_f64("ROBOT_BUS_PERF_MAX_LOSS_PCT", 1.0),
    ));
    md.push_str(&format!(
        "- Message **延迟**：另做 {} 次限速抽样（发一条等收到再发），测单程时延。\n",
        env_usize("ROBOT_BUS_PERF_MSG_LATENCY_SAMPLES", 5_000),
    ));
    md.push_str(&format!(
        "- Service / action：各 {} / {} 次（`ROBOT_BUS_PERF_SVC_ITERS` / `ROBOT_BUS_PERF_ACT_ITERS`）；延迟为每次 call / send_goal 本地计时。\n",
        env_usize("ROBOT_BUS_PERF_SVC_ITERS", 10_000),
        env_usize("ROBOT_BUS_PERF_ACT_ITERS", 5_000),
    ));
    md.push_str("- ZMQ：共享 `Context` + `Node::tcp` / `ipc` / `inproc`；gRPC：`Node::grpc_at`。\n");
    md.push_str("- inproc 与嵌入式 broker 必须共用同一 `Context`（ZeroMQ inproc 是 context-local）。\n");
    md.push_str("- 指标为单机本机回环，机器相关，不作为 CI 门槛。\n\n");

    md.push_str("## 横比\n\n");
    md.push_str("message 为 **max goodput**（丢包阈值内的最大可持续订阅速率）；括号为该档实测投递率。service/action 为完成速率。gRPC Node **不支持 publish**，发布格为 —。\n\n");
    md.push_str("| 场景 | tcp | ipc | inproc | grpc |\n");
    md.push_str("|------|-----|-----|--------|------|\n");
    md.push_str(&format!(
        "| message 发布 | {} | {} | {} | — |\n",
        cell_pub(results, "tcp", "message pub/sub"),
        cell_pub(results, "ipc", "message pub/sub"),
        cell_pub(results, "inproc", "message pub/sub"),
    ));
    md.push_str(&format!(
        "| message max goodput | {} | {} | {} | {} |\n",
        cell_sub(results, "tcp", "message pub/sub"),
        cell_sub(results, "ipc", "message pub/sub"),
        cell_sub(results, "inproc", "message pub/sub"),
        cell_sub(results, "grpc", "message Subscribe"),
    ));
    md.push_str(&format!(
        "| service call | {} | {} | {} | {} |\n",
        cell_rpc(results, "tcp", "service call"),
        cell_rpc(results, "ipc", "service call"),
        cell_rpc(results, "inproc", "service call"),
        cell_rpc(results, "grpc", "service Call"),
    ));
    md.push_str(&format!(
        "| action send_goal | {} | {} | {} | {} |\n\n",
        cell_rpc(results, "tcp", "action send_goal"),
        cell_rpc(results, "ipc", "action send_goal"),
        cell_rpc(results, "inproc", "action send_goal"),
        cell_rpc(results, "grpc", "action Run"),
    ));

    for group in ["tcp", "ipc", "inproc", "grpc"] {
        md.push_str(&format!("## {group}\n\n"));
        md.push_str(
            "| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |\n",
        );
        md.push_str(
            "|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|\n",
        );
        for r in results.iter().filter(|r| r.transport == group) {
            if r.note.is_some() {
                md.push_str(&format!(
                    "| {} | — | — | — | — | — | — | — | — | — | — |\n",
                    r.scenario
                ));
            } else {
                md.push_str(&format!(
                    "| {} | {} | {} | {:.3}s | {:.0} | {:.0} | {:.1} | {:.0} | {:.0} | {:.0} | {:.0} |\n",
                    r.scenario,
                    r.sent,
                    r.received,
                    r.elapsed.as_secs_f64(),
                    r.publish_per_s,
                    r.subscribe_per_s,
                    r.delivery_pct,
                    r.latency.p50_us,
                    r.latency.p95_us,
                    r.latency.p99_us,
                    r.latency.mean_us,
                ));
            }
        }
        md.push('\n');
    }

    md.push_str("## 复现\n\n");
    md.push_str("```bash\njust perf\n# 或\ncargo run --release --bin robot_bus_perf\n");
    md.push_str("# 仅 message：ROBOT_BUS_PERF_ONLY=message cargo run --release --bin robot_bus_perf --features grpc\n");
    md.push_str("```\n");

    fs::write(&path, md)?;
    Ok(path)
}

fn find<'a>(results: &'a [ScenarioResult], transport: &str, scenario: &str) -> Option<&'a ScenarioResult> {
    results
        .iter()
        .find(|r| r.transport == transport && r.scenario == scenario)
}

fn cell_pub(results: &[ScenarioResult], transport: &str, scenario: &str) -> String {
    match find(results, transport, scenario) {
        Some(r) if r.note.is_some() => "—".into(),
        Some(r) => format!("{:.0}/s", r.publish_per_s),
        None => "—".into(),
    }
}

fn cell_sub(results: &[ScenarioResult], transport: &str, scenario: &str) -> String {
    match find(results, transport, scenario) {
        Some(r) if r.note.is_some() => "—".into(),
        Some(r) if r.kind == ScenarioKind::Message => {
            format!("{:.0}/s ({:.1}% delivered)", r.subscribe_per_s, r.delivery_pct)
        }
        Some(r) => format!("{:.0}/s ({:.0}%)", r.subscribe_per_s, r.delivery_pct),
        None => "—".into(),
    }
}

pub fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn cell_rpc(results: &[ScenarioResult], transport: &str, scenario: &str) -> String {
    match find(results, transport, scenario) {
        Some(r) if r.note.is_some() => "—".into(),
        Some(r) => format!("{:.0}/s", r.subscribe_per_s),
        None => "—".into(),
    }
}

pub fn env_summary() -> Vec<String> {
    let mut lines = vec![format!("robot-bus: {}", env!("CARGO_PKG_VERSION"))];

    if let Ok(out) = std::process::Command::new("sw_vers").output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut product = None;
            let mut version = None;
            let mut build = None;
            for line in text.lines() {
                if let Some(v) = line.strip_prefix("ProductName:") {
                    product = Some(v.trim().to_string());
                } else if let Some(v) = line.strip_prefix("ProductVersion:") {
                    version = Some(v.trim().to_string());
                } else if let Some(v) = line.strip_prefix("BuildVersion:") {
                    build = Some(v.trim().to_string());
                }
            }
            match (product, version, build) {
                (Some(p), Some(v), Some(b)) => lines.push(format!("系统: {p} {v} ({b})")),
                (Some(p), Some(v), None) => lines.push(format!("系统: {p} {v}")),
                _ => {}
            }
        }
    }

    if let Ok(cpus) = std::thread::available_parallelism() {
        lines.push(format!("逻辑 CPU: {cpus}"));
    }

    push_sysctl(&mut lines, "CPU", "machdep.cpu.brand_string");
    push_sysctl(&mut lines, "CPU 核心(物理)", "hw.physicalcpu");
    push_sysctl(&mut lines, "CPU 核心(逻辑)", "hw.logicalcpu");
    if let Some(bytes) = sysctl_u64("hw.memsize") {
        lines.push(format!(
            "内存: {:.1} GiB",
            bytes as f64 / (1024.0 * 1024.0 * 1024.0)
        ));
    }

    lines
}

fn push_sysctl(lines: &mut Vec<String>, label: &str, key: &str) {
    if let Some(v) = sysctl_string(key) {
        if !v.is_empty() {
            lines.push(format!("{label}: {v}"));
        }
    }
}

fn sysctl_string(key: &str) -> Option<String> {
    let out = std::process::Command::new("sysctl")
        .args(["-n", key])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

fn sysctl_u64(key: &str) -> Option<u64> {
    sysctl_string(key)?.parse().ok()
}
