//! robot-bus performance harness — writes `docs/zh/perf-report.md` and `docs/en/perf-report.md`.
//!
//! Sources live under `benches/robot_bus_perf/`.
//! Run: `just perf` or `cargo run --release --bin robot_bus_perf`

#[path = "federation.rs"]
mod federation;
#[path = "native.rs"]
mod native;
#[path = "pacing.rs"]
mod pacing;
#[path = "support.rs"]
mod support;
#[path = "ws.rs"]
mod ws;

use std::thread;
use std::time::Duration;

use robot_bus::{Context, RobotBusBroker};
use support::{ScenarioResult, env_summary, lock_broker, perf_broker_config, write_report};

use federation::bench_federation;
use native::{bench_action, bench_pubsub, bench_service};
use pacing::{act_iters, svc_iters};
use ws::{bench_ws_action, bench_ws_publish, bench_ws_service, bench_ws_subscribe};

fn perf_only() -> String {
    std::env::var("ROBOT_BUS_PERF_ONLY")
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn main() {
    let _guard = lock_broker();
    let only = perf_only();
    let only_message = matches!(only.as_str(), "message" | "msg" | "pubsub" | "pub/sub");
    let only_federation = matches!(only.as_str(), "federation" | "fed" | "federated");
    let only_local = matches!(only.as_str(), "local");
    let skip_rpc = only_message;
    let run_local = !only_federation;
    let run_federation = !only_local;
    let svc_iters = svc_iters();
    let act_iters = act_iters();
    if skip_rpc {
        println!("ROBOT_BUS_PERF_ONLY=message — skipping service/action");
    } else {
        println!("service iters={svc_iters} action iters={act_iters}");
    }
    let mut results: Vec<ScenarioResult> = Vec::new();

    if run_local {
        println!("starting RobotBusBroker (bind_all + ws)…");
        let ctx = Context::new();
        let broker =
            RobotBusBroker::start_with_context(&ctx, perf_broker_config()).expect("start broker");
        thread::sleep(Duration::from_millis(300));

        for transport in ["tcp", "ipc", "inproc"] {
            println!("=== {transport} ===");
            results.push(bench_pubsub(&ctx, transport));
            if !skip_rpc {
                results.push(bench_service(&ctx, transport, svc_iters));
                results.push(bench_action(&ctx, transport, act_iters));
            }
        }

        let ws_url = broker.api_url();
        println!("=== ws ({ws_url}) ===");
        results.push(bench_ws_publish(&broker, &ws_url));
        results.push(bench_ws_subscribe(&broker, &ws_url));
        if !skip_rpc {
            results.push(bench_ws_service(&broker, &ws_url, svc_iters));
            results.push(bench_ws_action(&broker, &ws_url, act_iters));
        }

        broker.stop().expect("stop broker");
    } else {
        println!("ROBOT_BUS_PERF_ONLY=federation — skipping local tcp/ipc/inproc/ws");
    }

    if run_federation {
        println!("=== federation (tcp A→B) ===");
        results.extend(bench_federation(skip_rpc));
    }

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

    if std::env::var("ROBOT_BUS_PERF_SKIP_REPORT")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false)
    {
        println!("ROBOT_BUS_PERF_SKIP_REPORT set — not writing docs/*perf-report.md");
        return;
    }

    let path = write_report(&results, &env_summary()).expect("write report");
    println!("wrote {} (and docs/en/perf-report.md)", path.display());
}
