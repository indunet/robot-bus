use anyhow::{Context, Result};
use robot_bus::broker::action_bus::{
    run, ActionBusConfig, DEFAULT_HEARTBEAT_INTERVAL_MS, DEFAULT_HEARTBEAT_TIMEOUT_MS,
    DEFAULT_PENDING_TIMEOUT_MS, DEFAULT_RCV_HWM, DEFAULT_SND_HWM,
};
use std::env;

fn main() -> Result<()> {
    run(parse_config()?)
}

fn parse_config() -> Result<ActionBusConfig> {
    let mut config = ActionBusConfig::default();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--frontend-bind" | "--client-bind" => {
                let addr = args.next().context("--frontend-bind requires an address")?;
                config.frontend_bind = normalize_tcp_bind(&addr);
            }
            "--backend-bind" | "--worker-bind" => {
                let addr = args.next().context("--backend-bind requires an address")?;
                config.backend_bind = normalize_tcp_bind(&addr);
            }
            "--snd-hwm" => {
                config.snd_hwm = args
                    .next()
                    .context("--snd-hwm requires a number")?
                    .parse()
                    .context("invalid --snd-hwm")?;
            }
            "--rcv-hwm" => {
                config.rcv_hwm = args
                    .next()
                    .context("--rcv-hwm requires a number")?
                    .parse()
                    .context("invalid --rcv-hwm")?;
            }
            "--heartbeat-interval-ms" => {
                config.heartbeat_interval_ms = args
                    .next()
                    .context("--heartbeat-interval-ms requires a number")?
                    .parse()
                    .context("invalid --heartbeat-interval-ms")?;
            }
            "--heartbeat-timeout-ms" => {
                config.heartbeat_timeout_ms = args
                    .next()
                    .context("--heartbeat-timeout-ms requires a number")?
                    .parse()
                    .context("invalid --heartbeat-timeout-ms")?;
            }
            "--pending-timeout-ms" => {
                config.pending_timeout_ms = args
                    .next()
                    .context("--pending-timeout-ms requires a number")?
                    .parse()
                    .context("invalid --pending-timeout-ms")?;
            }
            "--tcp-only" => {
                config.bind_all_transports = false;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => anyhow::bail!("unknown argument: {arg}"),
        }
    }

    Ok(config)
}

fn normalize_tcp_bind(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else {
        format!("tcp://{addr}")
    }
}

fn print_help() {
    println!(
        "action_bus_broker — ZeroMQ dual-ROUTER action bus broker\n\n\
Usage:\n  \
action_bus_broker [--frontend-bind tcp://0.0.0.0:15664] [--backend-bind tcp://0.0.0.0:15665]\n\n\
Ports:\n  \
FRONTEND 15664 — clients (DEALER) connect here\n  \
BACKEND  15665 — workers (DEALER) connect here\n\n\
Wire format (DEALER clients, unlike service_bus's REQ):\n  \
client  -> broker: [action_name][goal_id][GOAL|CANCEL][body]  (4 frames)\n  \
broker  -> worker: [worker_id][client_id][action_name][goal_id][GOAL|CANCEL][body]  (6 frames)\n  \
worker  -> broker: [client_id][action_name][goal_id][FEEDBACK|RESULT][body]  (5 frames)\n  \
broker  -> client: [action_name][goal_id][FEEDBACK|RESULT][body]  (4 frames)\n\n\
Worker registration:\n  \
READY:      [b\"READY\"][action_name]\n  \
HEARTBEAT:  [b\"HEARTBEAT\"][action_name]\n  \
DISCONNECT: [b\"DISCONNECT\"][action_name]\n\n\
The broker parses only the action_name, goal_id, and kind frames.\n  \
The body frame is forwarded as opaque bytes (no protobuf dependency).\n  \
A goal may produce multiple FEEDBACK messages followed by exactly one RESULT.\n\n\
Options:\n  \
--snd-hwm N               send high-water mark (default {DEFAULT_SND_HWM})\n  \
--rcv-hwm N               receive high-water mark (default {DEFAULT_RCV_HWM})\n  \
--heartbeat-interval-ms N worker heartbeat interval (default {DEFAULT_HEARTBEAT_INTERVAL_MS})\n  \
--heartbeat-timeout-ms N  worker eviction timeout (default {DEFAULT_HEARTBEAT_TIMEOUT_MS})\n  \
--pending-timeout-ms N    NO_WORKER after queued goal waits this long (default {DEFAULT_PENDING_TIMEOUT_MS})\n  \
--tcp-only                bind only the TCP endpoints (skip shared ipc/inproc aliases)\n"
    );
}
