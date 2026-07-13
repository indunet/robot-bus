use anyhow::{Context, Result};
use robot_bus::broker::service_bus::{
    run, ServiceBusConfig, DEFAULT_HEARTBEAT_INTERVAL_MS, DEFAULT_HEARTBEAT_TIMEOUT_MS,
    DEFAULT_RCV_HWM, DEFAULT_SND_HWM,
};
use std::env;

fn main() -> Result<()> {
    run(parse_config()?)
}

fn parse_config() -> Result<ServiceBusConfig> {
    let mut config = ServiceBusConfig::default();

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
        "service_bus_broker — ZeroMQ dual-ROUTER service bus broker\n\n\
Usage:\n  \
service_bus_broker [--frontend-bind tcp://0.0.0.0:15662] [--backend-bind tcp://0.0.0.0:15663]\n\n\
Ports:\n  \
FRONTEND 15662 — clients (REQ) connect here\n  \
BACKEND  15663 — workers (DEALER) connect here\n\n\
Wire format:\n  \
client  -> broker: [service_name][request_id][body]  (3 frames)\n  \
broker  -> worker: [worker_id][client_id][service_name][request_id][body]  (5 frames)\n  \
worker  -> broker: [client_id][service_name][request_id][body]  (4 frames)\n  \
broker  -> client: [service_name][request_id][body]  (3 frames)\n\n\
Worker registration:\n  \
READY:      [b\"READY\"][service_name]\n  \
HEARTBEAT:  [b\"HEARTBEAT\"][service_name]\n  \
DISCONNECT: [b\"DISCONNECT\"][service_name]\n\n\
The broker parses only the service_name and control frames.\n  \
The body frame is forwarded as opaque bytes (no protobuf dependency).\n\n\
Options:\n  \
--snd-hwm N               send high-water mark (default {DEFAULT_SND_HWM})\n  \
--rcv-hwm N               receive high-water mark (default {DEFAULT_RCV_HWM})\n  \
--heartbeat-interval-ms N worker heartbeat interval (default {DEFAULT_HEARTBEAT_INTERVAL_MS})\n  \
--heartbeat-timeout-ms N  worker eviction timeout (default {DEFAULT_HEARTBEAT_TIMEOUT_MS})\n"
    );
}
