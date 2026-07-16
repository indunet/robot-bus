//! CLI / argv parsing for [`RobotBusConfig`](super::RobotBusConfig).
//!
//! Shared by `robot_bus_broker` and the Python `robot-bus-broker` entry point.

use anyhow::{bail, Context, Result};

use super::RobotBusConfig;

#[cfg(feature = "grpc")]
use std::net::SocketAddr;

fn normalize_tcp_bind(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else {
        format!("tcp://{addr}")
    }
}

fn require_arg<'a>(args: &'a [String], i: usize, flag: &str) -> Result<&'a str> {
    args.get(i)
        .map(String::as_str)
        .with_context(|| format!("{flag} requires a value"))
}

fn parse_i32(flag: &str, value: &str) -> Result<i32> {
    value
        .parse()
        .with_context(|| format!("invalid {flag}: {value}"))
}

fn parse_u64(flag: &str, value: &str) -> Result<u64> {
    value
        .parse()
        .with_context(|| format!("invalid {flag}: {value}"))
}

/// Parse broker startup flags into a [`RobotBusConfig`].
///
/// Returns `Ok(None)` when `--help` / `-h` was requested (caller should print help and exit).
pub fn parse_robot_bus_config(args: &[String]) -> Result<Option<RobotBusConfig>> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        return Ok(None);
    }

    let mut config = RobotBusConfig::default();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            // --- message bus ---
            "--message-xsub-bind" | "--xsub-bind" | "--pub-bind" => {
                i += 1;
                config.message.xsub_bind = normalize_tcp_bind(require_arg(args, i, arg)?);
            }
            "--message-xpub-bind" | "--xpub-bind" | "--sub-bind" => {
                i += 1;
                config.message.xpub_bind = normalize_tcp_bind(require_arg(args, i, arg)?);
            }
            "--message-snd-hwm" => {
                i += 1;
                config.message.snd_hwm = parse_i32(arg, require_arg(args, i, arg)?)?;
            }
            "--message-rcv-hwm" => {
                i += 1;
                config.message.rcv_hwm = parse_i32(arg, require_arg(args, i, arg)?)?;
            }

            // --- service bus ---
            "--service-frontend-bind" => {
                i += 1;
                config.service.frontend_bind = normalize_tcp_bind(require_arg(args, i, arg)?);
            }
            "--service-backend-bind" => {
                i += 1;
                config.service.backend_bind = normalize_tcp_bind(require_arg(args, i, arg)?);
            }
            "--service-snd-hwm" => {
                i += 1;
                config.service.snd_hwm = parse_i32(arg, require_arg(args, i, arg)?)?;
            }
            "--service-rcv-hwm" => {
                i += 1;
                config.service.rcv_hwm = parse_i32(arg, require_arg(args, i, arg)?)?;
            }
            "--service-heartbeat-interval-ms" => {
                i += 1;
                config.service.heartbeat_interval_ms =
                    parse_u64(arg, require_arg(args, i, arg)?)?;
            }
            "--service-heartbeat-timeout-ms" => {
                i += 1;
                config.service.heartbeat_timeout_ms =
                    parse_u64(arg, require_arg(args, i, arg)?)?;
            }

            // --- action bus ---
            "--action-frontend-bind" => {
                i += 1;
                config.action.frontend_bind = normalize_tcp_bind(require_arg(args, i, arg)?);
            }
            "--action-backend-bind" => {
                i += 1;
                config.action.backend_bind = normalize_tcp_bind(require_arg(args, i, arg)?);
            }
            "--action-snd-hwm" => {
                i += 1;
                config.action.snd_hwm = parse_i32(arg, require_arg(args, i, arg)?)?;
            }
            "--action-rcv-hwm" => {
                i += 1;
                config.action.rcv_hwm = parse_i32(arg, require_arg(args, i, arg)?)?;
            }
            "--action-heartbeat-interval-ms" => {
                i += 1;
                config.action.heartbeat_interval_ms =
                    parse_u64(arg, require_arg(args, i, arg)?)?;
            }
            "--action-heartbeat-timeout-ms" => {
                i += 1;
                config.action.heartbeat_timeout_ms =
                    parse_u64(arg, require_arg(args, i, arg)?)?;
            }
            "--action-pending-timeout-ms" => {
                i += 1;
                config.action.pending_timeout_ms = parse_u64(arg, require_arg(args, i, arg)?)?;
            }

            // --- shared bus options ---
            "--snd-hwm" => {
                i += 1;
                let v = parse_i32(arg, require_arg(args, i, arg)?)?;
                config.message.snd_hwm = v;
                config.service.snd_hwm = v;
                config.action.snd_hwm = v;
            }
            "--rcv-hwm" => {
                i += 1;
                let v = parse_i32(arg, require_arg(args, i, arg)?)?;
                config.message.rcv_hwm = v;
                config.service.rcv_hwm = v;
                config.action.rcv_hwm = v;
            }
            "--heartbeat-interval-ms" => {
                i += 1;
                let v = parse_u64(arg, require_arg(args, i, arg)?)?;
                config.service.heartbeat_interval_ms = v;
                config.action.heartbeat_interval_ms = v;
            }
            "--heartbeat-timeout-ms" => {
                i += 1;
                let v = parse_u64(arg, require_arg(args, i, arg)?)?;
                config.service.heartbeat_timeout_ms = v;
                config.action.heartbeat_timeout_ms = v;
            }
            "--tcp-only" => {
                config.message.bind_all_transports = false;
                config.service.bind_all_transports = false;
                config.action.bind_all_transports = false;
            }

            // --- gRPC ---
            #[cfg(feature = "grpc")]
            "--grpc-listen" | "--listen" => {
                i += 1;
                let value = require_arg(args, i, arg)?;
                config.grpc.listen = value
                    .parse::<SocketAddr>()
                    .with_context(|| format!("invalid {arg} {value}"))?;
            }
            #[cfg(feature = "grpc")]
            "--cors-origin" => {
                i += 1;
                config
                    .grpc
                    .cors_origins
                    .push(require_arg(args, i, arg)?.to_string());
            }
            #[cfg(not(feature = "grpc"))]
            "--grpc-listen" | "--listen" | "--cors-origin" => {
                bail!("{arg} requires the `grpc` feature");
            }

            other => bail!("unknown argument: {other} (try --help)"),
        }
        i += 1;
    }

    Ok(Some(config))
}

/// Help text for `robot_bus_broker` / `robot-bus-broker`.
pub fn robot_bus_broker_help() -> &'static str {
    "robot_bus_broker — start all ZeroMQ buses + gRPC gateway in one process\n\n\
Usage:\n  robot_bus_broker [options]\n\n\
Defaults:\n  \
message  XSUB 15560 / XPUB 15561\n  \
service  frontend 15662 / backend 15663\n  \
action   frontend 15664 / backend 15665\n  \
gRPC     0.0.0.0:15770 (gRPC + gRPC-Web)\n\n\
Message options:\n  \
--message-xsub-bind ADDR       Publisher bind (alias: --xsub-bind)\n  \
--message-xpub-bind ADDR       Subscriber bind (alias: --xpub-bind)\n  \
--message-snd-hwm N            Message send HWM\n  \
--message-rcv-hwm N            Message receive HWM\n\n\
Service options:\n  \
--service-frontend-bind ADDR   Client (REQ) bind\n  \
--service-backend-bind ADDR    Worker (DEALER) bind\n  \
--service-snd-hwm / --service-rcv-hwm N\n  \
--service-heartbeat-interval-ms N\n  \
--service-heartbeat-timeout-ms N\n\n\
Action options:\n  \
--action-frontend-bind ADDR    Client (DEALER) bind\n  \
--action-backend-bind ADDR     Worker (DEALER) bind\n  \
--action-snd-hwm / --action-rcv-hwm N\n  \
--action-heartbeat-interval-ms N\n  \
--action-heartbeat-timeout-ms N\n  \
--action-pending-timeout-ms N  NO_WORKER timeout for queued goals\n\n\
Shared bus options:\n  \
--snd-hwm / --rcv-hwm N        Apply HWM to message + service + action\n  \
--heartbeat-interval-ms N      Apply to service + action\n  \
--heartbeat-timeout-ms N       Apply to service + action\n  \
--tcp-only                     Bind TCP only (skip inproc/ipc aliases)\n\n\
gRPC options (feature `grpc`, default on):\n  \
--grpc-listen HOST:PORT        Listen address (alias: --listen)\n  \
--cors-origin ORIGIN           Allowed browser origin (repeatable)\n\n\
--help, -h                     Show this help\n\n\
Embed in code: robot_bus::RobotBusBroker::start(RobotBusConfig { ... }).\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn help_returns_none() {
        assert!(parse_robot_bus_config(&args(&["--help"]))
            .unwrap()
            .is_none());
    }

    #[test]
    fn parses_prefixed_binds_and_shared_hwm() {
        let config = parse_robot_bus_config(&args(&[
            "--message-xsub-bind",
            "127.0.0.1:20001",
            "--service-frontend-bind",
            "tcp://0.0.0.0:20002",
            "--action-backend-bind",
            "tcp://127.0.0.1:20003",
            "--snd-hwm",
            "16",
            "--tcp-only",
            "--grpc-listen",
            "127.0.0.1:20070",
            "--cors-origin",
            "http://localhost:3000",
        ]))
        .unwrap()
        .expect("config");

        assert_eq!(config.message.xsub_bind, "tcp://127.0.0.1:20001");
        assert_eq!(config.service.frontend_bind, "tcp://0.0.0.0:20002");
        assert_eq!(config.action.backend_bind, "tcp://127.0.0.1:20003");
        assert_eq!(config.message.snd_hwm, 16);
        assert_eq!(config.service.snd_hwm, 16);
        assert_eq!(config.action.snd_hwm, 16);
        assert!(!config.message.bind_all_transports);
        assert!(!config.service.bind_all_transports);
        assert!(!config.action.bind_all_transports);
        #[cfg(feature = "grpc")]
        {
            assert_eq!(config.grpc.listen.to_string(), "127.0.0.1:20070");
            assert_eq!(config.grpc.cors_origins, vec!["http://localhost:3000"]);
        }
    }
}
