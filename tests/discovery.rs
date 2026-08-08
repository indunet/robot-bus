//! HTTP discovery: wait for broker `/api/v1/discover` and connect via applied NodeOptions.

#[cfg(feature = "grpc")]
mod support;

use robot_bus::robot_bus_interface::msg::v1::TcpPorts;
use robot_bus::{NodeOptions, discovery};

#[cfg(feature = "grpc")]
use robot_bus::discovery::DiscoverOpts;
#[cfg(feature = "grpc")]
use robot_bus::message_bus::{Publisher, Subscriber};
#[cfg(feature = "grpc")]
use robot_bus::{Node, RobotBusBroker, RobotBusConfig};
#[cfg(feature = "grpc")]
use std::thread;
#[cfg(feature = "grpc")]
use std::time::Duration;
#[cfg(feature = "grpc")]
use support::{free_ports, lock_brokers};

/// HTTP discover needs the gRPC gateway listener (`GET /api/v1/discover`).
/// Skip under `cargo test --no-default-features` where no API server is started.
#[cfg(feature = "grpc")]
#[test]
fn discover_tcp_then_pubsub() {
    let _guard = lock_brokers();
    let ports = free_ports(1);
    let api_port = ports[0];

    let mut config = RobotBusConfig::default();
    config.discovery.advertise_host = Some("127.0.0.1".into());
    config.discovery.domain_id = 7;
    config.message.broker_id = "disco-test-broker".into();
    config.service.broker_id = "disco-test-broker".into();
    config.action.broker_id = "disco-test-broker".into();
    config.message.bind_all_transports = false;
    config.service.bind_all_transports = false;
    config.action.bind_all_transports = false;
    #[cfg(feature = "console")]
    {
        // Discover is served from the gateway when console UI is off.
        config.console.enabled = false;
    }
    config.grpc.listen = format!("127.0.0.1:{api_port}").parse().unwrap();

    let broker = RobotBusBroker::start(config).expect("start broker");
    thread::sleep(Duration::from_millis(300));

    let api_url = format!("http://127.0.0.1:{api_port}");
    let opts = DiscoverOpts {
        api_url: api_url.clone(),
        broker_id: Some("disco-test-broker".into()),
        timeout: Duration::from_secs(3),
    };
    let node_opts = NodeOptions::tcp()
        .discover(opts)
        .expect("discover + apply tcp");
    assert_eq!(node_opts.host, "127.0.0.1");
    let xpub = node_opts.message_xpub_endpoint().unwrap();
    let xsub = node_opts.message_xsub_endpoint().unwrap();
    assert_eq!(xsub, broker.discover.message_xsub);
    assert_eq!(xpub, broker.discover.message_xpub);
    assert!(xpub.starts_with("tcp://127.0.0.1:"));
    assert!(xsub.starts_with("tcp://127.0.0.1:"));
    assert_ne!(xpub, xsub);

    let sub = Subscriber::new(Some(&xpub)).expect("sub");
    sub.subscribe("disco/topic").expect("subscribe");
    let pub_ = Publisher::new(Some(&xsub)).expect("pub");
    // Slow-joiner: XPUB subscription must propagate before the first publish.
    thread::sleep(Duration::from_millis(400));
    pub_.publish("disco/topic", b"hello-disco")
        .expect("publish");

    let (topic, payload) = sub.receive(Some(Duration::from_secs(3))).expect("receive");
    assert_eq!(topic, "disco/topic");
    assert_eq!(payload, b"hello-disco");

    let disc = &broker.discover;
    assert_eq!(disc.broker_id, "disco-test-broker");
    assert_eq!(disc.api_url, api_url);
    assert_ne!(disc.message_xsub, disc.message_xpub);

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
    let err = ann.apply(NodeOptions::ipc()).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("ipc_dir"), "{msg}");
}
