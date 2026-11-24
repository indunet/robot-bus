use std::net::{SocketAddr, TcpListener};
use std::time::Duration;

use robot_bus::{discovery::fetch_discover};
use robot_bus_broker::{RobotBusBroker, RobotBusConfig};

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
    assert_eq!(response.broker_id, broker.discover.broker_id);
    assert_eq!(response.api_url, base);
    assert_eq!(broker.discover.api_url, base);
    if let Some(console_url) = response.console_url {
        assert_eq!(console_url, base);
    }
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
fn ephemeral_api_port_is_advertised_by_discovery() {
    let broker = RobotBusBroker::start(config("127.0.0.1:0".parse().unwrap(), true)).unwrap();
    #[cfg(feature = "ws")]
    let actual = broker.api_listen();
    #[cfg(not(feature = "ws"))]
    let actual = broker.console_listen().unwrap();
    assert_ne!(actual.port(), 0);
    assert_api(&broker, actual);
    broker.stop().unwrap();
}
