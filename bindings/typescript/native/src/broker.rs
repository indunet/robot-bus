//! Broker lifecycle, discovery CLI entrypoints, and version.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use robot_bus::broker::{
    apply_federation_opts, parse_robot_bus_config, robot_bus_broker_help,
    RobotBusBroker as RustRobotBusBroker, RobotBusConfig,
};
use robot_bus::{shutdown, transports};

use crate::node::Context;
use crate::util::{anyhow_err, map_endpoint_err, normalize_bind};

#[napi]
pub fn message_xsub_endpoint(
    host: Option<String>,
    transport: Option<String>,
) -> Result<String> {
    let host = host.unwrap_or_else(|| "localhost".into());
    let transport = transport.unwrap_or_else(|| "tcp".into());
    transports::message_xsub_endpoint(&host, &transport).map_err(map_endpoint_err)
}

#[napi]
pub fn message_xpub_endpoint(
    host: Option<String>,
    transport: Option<String>,
) -> Result<String> {
    let host = host.unwrap_or_else(|| "localhost".into());
    let transport = transport.unwrap_or_else(|| "tcp".into());
    transports::message_xpub_endpoint(&host, &transport).map_err(map_endpoint_err)
}

#[napi(object)]
pub struct BrokerStartOptions {
    pub message_xsub_bind: Option<String>,
    pub message_xpub_bind: Option<String>,
    pub message_snd_hwm: Option<i32>,
    pub message_rcv_hwm: Option<i32>,
    pub service_frontend_bind: Option<String>,
    pub service_backend_bind: Option<String>,
    pub service_snd_hwm: Option<i32>,
    pub service_rcv_hwm: Option<i32>,
    pub service_heartbeat_interval_ms: Option<u32>,
    pub service_heartbeat_timeout_ms: Option<u32>,
    pub action_frontend_bind: Option<String>,
    pub action_backend_bind: Option<String>,
    pub action_snd_hwm: Option<i32>,
    pub action_rcv_hwm: Option<i32>,
    pub action_heartbeat_interval_ms: Option<u32>,
    pub action_heartbeat_timeout_ms: Option<u32>,
    pub action_pending_timeout_ms: Option<u32>,
    pub snd_hwm: Option<i32>,
    pub rcv_hwm: Option<i32>,
    pub heartbeat_interval_ms: Option<u32>,
    pub heartbeat_timeout_ms: Option<u32>,
    pub tcp_only: Option<bool>,
    pub api_listen: Option<String>,
    pub cors_origins: Option<Vec<String>>,
    pub console_listen: Option<String>,
    pub no_console: Option<bool>,
    pub no_tank: Option<bool>,
    pub broker_id: Option<String>,
    pub message_peers: Option<Vec<String>>,
    pub service_peers: Option<Vec<String>>,
    pub action_peers: Option<Vec<String>>,
    pub peers: Option<Vec<String>>,
    pub domain_id: Option<u32>,
    pub no_discovery: Option<bool>,
    pub advertise_host: Option<String>,
}

#[napi]
pub struct RobotBusBroker {
    pub(crate) inner: Option<RustRobotBusBroker>,
}

#[napi]
impl RobotBusBroker {
    #[napi(factory)]
    pub fn start(
        options: Option<BrokerStartOptions>,
        context: Option<&Context>,
    ) -> Result<Self> {
        let mut config = RobotBusConfig::default();
        if let Some(o) = options {
            if let Some(v) = o.message_xsub_bind {
                config.message.xsub_bind = normalize_bind(&v);
            }
            if let Some(v) = o.message_xpub_bind {
                config.message.xpub_bind = normalize_bind(&v);
            }
            if let Some(v) = o.message_snd_hwm {
                config.message.snd_hwm = v;
            }
            if let Some(v) = o.message_rcv_hwm {
                config.message.rcv_hwm = v;
            }
            if let Some(v) = o.service_frontend_bind {
                config.service.frontend_bind = normalize_bind(&v);
            }
            if let Some(v) = o.service_backend_bind {
                config.service.backend_bind = normalize_bind(&v);
            }
            if let Some(v) = o.service_snd_hwm {
                config.service.snd_hwm = v;
            }
            if let Some(v) = o.service_rcv_hwm {
                config.service.rcv_hwm = v;
            }
            if let Some(v) = o.service_heartbeat_interval_ms {
                config.service.heartbeat_interval_ms = v as u64;
            }
            if let Some(v) = o.service_heartbeat_timeout_ms {
                config.service.heartbeat_timeout_ms = v as u64;
            }
            if let Some(v) = o.action_frontend_bind {
                config.action.frontend_bind = normalize_bind(&v);
            }
            if let Some(v) = o.action_backend_bind {
                config.action.backend_bind = normalize_bind(&v);
            }
            if let Some(v) = o.action_snd_hwm {
                config.action.snd_hwm = v;
            }
            if let Some(v) = o.action_rcv_hwm {
                config.action.rcv_hwm = v;
            }
            if let Some(v) = o.action_heartbeat_interval_ms {
                config.action.heartbeat_interval_ms = v as u64;
            }
            if let Some(v) = o.action_heartbeat_timeout_ms {
                config.action.heartbeat_timeout_ms = v as u64;
            }
            if let Some(v) = o.action_pending_timeout_ms {
                config.action.pending_timeout_ms = v as u64;
            }
            if let Some(v) = o.snd_hwm {
                config.message.snd_hwm = v;
                config.service.snd_hwm = v;
                config.action.snd_hwm = v;
            }
            if let Some(v) = o.rcv_hwm {
                config.message.rcv_hwm = v;
                config.service.rcv_hwm = v;
                config.action.rcv_hwm = v;
            }
            if let Some(v) = o.heartbeat_interval_ms {
                config.service.heartbeat_interval_ms = v as u64;
                config.action.heartbeat_interval_ms = v as u64;
            }
            if let Some(v) = o.heartbeat_timeout_ms {
                config.service.heartbeat_timeout_ms = v as u64;
                config.action.heartbeat_timeout_ms = v as u64;
            }
            if o.tcp_only.unwrap_or(false) {
                config.message.bind_all_transports = false;
                config.service.bind_all_transports = false;
                config.action.bind_all_transports = false;
            }
            if let Some(v) = o.cors_origins {
                config.ws.cors_origins = v;
            }
            if o.no_console.unwrap_or(false) {
                config.console.enabled = false;
            }
            if o.no_tank.unwrap_or(false) {
                config.console.tank_enabled = false;
            }
            if let Some(v) = o.console_listen {
                config.console.listen = v
                    .parse()
                    .map_err(|e| Error::from_reason(format!("invalid console_listen: {e}")))?;
                config.console.enabled = true;
            }
            apply_federation_opts(
                &mut config,
                o.broker_id.as_deref(),
                o.message_peers.as_deref().unwrap_or(&[]),
                o.service_peers.as_deref().unwrap_or(&[]),
                o.action_peers.as_deref().unwrap_or(&[]),
            )
            .map_err(anyhow_err)?;
            if let Some(peers) = &o.peers {
                robot_bus::apply_api_peers(&mut config, peers).map_err(anyhow_err)?;
            }
            if o.no_discovery.unwrap_or(false) {
                config.discovery.enabled = false;
            }
            if let Some(v) = o.domain_id {
                config.discovery.domain_id = v;
            }
            if let Some(v) = o.advertise_host {
                if !v.is_empty() {
                    config.discovery.advertise_host = Some(v);
                }
            }
            if let Some(v) = o.api_listen {
                if !v.is_empty() {
                    config.ws.listen = v
                        .parse()
                        .map_err(|e| Error::from_reason(format!("invalid api_listen: {e}")))?;
                    config.console.listen = config.ws.listen;
                }
            }
        }

        let broker = match context {
            Some(c) => RustRobotBusBroker::start_with_context(&c.inner, config),
            None => RustRobotBusBroker::start(config),
        }
        .map_err(anyhow_err)?;
        Ok(Self {
            inner: Some(broker),
        })
    }

    #[napi]
    pub fn stop(&mut self) -> Result<()> {
        if let Some(broker) = self.inner.take() {
            broker.stop().map_err(anyhow_err)?;
        }
        Ok(())
    }

    fn with_broker<T>(&self, f: impl FnOnce(&RustRobotBusBroker) -> T) -> Result<T> {
        self.inner
            .as_ref()
            .map(f)
            .ok_or_else(|| Error::from_reason("broker already stopped"))
    }

    #[napi(getter)]
    pub fn message_xsub_bind(&self) -> Result<String> {
        self.with_broker(|b| b.message.xsub_bind.clone())
    }

    #[napi(getter)]
    pub fn message_xpub_bind(&self) -> Result<String> {
        self.with_broker(|b| b.message.xpub_bind.clone())
    }

    #[napi(getter)]
    pub fn service_frontend_bind(&self) -> Result<String> {
        self.with_broker(|b| b.service.frontend_bind.clone())
    }

    #[napi(getter)]
    pub fn service_backend_bind(&self) -> Result<String> {
        self.with_broker(|b| b.service.backend_bind.clone())
    }

    #[napi(getter)]
    pub fn action_frontend_bind(&self) -> Result<String> {
        self.with_broker(|b| b.action.frontend_bind.clone())
    }

    #[napi(getter)]
    pub fn action_backend_bind(&self) -> Result<String> {
        self.with_broker(|b| b.action.backend_bind.clone())
    }

    #[napi(getter)]
    pub fn api_listen(&self) -> Result<String> {
        self.with_broker(|b| b.api_listen().to_string())
    }

    #[napi(getter)]
    pub fn console_listen(&self) -> Result<Option<String>> {
        self.with_broker(|b| b.console_listen().map(|a| a.to_string()))
    }
}

/// Blocking CLI entry: start broker and wait for Ctrl+C.
#[napi]
pub fn run_broker() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = match parse_robot_bus_config(&args).map_err(anyhow_err)? {
        None => {
            print!("{}", robot_bus_broker_help());
            return Ok(());
        }
        Some(config) => config,
    };

    let flag = Arc::new(AtomicBool::new(false));
    shutdown::install(flag.clone());

    println!("robot-bus-broker starting message + service + action buses + WebSocket + console…");
    let broker = RustRobotBusBroker::start(config).map_err(anyhow_err)?;
    let mut broker = RobotBusBroker {
        inner: Some(broker),
    };
    println!(
        "WebSocket RPC listening on http://{}",
        broker.api_listen()?
    );
    if let Some(addr) = broker.console_listen()? {
        println!("Web console listening on http://{addr}");
    }

    while !flag.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(50));
    }

    broker.stop()?;
    println!("robot-bus-broker stopped");
    Ok(())
}

#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
