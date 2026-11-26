use std::net::{SocketAddr, TcpListener};
use std::time::Duration;

use robot_bus::discovery::fetch_discover;
use robot_bus::{RobotBusBroker, RobotBusConfig};

fn consecutive_listeners() -> (TcpListener, TcpListener) {
    for _ in 0..32 {
        let first = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut next = first.local_addr().unwrap();
        if next.port() > u16::MAX - 2 {
            continue;
        }
        next.set_port(next.port() + 1);
        if let Ok(second) = TcpListener::bind(next) {
            next.set_port(next.port() + 1);
            if TcpListener::bind(next).is_ok() {
                return (first, second);
            }
        }
    }
    panic!("could not find three consecutive free loopback ports");
}

fn config(listen: SocketAddr, console: bool) -> RobotBusConfig {
    let mut config = crate::common::ephemeral_robot_bus_config();
    config.discovery.advertise_host = Some("127.0.0.1".into());
    #[cfg(feature = "ws")]
    {
        config.ws.listen = listen;
    }
    #[cfg(feature = "console-api")]
    {
        config.console.enabled = console;
        config.console.listen = listen;
    }
    #[cfg(not(feature = "console-api"))]
    let _ = console;
    config
}

fn assert_api(broker: &RobotBusBroker, listen: SocketAddr) {
    let base = format!("http://{listen}");
    #[cfg(feature = "ws")]
    assert_eq!(broker.api_listen(), listen);
    #[cfg(not(feature = "ws"))]
    assert_eq!(broker.console_listen(), Some(listen));
    let response = fetch_discover(&base, Duration::from_secs(3)).unwrap();
    assert_eq!(response, broker.discover);
    assert!(response.message_xpub_port > 0);
    #[cfg(feature = "ws")]
    assert_eq!(broker.api_url(), base);
}

#[test]
fn busy_api_ports_are_skipped_in_order() {
    let (first, _second) = consecutive_listeners();
    let requested = first.local_addr().unwrap();
    let broker = RobotBusBroker::start(config(requested, true)).unwrap();
    let mut expected = requested;
    expected.set_port(requested.port() + 2);
    assert_api(&broker, expected);
    #[cfg(feature = "ws")]
    {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let url = format!("ws://{expected}/ws-rpc");
            let (mut socket, _) = tokio_tungstenite::connect_async(url).await.unwrap();
            socket.close(None).await.unwrap();
        });
    }
    broker.stop().unwrap();
}

#[test]
fn discovery_works_on_ephemeral_api_port() {
    let broker = RobotBusBroker::start(config("127.0.0.1:0".parse().unwrap(), true)).unwrap();
    #[cfg(feature = "ws")]
    let actual = broker.api_listen();
    #[cfg(not(feature = "ws"))]
    let actual = broker.console_listen().unwrap();
    assert_ne!(actual.port(), 0);
    assert_api(&broker, actual);
    broker.stop().unwrap();
}

#[cfg(feature = "ws")]
#[test]
fn ws_discovery_keeps_the_requested_api_origin() {
    use robot_bus::{Node, NodeOptions, discovery::DiscoverOpts};

    let broker = RobotBusBroker::start(config("0.0.0.0:0".parse().unwrap(), false)).unwrap();
    let api_url = format!("http://localhost:{}", broker.api_listen().port());
    assert_ne!(broker.api_url(), api_url);

    let opts = NodeOptions::for_discovery("ws")
        .discover(DiscoverOpts::at(&api_url))
        .unwrap();
    assert_eq!(opts.ws_url.as_deref(), Some(api_url.as_str()));
    assert_eq!(opts.console_url.as_deref(), Some(api_url.as_str()));
    let node = Node::with_options("ws_discover_origin", opts);
    assert!(node.wait_for_broker(Some(Duration::from_secs(5))));
    drop(node);
    broker.stop().unwrap();
}
