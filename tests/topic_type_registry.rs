//! Topic type registry: typed create_publisher registers before any traffic.

#![cfg(feature = "console")]

mod support;

use std::thread;
use std::time::Duration;

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
use robot_bus::{console_topics, robot_bus_interface::msg::v1::TopicTypeRegister};
use serde::Deserialize;
use support::lock_brokers;

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
