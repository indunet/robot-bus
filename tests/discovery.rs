//! UDP discovery: wait for broker announce and connect via applied NodeOptions.

mod support;

use robot_bus::discovery::{DEFAULT_MULTICAST_ADDR, DiscoverOpts};
use robot_bus::message_bus::{Publisher, Subscriber};
use robot_bus::robot_bus_interface::msg::v1::TcpPorts;
use robot_bus::{Node, NodeOptions, RobotBusBroker, RobotBusConfig, discovery};
use std::thread;
use std::time::Duration;
use support::lock_brokers;

fn unique_discovery_port() -> u16 {
    let base = 45550u16;
    let tid = std::thread::current().id();
    let hash = format!("{tid:?}");
    let n: u16 = hash.bytes().map(|b| b as u16).sum();
    base.wrapping_add(n % 1000)
}

#[test]
fn discover_tcp_then_pubsub() {
    let _guard = lock_brokers();
    let disco_port = unique_discovery_port();
    let offset = (disco_port % 200) as u16;

    let mut config = RobotBusConfig::default();
    config.discovery.enabled = true;
    config.discovery.domain_id = 7;
    config.discovery.multicast_addr = DEFAULT_MULTICAST_ADDR;
    config.discovery.multicast_port = disco_port;
    config.discovery.advertise_host = Some("127.0.0.1".into());
    config.discovery.interval = Duration::from_millis(100);

    config.message.xsub_bind = format!("tcp://127.0.0.1:{}", 25560 + offset);
    config.message.xpub_bind = format!("tcp://127.0.0.1:{}", 25561 + offset);
    config.service.frontend_bind = format!("tcp://127.0.0.1:{}", 25662 + offset);
    config.service.backend_bind = format!("tcp://127.0.0.1:{}", 25663 + offset);
    config.action.frontend_bind = format!("tcp://127.0.0.1:{}", 25664 + offset);
    config.action.backend_bind = format!("tcp://127.0.0.1:{}", 25665 + offset);
    config.message.broker_id = "disco-test-broker".into();
    config.service.broker_id = "disco-test-broker".into();
    config.action.broker_id = "disco-test-broker".into();
    #[cfg(feature = "console")]
    {
        config.console.enabled = false;
    }
    #[cfg(feature = "grpc")]
    {
        config.grpc.listen = format!("127.0.0.1:{}", 25770 + offset).parse().unwrap();
    }

    let broker = RobotBusBroker::start(config).expect("start broker");
    thread::sleep(Duration::from_millis(200));

    let opts = DiscoverOpts {
        domain_id: 7,
        broker_id: Some("disco-test-broker".into()),
        multicast_addr: DEFAULT_MULTICAST_ADDR,
        multicast_port: disco_port,
        timeout: Duration::from_secs(3),
    };
    let node_opts = NodeOptions::tcp()
        .discover(opts)
        .expect("discover + apply tcp");
    assert_eq!(node_opts.host, "127.0.0.1");
    let xpub = node_opts.message_xpub_endpoint().unwrap();
    let xsub = node_opts.message_xsub_endpoint().unwrap();
    assert!(xpub.contains(&format!(":{}", 25561 + offset)));

    let sub = Subscriber::new(Some(&xpub)).expect("sub");
    sub.subscribe("disco/topic").expect("subscribe");
    let pub_ = Publisher::new(Some(&xsub)).expect("pub");
    thread::sleep(Duration::from_millis(150));
    pub_.publish("disco/topic", b"hello-disco")
        .expect("publish");

    let (topic, payload) = sub.receive(Some(Duration::from_secs(2))).expect("receive");
    assert_eq!(topic, "disco/topic");
    assert_eq!(payload, b"hello-disco");

    let _node = Node::with_options("disco_node", node_opts);
    broker.stop().expect("stop broker");
}

#[test]
fn apply_ipc_fails_when_tcp_only_announce() {
    let ann = discovery::BrokerAnnouncement {
        broker_id: "b".into(),
        domain_id: 0,
        advertise_host: "127.0.0.1".into(),
        tcp: Some(TcpPorts {
            message_xsub: 1,
            message_xpub: 2,
            service_frontend: 3,
            service_backend: 4,
            action_frontend: 5,
            action_backend: 6,
        }),
        ipc_dir: None,
        inproc_prefix: None,
        grpc_url: None,
        console_url: None,
    };
    let bytes = discovery::encode_announce(&ann).unwrap();
    assert!(!bytes.is_empty());
    let err = ann.apply(NodeOptions::ipc()).unwrap_err();
    assert!(err.to_string().contains("ipc_dir"));
}
