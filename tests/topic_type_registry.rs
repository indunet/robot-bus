//! Topic type registry: typed create_publisher registers before any traffic.

#![cfg(feature = "console")]

mod support;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use prost::{Message, Name};
use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::broker::{
    ConsoleBrokerConfig, DiscoveryConfig, WsGatewayConfig, RobotBusBroker, RobotBusConfig,
};
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::service_bus::ServiceClient;
use robot_bus::std_msgs::msg::v1::String as BusString;
use robot_bus::{Node, NodeOptions};
use robot_bus::{console_topics, robot_bus_interfaces::msg::v1::TopicTypeRegister};
use serde::Deserialize;
use support::{ephemeral_robot_bus_config, lock_brokers};

fn test_broker_config(
    msg_xsub: u16,
    msg_xpub: u16,
    svc_fe: u16,
    svc_be: u16,
    act_fe: u16,
    act_be: u16,
    http: u16,
) -> RobotBusConfig {
    RobotBusConfig {
        message: BusConfig {
            xsub_bind: format!("tcp://127.0.0.1:{msg_xsub}"),
            xpub_bind: format!("tcp://127.0.0.1:{msg_xpub}"),
            bind_all_transports: false,
            bind_opts: Default::default(),
            ..BusConfig::default()
        },
        service: ServiceBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{svc_fe}"),
            backend_bind: format!("tcp://127.0.0.1:{svc_be}"),
            bind_all_transports: false,
            bind_opts: Default::default(),
            ..ServiceBusConfig::default()
        },
        action: ActionBusConfig {
            frontend_bind: format!("tcp://127.0.0.1:{act_fe}"),
            backend_bind: format!("tcp://127.0.0.1:{act_be}"),
            bind_all_transports: false,
            bind_opts: Default::default(),
            ..ActionBusConfig::default()
        },
        discovery: DiscoveryConfig {
            enabled: false,
            ..DiscoveryConfig::default()
        },
        ws: WsGatewayConfig {
            listen: format!("127.0.0.1:{http}").parse().unwrap(),
            ..WsGatewayConfig::default()
        },
        console: ConsoleBrokerConfig {
            enabled: true,
            tank_enabled: false,
            docs_enabled: true,
            listen: format!("127.0.0.1:{http}").parse().unwrap(),
            cors_origins: vec![],
        },
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TopicRow {
    name: String,
    type_name: Option<String>,
    #[serde(default)]
    total_msgs: u64,
}

#[derive(Debug, Deserialize)]
struct TopicsEnvelope {
    topics: Vec<TopicRow>,
}

fn get_topics(console_url: &str) -> TopicsEnvelope {
    let url = format!("{console_url}/api/v1/topics");
    ureq::get(&url)
        .call()
        .unwrap_or_else(|e| panic!("GET {url}: {e}"))
        .into_json()
        .expect("decode topics")
}

fn get_topic_info(console_url: &str, topic: &str) -> TopicRow {
    let encoded: String = topic
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect();
    let url = format!("{console_url}/api/v1/topics/{encoded}");
    ureq::get(&url)
        .call()
        .unwrap_or_else(|e| panic!("GET {url}: {e}"))
        .into_json()
        .expect("decode topic info")
}

#[test]
fn typed_publisher_registers_type_before_traffic() {
    let _guard = lock_brokers();
    let http_port = 28770u16;
    let broker = RobotBusBroker::start(test_broker_config(
        28560, 28561, 28662, 28663, 28664, 28665, http_port,
    ))
    .expect("start broker");
    thread::sleep(Duration::from_millis(200));

    let console_url = format!("http://127.0.0.1:{http_port}");
    let opts = NodeOptions {
        message_xsub: Some("tcp://127.0.0.1:28560".into()),
        message_xpub: Some("tcp://127.0.0.1:28561".into()),
        service_frontend: Some("tcp://127.0.0.1:28662".into()),
        ..NodeOptions::default()
    };
    let mut node = Node::with_options("type_reg", opts);
    let _pub = node
        .create_publisher::<Imu>("/robot1/imu")
        .expect("create_publisher");

    // Bus control-plane register + console settle.
    thread::sleep(Duration::from_millis(500));

    let list = get_topics(&console_url);
    let row = list
        .topics
        .iter()
        .find(|t| t.name == "/robot1/imu")
        .unwrap_or_else(|| panic!("missing /robot1/imu in {list:?}"));
    assert_eq!(row.type_name.as_deref(), Some(Imu::full_name().as_str()));
    assert_eq!(row.total_msgs, 0, "no traffic yet");

    let info = get_topic_info(&console_url, "/robot1/imu");
    assert_eq!(info.name, "/robot1/imu");
    assert_eq!(info.type_name.as_deref(), Some("sensor_msgs.msg.v1.Imu"));

    drop(_pub);
    drop(node);
    broker.stop().expect("stop");
}

#[test]
fn type_register_last_write_wins() {
    let _guard = lock_brokers();
    let http_port = 28870u16;
    let broker = RobotBusBroker::start(test_broker_config(
        28860, 28861, 28862, 28863, 28864, 28865, http_port,
    ))
    .expect("start broker");
    thread::sleep(Duration::from_millis(200));

    let console_url = format!("http://127.0.0.1:{http_port}");
    let client = ServiceClient::new(Some("tcp://127.0.0.1:28862")).expect("control client");
    for type_name in ["sensor_msgs.msg.v1.Imu".to_string(), BusString::full_name()] {
        let payload = TopicTypeRegister {
            topic: "/conflict".into(),
            type_name,
        }
        .encode_to_vec();
        client
            .call(
                console_topics::TOPIC_TYPE_REGISTER,
                &payload,
                None,
                Some(Duration::from_secs(3)),
            )
            .expect("register type");
    }

    let info = get_topic_info(&console_url, "/conflict");
    assert_eq!(
        info.type_name.as_deref(),
        Some(BusString::full_name().as_str())
    );

    broker.stop().expect("stop");
}

#[derive(Debug, Deserialize)]
struct TopologyEnvelope {
    edges: Vec<TopologyEdgeRow>,
}

#[derive(Debug, Deserialize)]
struct TopologyEdgeRow {
    kind: String,
    topic: String,
}

fn console_enabled_config() -> RobotBusConfig {
    let mut config = ephemeral_robot_bus_config();
    config.console.enabled = true;
    config.console.tank_enabled = false;
    config
}

fn snapshot_restart_config(broker: &RobotBusBroker) -> RobotBusConfig {
    let mut config = console_enabled_config();
    config.message.xsub_bind = broker.message.xsub_bind.clone();
    config.message.xpub_bind = broker.message.xpub_bind.clone();
    config.service.frontend_bind = broker.service.frontend_bind.clone();
    config.service.backend_bind = broker.service.backend_bind.clone();
    config.action.frontend_bind = broker.action.frontend_bind.clone();
    config.action.backend_bind = broker.action.backend_bind.clone();
    #[cfg(feature = "ws")]
    {
        config.ws.listen = broker.api_listen();
        config.console.listen = broker.api_listen();
    }
    config
}

fn try_get_topics(console_url: &str) -> Option<TopicsEnvelope> {
    ureq::get(&format!("{console_url}/api/v1/topics"))
        .call()
        .ok()?
        .into_json()
        .ok()
}

fn wait_topic_type(console_url: &str, topic: &str, expected: &str, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    let mut last = String::from("(no response)");
    loop {
        if let Some(list) = try_get_topics(console_url) {
            last = format!("{list:?}");
            if let Some(row) = list.topics.iter().find(|t| t.name == topic) {
                if row.type_name.as_deref() == Some(expected) {
                    return;
                }
            }
        }
        if Instant::now() >= deadline {
            panic!("timed out waiting for {topic} type {expected}; last={last}");
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn wait_topology_edge(console_url: &str, kind: &str, topic: &str, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    loop {
        let url = format!("{console_url}/api/v1/topology");
        if let Ok(resp) = ureq::get(&url).call() {
            if let Ok(body) = resp.into_json::<TopologyEnvelope>() {
                if body
                    .edges
                    .iter()
                    .any(|e| e.kind == kind && e.topic == topic)
                {
                    return;
                }
            }
        }
        if Instant::now() >= deadline {
            panic!("timed out waiting for {kind} edge on {topic}");
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn wait_topology_publisher(console_url: &str, topic: &str, timeout: Duration) {
    wait_topology_edge(console_url, "publisher", topic, timeout);
}

#[test]
fn typed_publisher_restores_metadata_after_broker_restart() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(console_enabled_config()).expect("start broker");
    let restart_config = snapshot_restart_config(&broker);
    let console_url = broker.api_url();
    thread::sleep(Duration::from_millis(200));

    let mut opts = NodeOptions::tcp();
    opts.console_url = Some(console_url.clone());
    opts.message_xsub = Some(broker.message.xsub_bind.clone());
    opts.message_xpub = Some(broker.message.xpub_bind.clone());
    opts.service_frontend = Some(broker.service.frontend_bind.clone());
    let mut node = Node::with_options("type_restore", opts);
    assert!(
        node.wait_for_broker(Some(Duration::from_secs(5))),
        "state={}",
        node.connection_state()
    );
    let _pub = node
        .create_publisher::<Imu>("/restore/imu")
        .expect("create_publisher");
    thread::sleep(Duration::from_millis(500));

    wait_topic_type(
        &console_url,
        "/restore/imu",
        Imu::full_name().as_str(),
        Duration::from_secs(5),
    );
    wait_topology_publisher(&console_url, "/restore/imu", Duration::from_secs(5));

    broker.stop().expect("stop");
    thread::sleep(Duration::from_millis(400));

    let broker = RobotBusBroker::start(restart_config).expect("restart broker");
    assert!(
        node.wait_for_broker(Some(Duration::from_secs(8))),
        "did not reconnect, state={}",
        node.connection_state()
    );

    wait_topic_type(
        &console_url,
        "/restore/imu",
        Imu::full_name().as_str(),
        Duration::from_secs(8),
    );
    wait_topology_publisher(&console_url, "/restore/imu", Duration::from_secs(8));

    drop(_pub);
    drop(node);
    broker.stop().expect("stop");
}

fn tcp_node_on_broker(name: &str, broker: &RobotBusBroker, console_url: &str) -> Node {
    let mut opts = NodeOptions::tcp();
    opts.console_url = Some(console_url.to_string());
    opts.message_xsub = Some(broker.message.xsub_bind.clone());
    opts.message_xpub = Some(broker.message.xpub_bind.clone());
    opts.service_frontend = Some(broker.service.frontend_bind.clone());
    let node = Node::with_options(name, opts);
    assert!(
        node.wait_for_broker(Some(Duration::from_secs(5))),
        "state={}",
        node.connection_state()
    );
    node
}

#[test]
fn typed_subscription_restores_metadata_after_broker_restart() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(console_enabled_config()).expect("start broker");
    let restart_config = snapshot_restart_config(&broker);
    let console_url = broker.api_url();
    thread::sleep(Duration::from_millis(200));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = Arc::clone(&hits);
    let mut node = tcp_node_on_broker("type_restore_sub", &broker, &console_url);
    node.create_subscription::<Imu, _>(
        "/restore/sub",
        move |_topic, _imu| {
            hits_cb.fetch_add(1, Ordering::SeqCst);
        },
        None,
    )
    .expect("create_subscription");
    thread::sleep(Duration::from_millis(500));

    wait_topic_type(
        &console_url,
        "/restore/sub",
        Imu::full_name().as_str(),
        Duration::from_secs(5),
    );
    wait_topology_edge(
        &console_url,
        "subscriber",
        "/restore/sub",
        Duration::from_secs(5),
    );

    broker.stop().expect("stop");
    thread::sleep(Duration::from_millis(400));

    let broker = RobotBusBroker::start(restart_config).expect("restart broker");
    assert!(
        node.wait_for_broker(Some(Duration::from_secs(8))),
        "did not reconnect, state={}",
        node.connection_state()
    );

    wait_topic_type(
        &console_url,
        "/restore/sub",
        Imu::full_name().as_str(),
        Duration::from_secs(8),
    );
    wait_topology_edge(
        &console_url,
        "subscriber",
        "/restore/sub",
        Duration::from_secs(8),
    );

    let pub_ = robot_bus::Publisher::new(Some(&broker.message.xsub_bind)).expect("publisher");
    let payload = Imu::default().encode_to_vec();
    let deadline = Instant::now() + Duration::from_secs(5);
    while hits.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
        let _ = pub_.publish("/restore/sub", &payload);
        let _ = node.spin_once(Some(Duration::from_millis(50)));
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        hits.load(Ordering::SeqCst) >= 1,
        "subscription should receive after broker restart"
    );

    drop(node);
    broker.stop().expect("stop");
}
