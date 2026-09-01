//! CLI / argv parsing for [`RobotBusConfig`](super::RobotBusConfig).
//!
//! Shared by `robot_bus_broker`, `python -m robot_bus.broker`, and `npx robot-bus`.

use anyhow::{Context, Result, bail};

use super::RobotBusConfig;
use super::action_bus::ActionPeer;
use super::message_bus::MessagePeer;
use super::service_bus::ServicePeer;

#[cfg(any(feature = "ws", feature = "console"))]
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

/// Drop leading non-flag tokens (Node script path, `tsx`, etc.).
pub fn strip_runtime_argv(mut args: Vec<String>) -> Vec<String> {
    while args.first().is_some_and(|a| !a.starts_with('-')) {
        args.remove(0);
    }
    args
}

/// CLI args for language-binding `run_broker` entrypoints.
///
/// Drops argv0 and any leading non-flag tokens so [`parse_robot_bus_config`]
/// only sees `--flags`.
pub fn binding_broker_cli_args() -> Vec<String> {
    strip_runtime_argv(std::env::args().skip(1).collect())
}

/// Parse broker startup flags into a [`RobotBusConfig`].
///
/// Returns `Ok(None)` when `--help` / `-h` was requested (caller should print help and exit).
pub fn parse_robot_bus_config(args: &[String]) -> Result<Option<RobotBusConfig>> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        return Ok(None);
    }

    let mut config = RobotBusConfig::default();
    let mut api_peers: Vec<String> = Vec::new();
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
            "--broker-id" => {
                i += 1;
                let id = require_arg(args, i, arg)?.to_string();
                config.message.broker_id = id.clone();
                config.service.broker_id = id.clone();
                config.action.broker_id = id;
            }
            "--message-peer" => {
                i += 1;
                let value = require_arg(args, i, arg)?;
                config.message.peers.push(
                    MessagePeer::from_xpub(value)
                        .with_context(|| format!("invalid --message-peer {value}"))?,
                );
            }
            "--peer" => {
                i += 1;
                api_peers.push(require_arg(args, i, arg)?.to_string());
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
                config.service.heartbeat_interval_ms = parse_u64(arg, require_arg(args, i, arg)?)?;
            }
            "--service-heartbeat-timeout-ms" => {
                i += 1;
                config.service.heartbeat_timeout_ms = parse_u64(arg, require_arg(args, i, arg)?)?;
            }
            "--service-pending-timeout-ms" => {
                i += 1;
                config.service.pending_timeout_ms = parse_u64(arg, require_arg(args, i, arg)?)?;
            }
            "--service-max-pending" => {
                i += 1;
                config.service.max_pending = parse_u64(arg, require_arg(args, i, arg)?)? as usize;
            }
            "--service-peer" => {
                i += 1;
                let value = require_arg(args, i, arg)?;
                // Prefer backend form; if the port looks like a frontend default
                // pair, `from_backend` still accepts an explicit backend port.
                config.service.peers.push(
                    ServicePeer::from_backend(value)
                        .with_context(|| format!("invalid --service-peer {value}"))?,
                );
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
                config.action.heartbeat_interval_ms = parse_u64(arg, require_arg(args, i, arg)?)?;
            }
            "--action-heartbeat-timeout-ms" => {
                i += 1;
                config.action.heartbeat_timeout_ms = parse_u64(arg, require_arg(args, i, arg)?)?;
            }
            "--action-pending-timeout-ms" => {
                i += 1;
                config.action.pending_timeout_ms = parse_u64(arg, require_arg(args, i, arg)?)?;
            }
            "--action-peer" => {
                i += 1;
                let value = require_arg(args, i, arg)?;
                config.action.peers.push(
                    ActionPeer::from_backend(value)
                        .with_context(|| format!("invalid --action-peer {value}"))?,
                );
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

            // --- discovery (HTTP via API listen; advertise host only) ---
            "--domain-id" => {
                i += 1;
                config.discovery.domain_id = require_arg(args, i, arg)?
                    .parse()
                    .with_context(|| format!("invalid {arg}"))?;
            }
            "--no-discovery" => {
                // Kept for compatibility; UDP announce is already removed.
                config.discovery.enabled = false;
            }
            "--advertise-host" => {
                i += 1;
                config.discovery.advertise_host = Some(require_arg(args, i, arg)?.to_string());
            }
            "--discovery-port" | "--discovery-addr" => {
                // Deprecated UDP flags: ignored with a soft warning via bail message
                // that points users to --api-listen / --peer.
                let _ = require_arg(args, i + 1, arg)?;
                i += 1;
                log::warn!(
                    "{arg} is deprecated: UDP discovery was removed; use --api-listen and GET /api/v1/discover"
                );
            }

            // --- API listen (WS + console + discover) ---
            #[cfg(feature = "ws")]
            "--api-listen" | "--listen" => {
                i += 1;
                let value = require_arg(args, i, arg)?;
                config.ws.listen = value
                    .parse::<SocketAddr>()
                    .with_context(|| format!("invalid {arg} {value}"))?;
                #[cfg(feature = "console")]
                {
                    config.console.listen = config.ws.listen;
                }
            }
            #[cfg(feature = "ws")]
            "--cors-origin" => {
                i += 1;
                config
                    .ws
                    .cors_origins
                    .push(require_arg(args, i, arg)?.to_string());
            }
            #[cfg(not(feature = "ws"))]
            "--api-listen" | "--listen" | "--cors-origin" => {
                bail!("{arg} requires the `ws` feature");
            }

            // --- console ---
            #[cfg(feature = "console")]
            "--console-listen" => {
                i += 1;
                let value = require_arg(args, i, arg)?;
                let addr = value
                    .parse::<SocketAddr>()
                    .with_context(|| format!("invalid {arg} {value}"))?;
                config.console.listen = addr;
                config.console.enabled = true;
                // Single-port: console is co-located with the API listen when both features are on.
                #[cfg(feature = "ws")]
                {
                    config.ws.listen = addr;
                }
            }
            #[cfg(feature = "console")]
            "--no-console" => {
                config.console.enabled = false;
            }
            #[cfg(feature = "console")]
            "--no-tank" => {
                config.console.tank_enabled = false;
            }
            #[cfg(feature = "console")]
            "--no-docs" => {
                config.console.docs_enabled = false;
            }
            #[cfg(feature = "console")]
            "--console-cors-origin" => {
                i += 1;
                config
                    .console
                    .cors_origins
                    .push(require_arg(args, i, arg)?.to_string());
            }
            #[cfg(not(feature = "console"))]
            "--console-listen"
            | "--no-console"
            | "--no-tank"
            | "--no-docs"
            | "--console-cors-origin" => {
                bail!("{arg} requires the `console` feature");
            }

            other => bail!("unknown argument: {other} (try --help)"),
        }
        i += 1;
    }

    if !api_peers.is_empty() {
        super::federation::apply_api_peers(&mut config, &api_peers)?;
    }

    Ok(Some(config))
}

/// Help text for `python -m robot_bus.broker` / `npx robot-bus` / `cargo run --bin robot_bus_broker` / `robot_bus_broker`.
pub fn robot_bus_broker_help() -> &'static str {
    "robot-bus broker — start all ZeroMQ buses + API gateway + Web console in one process\n\n\
Usage:\n  \
python -m robot_bus.broker [options]\n  \
npx robot-bus [options]\n  \
cargo run --bin robot_bus_broker -- [options]\n  \
robot_bus_broker [options]\n\n\
Defaults:\n  \
message / service / action TCP binds: 0.0.0.0:0 (OS assigns free ports)\n  \
API listen 0.0.0.0:15570 (WS /ws + discover REST + embedded Web UI)\n  \
console  same port as API (use --no-console to disable UI; discover still on API)\n\n\
Message options:\n  \
--message-xsub-bind ADDR       Publisher bind (default tcp://0.0.0.0:0; alias: --xsub-bind)\n  \
--message-xpub-bind ADDR       Subscriber bind (default tcp://0.0.0.0:0; alias: --xpub-bind)\n  \
--message-snd-hwm N            Message send HWM\n  \
--message-rcv-hwm N            Message receive HWM\n  \
--broker-id ID                 Hop-path id for message/service/action federation (default: random UUID)\n  \
--message-peer tcp://HOST:XPUB Peer broker XPUB (advanced; XSUB = XPUB port - 1)\n  \
--peer HOST:PORT               Federation peer via API listen (GET /api/v1/discover; repeatable)\n\n\
Service options:\n  \
--service-frontend-bind ADDR   Client (REQ) bind (default :0)\n  \
--service-backend-bind ADDR    Worker (DEALER) bind (default :0)\n  \
--service-snd-hwm / --service-rcv-hwm N\n  \
--service-heartbeat-interval-ms N\n  \
--service-heartbeat-timeout-ms N\n  \
--service-pending-timeout-ms N     NO_WORKER timeout for queued requests\n  \
--service-max-pending N            Max queued requests before NO_WORKER\n  \
--service-peer [ID=]tcp://HOST:BE  Peer service backend (advanced; optional ID=)\n\n\
Action options:\n  \
--action-frontend-bind ADDR    Client (DEALER) bind (default :0)\n  \
--action-backend-bind ADDR     Worker (DEALER) bind (default :0)\n  \
--action-snd-hwm / --action-rcv-hwm N\n  \
--action-heartbeat-interval-ms N\n  \
--action-heartbeat-timeout-ms N\n  \
--action-pending-timeout-ms N  NO_WORKER timeout for queued goals\n  \
--action-peer [ID=]tcp://HOST:BE  Peer action backend (advanced; optional ID=)\n\n\
Shared bus options:\n  \
--snd-hwm / --rcv-hwm N        Apply HWM to message + service + action\n  \
--heartbeat-interval-ms N      Apply to service + action\n  \
--heartbeat-timeout-ms N       Apply to service + action\n  \
--tcp-only                     Bind TCP only (skip inproc/ipc aliases)\n\n\
Discovery / advertise:\n  \
--domain-id N                  Soft label returned in /api/v1/discover (default: 0)\n  \
--advertise-host HOST          Host clients should connect to (default: inferred)\n  \
--no-discovery                 Compatibility no-op (UDP announce removed)\n\n\
API options (feature `ws`, default on):\n  \
--api-listen HOST:PORT         API listen (aliases: --listen, --console-listen)\n  \
--cors-origin ORIGIN           Allowed browser origin (repeatable)\n\n\
Console options (feature `console`, default on):\n  \
--console-listen HOST:PORT     Alias of --api-listen (single-port UI + API)\n  \
--no-console                   Do not start the Web console UI\n  \
--no-tank                      Hide tank demo in console; reject /api/v1/tank/session\n  \
--no-docs                      Hide docs sidebar in console (shown by default)\n  \
--console-cors-origin ORIGIN   Allow browser origin (repeatable)\n\n\
--help, -h                     Show this help\n\n\
Embed in code: robot_bus::RobotBusBroker::start(RobotBusConfig { ... }).\n\
Clients: GET http://HOST:15570/api/v1/discover then connect to returned ZMQ endpoints.\n"
}

/// Apply federation options shared by language bindings (CLI-compatible string forms).
///
/// - `broker_id`: hop-path id for message/service/action (empty → leave defaults)
/// - `message_peers`: peer XPUB endpoints (`MessagePeer::from_xpub`)
/// - `service_peers`: peer backends (`ServicePeer::from_backend`, optional `id=`)
/// - `action_peers`: peer backends (`ActionPeer::from_backend`, optional `id=`)
pub fn apply_federation_opts(
    config: &mut RobotBusConfig,
    broker_id: Option<&str>,
    message_peers: &[String],
    service_peers: &[String],
    action_peers: &[String],
) -> Result<()> {
    if let Some(id) = broker_id {
        if !id.is_empty() {
            config.message.broker_id = id.to_string();
            config.service.broker_id = id.to_string();
            config.action.broker_id = id.to_string();
        }
    }

    for value in message_peers {
        config.message.peers.push(
            MessagePeer::from_xpub(value)
                .with_context(|| format!("invalid message peer {value}"))?,
        );
    }
    for value in service_peers {
        config.service.peers.push(
            ServicePeer::from_backend(value)
                .with_context(|| format!("invalid service peer {value}"))?,
        );
    }
    for value in action_peers {
        config.action.peers.push(
            ActionPeer::from_backend(value)
                .with_context(|| format!("invalid action peer {value}"))?,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn help_returns_none() {
        assert!(
            parse_robot_bus_config(&args(&["--help"]))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn strip_runtime_argv_drops_node_script_path() {
        assert_eq!(
            strip_runtime_argv(args(&["/tmp/start.mjs", "--tcp-only", "--no-console"])),
            args(&["--tcp-only", "--no-console"])
        );
        assert_eq!(
            strip_runtime_argv(args(&[
                "tsx",
                "broker.ts",
                "--api-listen",
                "127.0.0.1:15570"
            ])),
            args(&["--api-listen", "127.0.0.1:15570"])
        );
        assert_eq!(
            strip_runtime_argv(args(&["--tcp-only"])),
            args(&["--tcp-only"])
        );
    }

    #[test]
    fn parses_prefixed_binds_and_shared_hwm() {
        #[allow(unused_mut)] // mutated only when feature `ws` is enabled
        let mut argv = args(&[
            "--message-xsub-bind",
            "127.0.0.1:20001",
            "--service-frontend-bind",
            "tcp://0.0.0.0:20002",
            "--action-backend-bind",
            "tcp://127.0.0.1:20003",
            "--snd-hwm",
            "16",
            "--tcp-only",
        ]);
        #[cfg(feature = "ws")]
        {
            argv.extend(args(&[
                "--api-listen",
                "127.0.0.1:20070",
                "--cors-origin",
                "http://localhost:3000",
            ]));
        }
        let config = parse_robot_bus_config(&argv).unwrap().expect("config");

        assert_eq!(config.message.xsub_bind, "tcp://127.0.0.1:20001");
        assert_eq!(config.service.frontend_bind, "tcp://0.0.0.0:20002");
        assert_eq!(config.action.backend_bind, "tcp://127.0.0.1:20003");
        assert_eq!(config.message.snd_hwm, 16);
        assert_eq!(config.service.snd_hwm, 16);
        assert_eq!(config.action.snd_hwm, 16);
        assert!(!config.message.bind_all_transports);
        assert!(!config.service.bind_all_transports);
        assert!(!config.action.bind_all_transports);
        #[cfg(feature = "ws")]
        {
            assert_eq!(config.ws.listen.to_string(), "127.0.0.1:20070");
            assert_eq!(config.ws.cors_origins, vec!["http://localhost:3000"]);
        }
    }

    #[cfg(feature = "console")]
    #[test]
    fn parses_console_flags() {
        let config = parse_robot_bus_config(&args(&["--console-listen", "127.0.0.1:25771"]))
            .unwrap()
            .expect("config");
        assert!(config.console.enabled);
        assert_eq!(config.console.listen.to_string(), "127.0.0.1:25771");
        #[cfg(feature = "ws")]
        assert_eq!(config.ws.listen.to_string(), "127.0.0.1:25771");

        let config = parse_robot_bus_config(&args(&["--no-console"]))
            .unwrap()
            .expect("config");
        assert!(!config.console.enabled);

        let config = parse_robot_bus_config(&args(&["--no-tank"]))
            .unwrap()
            .expect("config");
        assert!(config.console.enabled);
        assert!(!config.console.tank_enabled);
        assert!(config.console.docs_enabled);

        let config = parse_robot_bus_config(&args(&["--no-docs"]))
            .unwrap()
            .expect("config");
        assert!(config.console.enabled);
        assert!(config.console.tank_enabled);
        assert!(!config.console.docs_enabled);
    }

    #[test]
    fn parses_broker_id_and_message_peers() {
        let config = parse_robot_bus_config(&args(&[
            "--broker-id",
            "broker-a",
            "--message-peer",
            "tcp://127.0.0.1:16561",
            "--message-peer",
            "10.0.0.2:17561",
        ]))
        .unwrap()
        .expect("config");
        assert_eq!(config.message.broker_id, "broker-a");
        assert_eq!(config.service.broker_id, "broker-a");
        assert_eq!(config.action.broker_id, "broker-a");
        assert_eq!(config.message.peers.len(), 2);
        assert_eq!(config.message.peers[0].xpub, "tcp://127.0.0.1:16561");
        assert_eq!(config.message.peers[0].xsub, "tcp://127.0.0.1:16560");
        assert_eq!(config.message.peers[1].xpub, "tcp://10.0.0.2:17561");
        assert_eq!(config.message.peers[1].xsub, "tcp://10.0.0.2:17560");
    }

    #[test]
    fn parses_discovery_flags() {
        let config = parse_robot_bus_config(&args(&[
            "--domain-id",
            "3",
            "--advertise-host",
            "10.0.0.5",
            "--discovery-port",
            "45550",
            "--discovery-addr",
            "239.255.76.67",
            "--no-discovery",
        ]))
        .unwrap()
        .expect("config");
        assert!(!config.discovery.enabled);
        assert_eq!(config.discovery.domain_id, 3);
        assert_eq!(config.discovery.advertise_host.as_deref(), Some("10.0.0.5"));
    }

    #[test]
    fn parses_api_listen_alias() {
        let result = parse_robot_bus_config(&args(&["--api-listen", "127.0.0.1:15771"]));
        #[cfg(feature = "ws")]
        {
            let config = result.unwrap().expect("config");
            assert_eq!(config.ws.listen.to_string(), "127.0.0.1:15771");
        }
        #[cfg(not(feature = "ws"))]
        {
            let err = result.expect_err("--api-listen requires ws");
            assert!(err.to_string().contains("requires the `ws` feature"));
        }
    }

    #[test]
    fn parses_service_peers() {
        let config = parse_robot_bus_config(&args(&[
            "--broker-id",
            "broker-a",
            "--service-peer",
            "tcp://127.0.0.1:16663",
            "--service-peer",
            "broker-c=10.0.0.2:15663",
        ]))
        .unwrap()
        .expect("config");
        assert_eq!(config.service.broker_id, "broker-a");
        assert_eq!(config.service.peers.len(), 2);
        assert_eq!(config.service.peers[0].backend, "tcp://127.0.0.1:16663");
        assert!(config.service.peers[0].broker_id.is_empty());
        assert_eq!(config.service.peers[1].backend, "tcp://10.0.0.2:15663");
        assert_eq!(config.service.peers[1].broker_id, "broker-c");
    }

    #[test]
    fn parses_action_peers() {
        let config = parse_robot_bus_config(&args(&[
            "--broker-id",
            "broker-a",
            "--action-peer",
            "tcp://127.0.0.1:16665",
            "--action-peer",
            "broker-c=10.0.0.2:15665",
        ]))
        .unwrap()
        .expect("config");
        assert_eq!(config.action.broker_id, "broker-a");
        assert_eq!(config.action.peers.len(), 2);
        assert_eq!(config.action.peers[0].backend, "tcp://127.0.0.1:16665");
        assert!(config.action.peers[0].broker_id.is_empty());
        assert_eq!(config.action.peers[1].backend, "tcp://10.0.0.2:15665");
        assert_eq!(config.action.peers[1].broker_id, "broker-c");
    }

    #[test]
    fn apply_federation_opts_sets_id_and_peers() {
        let mut config = RobotBusConfig::default();
        apply_federation_opts(
            &mut config,
            Some("broker-a"),
            &["tcp://10.0.0.2:15561".to_string()],
            &["broker-b=tcp://10.0.0.2:15663".to_string()],
            &["broker-b=tcp://10.0.0.2:15665".to_string()],
        )
        .unwrap();
        assert_eq!(config.message.broker_id, "broker-a");
        assert_eq!(config.service.broker_id, "broker-a");
        assert_eq!(config.action.broker_id, "broker-a");
        assert_eq!(config.message.peers.len(), 1);
        assert_eq!(config.message.peers[0].xpub, "tcp://10.0.0.2:15561");
        assert_eq!(config.message.peers[0].xsub, "tcp://10.0.0.2:15560");
        assert_eq!(config.service.peers[0].broker_id, "broker-b");
        assert_eq!(config.action.peers[0].broker_id, "broker-b");
    }

    #[test]
    fn apply_federation_opts_rejects_bad_message_peer() {
        let mut config = RobotBusConfig::default();
        let err = apply_federation_opts(
            &mut config,
            None,
            &["tcp://127.0.0.1:0".to_string()],
            &[],
            &[],
        )
        .unwrap_err();
        assert!(err.to_string().contains("invalid message peer"));
    }
}
