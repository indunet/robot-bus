//! Two-broker TCP federation scenarios (A → B).
//!
//! Each broker gets its own ZMQ context (`RobotBusBroker::start`). Nodes pin
//! explicit TCP endpoints so they do not share ipc/inproc names with the
//! local bind_all bench.

use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "console-api")]
use robot_bus::ConsoleBrokerConfig;
use robot_bus::action_bus::ActionClient;
use robot_bus::broker::action_bus::{ActionBusConfig, ActionPeer};
use robot_bus::broker::message_bus::{BusConfig, MessagePeer};
use robot_bus::broker::service_bus::{ServiceBusConfig, ServicePeer};
use robot_bus::service_bus::ServiceClient;
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{
    DiscoveryConfig, HighWaterMark, Node, NodeOptions, Publisher, RobotBusBroker, RobotBusConfig,
};

#[cfg(feature = "ws")]
use robot_bus::WsConfig;

use crate::native::run_pubsub;
use crate::pacing::{MSG_HWM, WARMUP, act_iters, make_payload, svc_iters};
use crate::support::{LatencyStats, ScenarioResult};

const TRANSPORT: &str = "federation";

fn connect_addr(bind: &str) -> String {
    bind.replace("tcp://0.0.0.0:", "tcp://127.0.0.1:")
        .replace("tcp://*:", "tcp://127.0.0.1:")
}

fn free_ports(n: usize) -> Vec<u16> {
    let listeners: Vec<TcpListener> = (0..n)
        .map(|_| TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port"))
        .collect();
    listeners
        .iter()
        .map(|l| l.local_addr().expect("local addr").port())
        .collect()
}

struct BrokerPorts {
    xsub: u16,
    xpub: u16,
    svc_fe: u16,
    svc_be: u16,
    act_fe: u16,
    act_be: u16,
}

fn alloc_ports() -> (BrokerPorts, BrokerPorts) {
    let raw = free_ports(12);
    let a = BrokerPorts {
        xsub: raw[0],
        xpub: raw[1],
        svc_fe: raw[2],
        svc_be: raw[3],
        act_fe: raw[4],
        act_be: raw[5],
    };
    let b = BrokerPorts {
        xsub: raw[6],
        xpub: raw[7],
        svc_fe: raw[8],
        svc_be: raw[9],
        act_fe: raw[10],
        act_be: raw[11],
    };
    (a, b)
}

fn federated_config(
    broker_id: &str,
    ports: &BrokerPorts,
    msg_peer: Option<MessagePeer>,
    svc_peer: Option<ServicePeer>,
    act_peer: Option<ActionPeer>,
) -> RobotBusConfig {
    RobotBusConfig {
        message: BusConfig {
            xsub_bind: format!("tcp://127.0.0.1:{}", ports.xsub),
            xpub_bind: format!("tcp://127.0.0.1:{}", ports.xpub),
            snd_hwm: 2_048,
            rcv_hwm: 2_048,
            bind_all_transports: false,
            bind_opts: Default::default(),
            broker_id: broker_id.to_string(),
            peers: msg_peer.into_iter().collect(),
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", ports.svc_fe),
            backend_bind: format!("tcp://127.0.0.1:{}", ports.svc_be),
            snd_hwm: 64,
            rcv_hwm: 64,
            bind_all_transports: false,
            bind_opts: Default::default(),
            broker_id: broker_id.to_string(),
            peers: svc_peer.into_iter().collect(),
            heartbeat_interval_ms: 200,
            heartbeat_timeout_ms: 2_000,
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{}", ports.act_fe),
            backend_bind: format!("tcp://127.0.0.1:{}", ports.act_be),
            snd_hwm: 64,
            rcv_hwm: 64,
            bind_all_transports: false,
            bind_opts: Default::default(),
            broker_id: broker_id.to_string(),
            peers: act_peer.into_iter().collect(),
            heartbeat_interval_ms: 200,
            heartbeat_timeout_ms: 2_000,
            pending_timeout_ms: 3_000,
            ..ActionBusConfig::default()
        },
        #[cfg(feature = "ws")]
        ws: WsConfig {
            listen: "127.0.0.1:0".parse().expect("ws listen"),
            cors_origins: Vec::new(),
        },
        discovery: DiscoveryConfig {
            enabled: false,
            ..DiscoveryConfig::default()
        },
        #[cfg(feature = "console-api")]
        console: ConsoleBrokerConfig {
            enabled: false,
            tank_enabled: false,
            docs_enabled: true,
            listen: "127.0.0.1:0".parse().expect("console listen"),
            cors_origins: vec![],
        },
    }
}

fn options_for_broker(broker: &RobotBusBroker) -> NodeOptions {
    let mut opts = NodeOptions::tcp();
    opts.message_xsub = Some(connect_addr(&broker.message.xsub_bind));
    opts.message_xpub = Some(connect_addr(&broker.message.xpub_bind));
    opts.service_frontend = Some(connect_addr(&broker.service.frontend_bind));
    opts.service_backend = Some(connect_addr(&broker.service.backend_bind));
    opts.action_frontend = Some(connect_addr(&broker.action.frontend_bind));
    opts.action_backend = Some(connect_addr(&broker.action.backend_bind));
    opts
}

fn node_on(name: impl Into<String>, broker: &RobotBusBroker) -> Node {
    Node::with_options(name, options_for_broker(broker))
}

/// Bidirectional A↔B federation; benches send A → B (pub/client on A, sub/server on B).
pub fn bench_federation(skip_rpc: bool) -> Vec<ScenarioResult> {
    let (ports_a, ports_b) = alloc_ports();
    let cfg_a = federated_config(
        "perf-fed-a",
        &ports_a,
        Some(MessagePeer {
            xpub: format!("tcp://127.0.0.1:{}", ports_b.xpub),
            xsub: format!("tcp://127.0.0.1:{}", ports_b.xsub),
        }),
        Some(ServicePeer {
            backend: format!("tcp://127.0.0.1:{}", ports_b.svc_be),
            broker_id: "perf-fed-b".into(),
        }),
        Some(ActionPeer {
            backend: format!("tcp://127.0.0.1:{}", ports_b.act_be),
            broker_id: "perf-fed-b".into(),
        }),
    );
    let cfg_b = federated_config(
        "perf-fed-b",
        &ports_b,
        Some(MessagePeer {
            xpub: format!("tcp://127.0.0.1:{}", ports_a.xpub),
            xsub: format!("tcp://127.0.0.1:{}", ports_a.xsub),
        }),
        Some(ServicePeer {
            backend: format!("tcp://127.0.0.1:{}", ports_a.svc_be),
            broker_id: "perf-fed-a".into(),
        }),
        Some(ActionPeer {
            backend: format!("tcp://127.0.0.1:{}", ports_a.act_be),
            broker_id: "perf-fed-a".into(),
        }),
    );

    println!("starting federated RobotBusBroker pair (tcp A↔B)…");
    let broker_a = match RobotBusBroker::start(cfg_a) {
        Ok(b) => b,
        Err(err) => {
            return vec![ScenarioResult::skipped(
                TRANSPORT,
                "message pub/sub",
                format!("start broker a: {err}"),
            )];
        }
    };
    let broker_b = match RobotBusBroker::start(cfg_b) {
        Ok(b) => b,
        Err(err) => {
            let _ = broker_a.stop();
            return vec![ScenarioResult::skipped(
                TRANSPORT,
                "message pub/sub",
                format!("start broker b: {err}"),
            )];
        }
    };
    thread::sleep(Duration::from_millis(200));

    let mut results = Vec::new();
    results.push(bench_fed_pubsub(&broker_a, &broker_b));
    if !skip_rpc {
        results.push(bench_fed_service(&broker_a, &broker_b, svc_iters()));
        results.push(bench_fed_action(&broker_a, &broker_b, act_iters()));
    }

    if let Err(err) = broker_a.stop() {
        eprintln!("stop broker a: {err}");
    }
    if let Err(err) = broker_b.stop() {
        eprintln!("stop broker b: {err}");
    }
    results
}

fn bench_fed_pubsub(broker_a: &RobotBusBroker, broker_b: &RobotBusBroker) -> ScenarioResult {
    let scenario = "message pub/sub";
    let sub = node_on("perf-fed-sub", broker_b);
    if !sub.wait_for_broker(Some(Duration::from_secs(5))) {
        return ScenarioResult::skipped(TRANSPORT, scenario, "wait_for_broker timed out");
    }
    let hwm = HighWaterMark {
        snd: MSG_HWM,
        rcv: MSG_HWM,
    };
    let xsub = connect_addr(&broker_a.message.xsub_bind);
    let publisher = match Publisher::with_hwm(Some(&xsub), hwm) {
        Ok(p) => p,
        Err(err) => {
            return ScenarioResult::skipped(TRANSPORT, scenario, format!("publisher: {err}"));
        }
    };
    // XPUB demand must reach the publisher's broker via the peer SUB.
    run_pubsub(
        TRANSPORT,
        scenario,
        sub,
        publisher,
        Duration::from_millis(500),
    )
}

fn bench_fed_service(
    broker_a: &RobotBusBroker,
    broker_b: &RobotBusBroker,
    n: usize,
) -> ScenarioResult {
    let scenario = "service call";
    let name = "perf.federation.echo";
    let worker = match WorkerThread::spawn_service(
        name,
        Arc::new(|body| body.to_vec()),
        &connect_addr(&broker_b.service.backend_bind),
    ) {
        Ok(w) => w,
        Err(err) => {
            return ScenarioResult::skipped(TRANSPORT, scenario, format!("worker: {err}"));
        }
    };
    thread::sleep(Duration::from_millis(500));
    let client = match ServiceClient::new(Some(&connect_addr(&broker_a.service.frontend_bind))) {
        Ok(c) => c,
        Err(err) => {
            worker.stop();
            return ScenarioResult::skipped(TRANSPORT, scenario, format!("client: {err}"));
        }
    };
    let payload = make_payload(0);
    let mut ready = false;
    for _ in 0..8 {
        if client
            .call(name, &payload, None, Some(Duration::from_secs(2)))
            .is_ok()
        {
            ready = true;
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }
    if !ready {
        worker.stop();
        return ScenarioResult::skipped(TRANSPORT, scenario, "federated service not ready");
    }
    for _ in 1..WARMUP {
        let _ = client.call(name, &payload, None, Some(Duration::from_secs(2)));
    }

    let mut samples = Vec::with_capacity(n);
    let t0 = Instant::now();
    let mut received = 0usize;
    for _ in 0..n {
        let start = Instant::now();
        match client.call(name, &payload, None, Some(Duration::from_secs(5))) {
            Ok(_) => {
                samples.push(start.elapsed().as_nanos() as u64);
                received += 1;
            }
            Err(err) => {
                if received == 0 {
                    worker.stop();
                    return ScenarioResult::skipped(
                        TRANSPORT,
                        scenario,
                        format!("call failed: {err}"),
                    );
                }
                break;
            }
        }
    }
    let elapsed = t0.elapsed();
    worker.stop();
    if received == 0 {
        return ScenarioResult::skipped(TRANSPORT, scenario, "0 successful calls");
    }
    ScenarioResult::ok_rpc(
        TRANSPORT,
        scenario,
        n,
        received,
        elapsed,
        LatencyStats::from_ns(samples),
    )
}

fn bench_fed_action(
    broker_a: &RobotBusBroker,
    broker_b: &RobotBusBroker,
    n: usize,
) -> ScenarioResult {
    let scenario = "action send_goal";
    let name = "perf.federation.act";
    let worker = match WorkerThread::spawn_action(
        name,
        Arc::new(|body| {
            vec![
                ("FEEDBACK".into(), b"f".to_vec()),
                ("RESULT".into(), body.to_vec()),
            ]
        }),
        &connect_addr(&broker_b.action.backend_bind),
    ) {
        Ok(w) => w,
        Err(err) => {
            return ScenarioResult::skipped(TRANSPORT, scenario, format!("worker: {err}"));
        }
    };
    thread::sleep(Duration::from_millis(500));
    let client = match ActionClient::new(Some(&connect_addr(&broker_a.action.frontend_bind))) {
        Ok(c) => c,
        Err(err) => {
            worker.stop();
            return ScenarioResult::skipped(TRANSPORT, scenario, format!("client: {err}"));
        }
    };
    let payload = make_payload(0);
    let mut ready = false;
    for _ in 0..8 {
        if client
            .send_goal(name, &payload, None, Some(Duration::from_secs(2)))
            .is_ok()
        {
            ready = true;
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }
    if !ready {
        worker.stop();
        return ScenarioResult::skipped(TRANSPORT, scenario, "federated action not ready");
    }
    for _ in 1..WARMUP.min(10) {
        let _ = client.send_goal(name, &payload, None, Some(Duration::from_secs(2)));
    }

    let mut samples = Vec::with_capacity(n);
    let t0 = Instant::now();
    let mut received = 0usize;
    for _ in 0..n {
        let start = Instant::now();
        match client.send_goal(name, &payload, None, Some(Duration::from_secs(5))) {
            Ok(_) => {
                samples.push(start.elapsed().as_nanos() as u64);
                received += 1;
            }
            Err(err) => {
                if received == 0 {
                    worker.stop();
                    return ScenarioResult::skipped(
                        TRANSPORT,
                        scenario,
                        format!("send_goal failed: {err}"),
                    );
                }
                break;
            }
        }
    }
    let elapsed = t0.elapsed();
    worker.stop();
    if received == 0 {
        return ScenarioResult::skipped(TRANSPORT, scenario, "0 successful goals");
    }
    ScenarioResult::ok_rpc(
        TRANSPORT,
        scenario,
        n,
        received,
        elapsed,
        LatencyStats::from_ns(samples),
    )
}
