//! Broker↔broker federation over `ipc://` endpoints (same host).

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::common::{ephemeral_robot_bus_config, lock_brokers};
use robot_bus::action_bus::{ActionClient, ActionKind};
use robot_bus::broker::FederationPeerEndpoints;
use robot_bus::broker::action_bus::ActionPeer;
use robot_bus::broker::message_bus::MessagePeer;
use robot_bus::broker::service_bus::ServicePeer;
use robot_bus::service_bus::ServiceClient;
use robot_bus::transports::{
    ACTION_BACKEND_CHANNEL, SERVICE_BACKEND_CHANNEL, XPUB_CHANNEL, ipc_endpoint_in, ipc_supported,
};
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{Publisher, Subscriber};
use robot_bus::{RobotBusBroker, RobotBusConfig};

fn ipc_broker_config(broker_id: &str, ipc_dir: &str) -> RobotBusConfig {
    let mut config = ephemeral_robot_bus_config();
    config.message.broker_id = broker_id.into();
    config.service.broker_id = broker_id.into();
    config.action.broker_id = broker_id.into();
    config.message.bind_all_transports = true;
    config.service.bind_all_transports = true;
    config.action.bind_all_transports = true;
    config.discovery.ipc_dir = Some(ipc_dir.into());
    config
}

fn recv_exact(sub: &Subscriber, expect: &[u8], timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok((_, payload)) = sub.receive(Some(Duration::from_millis(50))) {
            if payload == expect {
                return;
            }
        }
    }
    panic!("timed out waiting for payload {expect:?}");
}

fn call_service(frontend: &str, name: &str, body: &[u8]) -> Vec<u8> {
    ServiceClient::new(Some(frontend))
        .expect("service client")
        .call(name, body, None, Some(Duration::from_secs(5)))
        .unwrap_or_else(|e| panic!("call {name}: {e}"))
}

fn goal_result(frontend: &str, name: &str, body: &[u8]) -> Vec<u8> {
    let msgs = ActionClient::new(Some(frontend))
        .expect("action client")
        .send_goal(name, body, None, Some(Duration::from_secs(5)))
        .unwrap_or_else(|e| panic!("goal {name}: {e}"));
    assert!(msgs.iter().any(|m| m.kind == ActionKind::Feedback));
    msgs.into_iter()
        .find(|m| m.kind == ActionKind::Result)
        .expect("result")
        .body
}

/// A and B link only through each other's IPC sockets; clients stay on TCP.
#[test]
fn bidirectional_federation_over_ipc() {
    if !ipc_supported() {
        eprintln!("skipping: libzmq has no ipc transport here");
        return;
    }
    let _guard = lock_brokers();
    let dir_a = tempfile::tempdir().expect("ipc dir a");
    let dir_b = tempfile::tempdir().expect("ipc dir b");
    let dir_a = dir_a.path().to_str().unwrap();
    let dir_b = dir_b.path().to_str().unwrap();

    // B starts first, so it can only name A's sockets from A's fixed IPC dir.
    let mut config_b = ipc_broker_config("broker-b", dir_b);
    config_b
        .message
        .peers
        .push(MessagePeer::from_xpub(&ipc_endpoint_in(dir_a, XPUB_CHANNEL)).unwrap());
    config_b.service.peers.push(
        ServicePeer::from_backend(&format!(
            "broker-a={}",
            ipc_endpoint_in(dir_a, SERVICE_BACKEND_CHANNEL)
        ))
        .unwrap(),
    );
    config_b.action.peers.push(
        ActionPeer::from_backend(&format!(
            "broker-a={}",
            ipc_endpoint_in(dir_a, ACTION_BACKEND_CHANNEL)
        ))
        .unwrap(),
    );
    let broker_b = RobotBusBroker::start(config_b).expect("broker b");

    // A resolves B the way `--peer ipc+HOST:PORT` does after fetching discover.
    let peer_b = FederationPeerEndpoints::from_discover_ipc(
        &broker_b.discover,
        &robot_bus::discovery::DiscoverOpts::default().api_url,
    )
    .expect("ipc peer");
    assert_eq!(peer_b.message.xpub, ipc_endpoint_in(dir_b, XPUB_CHANNEL));
    let mut config_a = ipc_broker_config("broker-a", dir_a);
    config_a.message.peers.push(peer_b.message);
    config_a.service.peers.push(peer_b.service);
    config_a.action.peers.push(peer_b.action);
    let broker_a = RobotBusBroker::start(config_a).expect("broker a");

    thread::sleep(Duration::from_millis(200));

    let sub_a = Subscriber::new(Some(&broker_a.message.xpub_bind)).expect("sub a");
    sub_a.subscribe("fleet/pose").expect("subscribe a");
    let sub_b = Subscriber::new(Some(&broker_b.message.xpub_bind)).expect("sub b");
    sub_b.subscribe("fleet/pose").expect("subscribe b");
    thread::sleep(Duration::from_millis(300));
    let pub_a = Publisher::new(Some(&broker_a.message.xsub_bind)).expect("pub a");
    let pub_b = Publisher::new(Some(&broker_b.message.xsub_bind)).expect("pub b");
    thread::sleep(Duration::from_millis(100));
    pub_a.publish("fleet/pose", b"from-a").expect("publish a");
    recv_exact(&sub_b, b"from-a", Duration::from_secs(2));
    pub_b.publish("fleet/pose", b"from-b").expect("publish b");
    recv_exact(&sub_a, b"from-b", Duration::from_secs(2));

    let mut workers = Vec::new();
    for (tag, broker) in [("a", &broker_a), ("b", &broker_b)] {
        let prefix = format!("{tag}:").into_bytes();
        workers.push(
            WorkerThread::spawn_service(
                &format!("svc.{tag}"),
                Arc::new(move |body| [prefix.as_slice(), body].concat()),
                &broker.service.backend_bind,
            )
            .expect("service worker"),
        );
        let prefix = format!("done:{tag}:").into_bytes();
        workers.push(
            WorkerThread::spawn_action(
                &format!("act.{tag}"),
                Arc::new(move |body| {
                    vec![
                        ("FEEDBACK".into(), b"fb".to_vec()),
                        ("RESULT".into(), [prefix.as_slice(), body].concat()),
                    ]
                }),
                &broker.action.backend_bind,
            )
            .expect("action worker"),
        );
    }
    thread::sleep(Duration::from_millis(600));

    assert_eq!(
        call_service(&broker_b.service.frontend_bind, "svc.a", b"ping"),
        b"a:ping"
    );
    assert_eq!(
        call_service(&broker_a.service.frontend_bind, "svc.b", b"pong"),
        b"b:pong"
    );
    assert_eq!(
        goal_result(&broker_b.action.frontend_bind, "act.a", b"go"),
        b"done:a:go"
    );
    assert_eq!(
        goal_result(&broker_a.action.frontend_bind, "act.b", b"go"),
        b"done:b:go"
    );

    for worker in workers {
        worker.stop();
    }
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}
