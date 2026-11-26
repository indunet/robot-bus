//! One-sided service federation: only the joining broker lists `--peer`.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::common::lock_brokers;
use crate::service_bus_federation::{
    ServiceBrokerPorts, alloc_service_broker_ports, connect_addr, federated_service_config,
};
use robot_bus::broker::service_bus::ServicePeer;
use robot_bus::service_bus::ServiceClient;
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{RobotBusBroker, RobotBusConfig};

fn join_peer(broker_id: &str, ports: &ServiceBrokerPorts) -> ServicePeer {
    ServicePeer {
        backend: format!("tcp://127.0.0.1:{}", ports.svc_be),
        broker_id: broker_id.to_string(),
        join: true,
    }
}

fn config(id: &str, peers: Vec<ServicePeer>, ports: &ServiceBrokerPorts) -> RobotBusConfig {
    let mut cfg = federated_service_config(id, peers, ports);
    cfg.service.heartbeat_interval_ms = 100;
    cfg.service.heartbeat_timeout_ms = 1000;
    cfg
}

fn client(broker: &RobotBusBroker) -> ServiceClient {
    ServiceClient::new(Some(&connect_addr(&broker.service.frontend_bind))).expect("client")
}

fn worker(broker: &RobotBusBroker, svc: &str, tag: &'static str) -> WorkerThread {
    WorkerThread::spawn_service(
        svc,
        Arc::new(move |body| [tag.as_bytes(), b":", body].concat()),
        &connect_addr(&broker.service.backend_bind),
    )
    .expect("worker")
}

/// Call until it succeeds (links need a moment to advertise).
fn call_until(client: &ServiceClient, svc: &str, body: &[u8]) -> Vec<u8> {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        match client.call(svc, body, None, Some(Duration::from_millis(800))) {
            Ok(reply) => return reply,
            Err(e) if Instant::now() >= deadline => panic!("{svc}: never answered: {e:?}"),
            Err(_) => thread::sleep(Duration::from_millis(100)),
        }
    }
}

#[test]
fn joining_broker_calls_and_is_called_in_both_directions() {
    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    // B has no peers and is never reconfigured.
    let broker_b = RobotBusBroker::start(config("broker-b", Vec::new(), b)).expect("broker b");
    let w_b = worker(&broker_b, "svc.b", "b");
    thread::sleep(Duration::from_millis(100));
    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("broker-b", b)], a))
        .expect("broker a");
    let w_a = worker(&broker_a, "svc.a", "a");

    assert_eq!(call_until(&client(&broker_a), "svc.b", b"1"), b"b:1");
    assert_eq!(call_until(&client(&broker_b), "svc.a", b"2"), b"a:2");

    w_a.stop();
    w_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn configuring_both_sides_serves_each_call_once() {
    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("broker-b", b)], a))
        .expect("broker a");
    let broker_b = RobotBusBroker::start(config("broker-b", vec![join_peer("broker-a", a)], b))
        .expect("broker b");

    let hits_a = Arc::new(AtomicUsize::new(0));
    let hits_b = Arc::new(AtomicUsize::new(0));
    let (ha, hb) = (hits_a.clone(), hits_b.clone());
    let w_a = WorkerThread::spawn_service(
        "svc.a",
        Arc::new(move |body| {
            ha.fetch_add(1, Ordering::SeqCst);
            [b"a:", body].concat()
        }),
        &connect_addr(&broker_a.service.backend_bind),
    )
    .expect("worker a");
    let w_b = WorkerThread::spawn_service(
        "svc.b",
        Arc::new(move |body| {
            hb.fetch_add(1, Ordering::SeqCst);
            [b"b:", body].concat()
        }),
        &connect_addr(&broker_b.service.backend_bind),
    )
    .expect("worker b");

    assert_eq!(call_until(&client(&broker_a), "svc.b", b"x"), b"b:x");
    assert_eq!(call_until(&client(&broker_b), "svc.a", b"y"), b"a:y");
    // Let any stray duplicate show up.
    thread::sleep(Duration::from_millis(300));
    assert_eq!(hits_a.load(Ordering::SeqCst), 1, "svc.a handled once");
    assert_eq!(hits_b.load(Ordering::SeqCst), 1, "svc.b handled once");

    w_a.stop();
    w_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn two_brokers_joining_one_hub_reach_each_other_through_it() {
    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(3);
    let (a, b, c) = (&ports[0], &ports[1], &ports[2]);

    let hub = RobotBusBroker::start(config("hub", Vec::new(), b)).expect("hub");
    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("hub", b)], a))
        .expect("broker a");
    let broker_c = RobotBusBroker::start(config("broker-c", vec![join_peer("hub", b)], c))
        .expect("broker c");
    let w_a = worker(&broker_a, "svc.a", "a");
    let w_c = worker(&broker_c, "svc.c", "c");

    assert_eq!(call_until(&client(&broker_a), "svc.c", b"1"), b"c:1");
    assert_eq!(call_until(&client(&broker_c), "svc.a", b"2"), b"a:2");

    w_a.stop();
    w_c.stop();
    broker_a.stop().expect("stop a");
    broker_c.stop().expect("stop c");
    hub.stop().expect("stop hub");
}

#[test]
fn restarted_joined_broker_is_found_again() {
    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("broker-b", b)], a))
        .expect("broker a");
    let broker_b = RobotBusBroker::start(config("broker-b", Vec::new(), b)).expect("broker b");
    let w_b = worker(&broker_b, "svc.b", "b");
    assert_eq!(call_until(&client(&broker_a), "svc.b", b"1"), b"b:1");

    w_b.stop();
    broker_b.stop().expect("stop b");

    // B comes back with nothing configured; A rejoins on its own.
    let broker_b = RobotBusBroker::start(config("broker-b", Vec::new(), b)).expect("broker b2");
    let w_b = worker(&broker_b, "svc.b", "b2");
    assert_eq!(call_until(&client(&broker_a), "svc.b", b"2"), b"b2:2");
    let w_a = worker(&broker_a, "svc.a", "a");
    assert_eq!(call_until(&client(&broker_b), "svc.a", b"3"), b"a:3");

    w_a.stop();
    w_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn leaving_broker_takes_its_services_with_it() {
    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_b = RobotBusBroker::start(config("broker-b", Vec::new(), b)).expect("broker b");
    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("broker-b", b)], a))
        .expect("broker a");
    let w_a = worker(&broker_a, "svc.a", "a");
    let client_b = client(&broker_b);
    assert_eq!(call_until(&client_b, "svc.a", b"1"), b"a:1");

    w_a.stop();
    broker_a.stop().expect("stop a");

    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if client_b
            .call("svc.a", b"2", None, Some(Duration::from_millis(500)))
            .is_err()
        {
            break;
        }
        assert!(Instant::now() < deadline, "svc.a still routed after A left");
        thread::sleep(Duration::from_millis(100));
    }
    broker_b.stop().expect("stop b");
}

#[test]
fn legacy_peers_still_work_with_mutual_configuration() {
    let _guard = lock_brokers();
    let ports = alloc_service_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);
    let legacy = |id: &str, p: &ServiceBrokerPorts| ServicePeer {
        join: false,
        ..join_peer(id, p)
    };

    let broker_a = RobotBusBroker::start(config("broker-a", vec![legacy("broker-b", b)], a))
        .expect("broker a");
    let broker_b = RobotBusBroker::start(config("broker-b", vec![legacy("broker-a", a)], b))
        .expect("broker b");
    let w_a = worker(&broker_a, "svc.a", "a");
    let w_b = worker(&broker_b, "svc.b", "b");

    assert_eq!(call_until(&client(&broker_a), "svc.b", b"1"), b"b:1");
    assert_eq!(call_until(&client(&broker_b), "svc.a", b"2"), b"a:2");

    w_a.stop();
    w_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}
