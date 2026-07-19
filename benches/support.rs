//! Shared helpers for `robot_bus_perf`.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
use robot_bus::{NodeOptions, RobotBusConfig};

/// Only one bind_all broker at a time (fixed ipc / inproc names).
static BROKER_LOCK: Mutex<()> = Mutex::new(());

pub fn lock_broker() -> MutexGuard<'static, ()> {
    BROKER_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn perf_broker_config() -> RobotBusConfig {
    let mut config = RobotBusConfig::default();
    // Deep message queues for firehose pub/sub (service/action stay moderate).
    config.message.snd_hwm = 100_000;
    config.message.rcv_hwm = 100_000;
    config.service.snd_hwm = 1000;
    config.service.rcv_hwm = 1000;
    config.action.snd_hwm = 1000;
    config.action.rcv_hwm = 1000;
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
    pub iterations: usize,
    pub received: usize,
    pub elapsed: Duration,
    pub throughput_per_s: f64,
    pub latency: LatencyStats,
    pub note: Option<String>,
}

impl ScenarioResult {
    pub fn ok(
        transport: &str,
        scenario: &str,
        iterations: usize,
        received: usize,
        elapsed: Duration,
        latency: LatencyStats,
    ) -> Self {
        let secs = elapsed.as_secs_f64().max(1e-9);
        Self {
            transport: transport.into(),
            scenario: scenario.into(),
            iterations,
            received,
            elapsed,
            throughput_per_s: received as f64 / secs,
            latency,
            note: None,
        }
    }

    pub fn skipped(transport: &str, scenario: &str, note: impl Into<String>) -> Self {
        Self {
            transport: transport.into(),
            scenario: scenario.into(),
            iterations: 0,
            received: 0,
            elapsed: Duration::ZERO,
            throughput_per_s: 0.0,
            latency: LatencyStats::from_ns(Vec::new()),
            note: Some(note.into()),
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
    md.push_str("- Console HTTP 关闭；message/service/action HWM=1000。\n");
    md.push_str("- 各传输迭代次数相同：message 10000（狂发） / service 5000 / action 1000（便于横比）。\n");
    md.push_str("- Payload：64 字节 raw（前 8 字节为发送端 Unix 纳秒时间戳，用于延迟）。\n");
    md.push_str("- Message：发布端尽快连发 N 条（不等待 ACK），订阅端收满 N 条即结束；message HWM=100000。\n");
    md.push_str("- ZMQ：`Node::tcp` / `Node::ipc` / `Node::inproc`；gRPC：`Node::grpc_at`。\n");
    md.push_str("- 指标为单机本机回环，机器相关，不作为 CI 门槛。\n\n");

    md.push_str("## 横比\n\n");
    md.push_str("单元格为 **吞吐/s · p50(µs)**。gRPC Node **不支持 publish**，对应格为 —。\n");
    md.push_str("ZMQ（tcp / ipc）下 message 发布与订阅测的是同一条 pub→sub 端到端路径；gRPC 仅测 Subscribe。\n\n");
    md.push_str("| 场景 | tcp | ipc | inproc | grpc |\n");
    md.push_str("|------|-----|-----|--------|------|\n");
    md.push_str(&format!(
        "| message 发布 | {} | {} | {} | — |\n",
        cell(results, "tcp", "message pub/sub"),
        cell(results, "ipc", "message pub/sub"),
        cell(results, "inproc", "message pub/sub"),
    ));
    md.push_str(&format!(
        "| message 订阅 | {} | {} | {} | {} |\n\n",
        cell(results, "tcp", "message pub/sub"),
        cell(results, "ipc", "message pub/sub"),
        cell(results, "inproc", "message pub/sub"),
        cell(results, "grpc", "message Subscribe"),
    ));

    for group in ["tcp", "ipc", "inproc", "grpc"] {
        md.push_str(&format!("## {group}\n\n"));
        md.push_str(
            "| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) | 备注 |\n",
        );
        md.push_str(
            "|------|----------|------|------|------|----------|----------|----------|-----------|------|\n",
        );
        for r in results.iter().filter(|r| r.transport == group) {
            if let Some(note) = &r.note {
                md.push_str(&format!(
                    "| {} | — | — | — | — | — | — | — | — | {} |\n",
                    r.scenario, note
                ));
            } else {
                md.push_str(&format!(
                    "| {} | {} | {} | {:.3}s | {:.0}/s | {:.1} | {:.1} | {:.1} | {:.1} | |\n",
                    r.scenario,
                    r.iterations,
                    r.received,
                    r.elapsed.as_secs_f64(),
                    r.throughput_per_s,
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
    md.push_str("```bash\njust perf\n# 或\ncargo run --release --bin robot_bus_perf\n```\n");

    fs::write(&path, md)?;
    Ok(path)
}

fn cell(results: &[ScenarioResult], transport: &str, scenario: &str) -> String {
    match results
        .iter()
        .find(|r| r.transport == transport && r.scenario == scenario)
    {
        Some(r) if r.note.is_some() => "—".into(),
        Some(r) => format!("{:.0}/s · {:.0}", r.throughput_per_s, r.latency.p50_us),
        None => "—".into(),
    }
}

pub fn env_summary() -> Vec<String> {
    let mut lines = vec![
        format!("robot-bus: {}", env!("CARGO_PKG_VERSION")),
        format!("构建: release (`cargo run --release`)"),
    ];

    if let Ok(out) = std::process::Command::new("hostname").output() {
        if out.status.success() {
            let h = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !h.is_empty() {
                lines.push(format!("主机名: {h}"));
            }
        }
    }

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
    push_sysctl(&mut lines, "机型", "hw.model");

    if let Ok(out) = std::process::Command::new("rustc").args(["-V"]).output() {
        if out.status.success() {
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !v.is_empty() {
                lines.push(format!("rustc: {v}"));
            }
        }
    }

    lines.push(
        "负载说明: 本机回环单进程 broker + SDK；非跨机、非多订阅者压测".into(),
    );
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
