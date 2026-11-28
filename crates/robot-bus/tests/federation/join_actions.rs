//! One-sided action federation: only the joining broker lists `--peer`.

use robot_bus::zmq_helpers as zmq;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::action_bus_federation::{
    ActionBrokerPorts, alloc_action_broker_ports, connect_addr, demo_handler,
    federated_action_config,
};
use crate::common::lock_brokers;
use robot_bus::action_bus::{ActionClient, ActionKind, ActionMessage};
use robot_bus::broker::action_bus::ActionPeer;
use robot_bus::errors::BusError;
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{RobotBusBroker, RobotBusConfig};

fn join_peer(broker_id: &str, ports: &ActionBrokerPorts) -> ActionPeer {
    ActionPeer {
        backend: format!("tcp://127.0.0.1:{}", ports.act_be),
        broker_id: broker_id.to_string(),
        join: true,
    }
}

fn legacy_peer(broker_id: &str, ports: &ActionBrokerPorts) -> ActionPeer {
    ActionPeer {
        join: false,
        ..join_peer(broker_id, ports)
    }
}

fn config(id: &str, peers: Vec<ActionPeer>, ports: &ActionBrokerPorts) -> RobotBusConfig {
    let mut cfg = federated_action_config(id, peers, ports);
    cfg.action.heartbeat_interval_ms = 100;
    cfg.action.heartbeat_timeout_ms = 1000;
    cfg
}

fn client(broker: &RobotBusBroker) -> ActionClient {
    ActionClient::new(Some(&connect_addr(&broker.action.frontend_bind))).expect("client")
}

fn worker(broker: &RobotBusBroker, action: &str, tag: &'static [u8]) -> WorkerThread {
    WorkerThread::spawn_action(
        action,
        demo_handler(tag),
        &connect_addr(&broker.action.backend_bind),
    )
    .expect("worker")
}

/// Send a goal until it is answered (links need a moment to advertise).
///
/// Every attempt uses a fresh client: a goal that waited for a route may be answered after
/// its attempt gave up, and that late reply must not reach the next attempt.
fn goal_until(broker: &RobotBusBroker, action: &str, body: &[u8]) -> Vec<ActionMessage> {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        match client(broker).send_goal(action, body, None, Some(Duration::from_millis(800))) {
            Ok(msgs) if msgs.iter().any(|m| m.kind == ActionKind::Result) => return msgs,
            Ok(_) => {}
            Err(e) if Instant::now() >= deadline => panic!("{action}: never answered: {e:?}"),
            Err(_) => {}
        }
        assert!(Instant::now() < deadline, "{action}: never answered");
        thread::sleep(Duration::from_millis(100));
    }
}

fn result_of(msgs: &[ActionMessage]) -> &[u8] {
    &msgs
        .iter()
        .find(|m| m.kind == ActionKind::Result)
        .expect("RESULT")
        .body
}

#[test]
fn joining_broker_runs_goals_in_both_directions() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    // B has no peers and is never reconfigured.
    let broker_b = RobotBusBroker::start(config("broker-b", Vec::new(), b)).expect("broker b");
    let w_b = worker(&broker_b, "act.b", b"b");
    thread::sleep(Duration::from_millis(100));
    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("broker-b", b)], a))
        .expect("broker a");
    let w_a = worker(&broker_a, "act.a", b"a");

    let msgs = goal_until(&broker_a, "act.b", b"1");
    assert!(msgs.iter().any(|m| m.kind == ActionKind::Feedback));
    assert_eq!(result_of(&msgs), b"done:b:1");
    let msgs = goal_until(&broker_b, "act.a", b"2");
    assert!(msgs.iter().any(|m| m.kind == ActionKind::Feedback));
    assert_eq!(result_of(&msgs), b"done:a:2");

    w_a.stop();
    w_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

/// A worker that reports FEEDBACK for a GOAL and answers a CANCEL with a RESULT.
fn cancellable_worker(backend: &str, action: &'static str) -> thread::JoinHandle<()> {
    let ctx = zmq::Context::new();
    let sock = ctx.socket(zmq::SocketType::DEALER).expect("dealer");
    sock.set_linger(0).unwrap();
    sock.connect(backend).expect("connect");
    sock.send_multipart([b"READY".as_slice(), action.as_bytes()], 0)
        .expect("ready");
    thread::spawn(move || {
        let _ctx = ctx;
        let end = Instant::now() + Duration::from_secs(20);
        let mut last_hb = Instant::now();
        while Instant::now() < end {
            if last_hb.elapsed() > Duration::from_millis(100) {
                let _ = sock.send_multipart(
                    [b"HEARTBEAT".as_slice(), action.as_bytes()],
                    zmq::DONTWAIT,
                );
                last_hb = Instant::now();
            }
            if sock.poll(zmq::POLLIN, 50).unwrap_or(0) == 0 {
                continue;
            }
            let Ok(f) = sock.recv_multipart(0) else { continue };
            if f.len() != 5 {
                continue;
            }
            let (cid, act, gid, kind) = (&f[0], &f[1], &f[2], &f[3]);
            match kind.as_slice() {
                b"GOAL" => {
                    let _ = sock.send_multipart(
                        [cid.as_slice(), act, gid, b"FEEDBACK", b"started"],
                        0,
                    );
                }
                b"CANCEL" => {
                    let _ = sock.send_multipart(
                        [cid.as_slice(), act, gid, b"RESULT", b"cancelled"],
                        0,
                    );
                    return;
                }
                _ => {}
            }
        }
    })
}

fn cancel_roundtrip(caller: &RobotBusBroker, action: &'static str) {
    let c = client(caller);
    // Wait for the route, then start a goal that only ends by cancel.
    let deadline = Instant::now() + Duration::from_secs(8);
    let gid = loop {
        let gid = c.submit_goal(action, b"go", None).expect("submit");
        match c.recv_message(Some(Duration::from_millis(800))) {
            Ok(m) if m.goal_id == gid && m.kind == ActionKind::Feedback => break gid,
            Ok(m) if m.goal_id == gid && m.kind == ActionKind::Result => {
                assert!(
                    Instant::now() < deadline,
                    "{action}: no route yet: {:?}",
                    robot_bus::parse_error_body(&m.body)
                );
                thread::sleep(Duration::from_millis(100));
            }
            Ok(_) | Err(BusError::Timeout(_)) => {
                assert!(Instant::now() < deadline, "{action}: never answered");
            }
            Err(e) => panic!("{action}: {e:?}"),
        }
    };
    let res = c
        .cancel(action, &gid, b"", Some(Duration::from_secs(3)))
        .expect("cancel");
    assert_eq!(res.body, b"cancelled");
}

#[test]
fn cancel_crosses_a_join_link_in_both_directions() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_b = RobotBusBroker::start(config("broker-b", Vec::new(), b)).expect("broker b");
    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("broker-b", b)], a))
        .expect("broker a");
    let w_b = cancellable_worker(&connect_addr(&broker_b.action.backend_bind), "act.b");
    let w_a = cancellable_worker(&connect_addr(&broker_a.action.backend_bind), "act.a");

    cancel_roundtrip(&broker_a, "act.b"); // A -> B
    // The A-side worker ended after its cancel; start a fresh one for the B -> A direction.
    w_a.join().ok();
    let w_a = cancellable_worker(&connect_addr(&broker_a.action.backend_bind), "act.a");
    cancel_roundtrip(&broker_b, "act.a"); // B -> A

    w_a.join().ok();
    w_b.join().ok();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn configuring_both_sides_runs_each_goal_once() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("broker-b", b)], a))
        .expect("broker a");
    let broker_b = RobotBusBroker::start(config("broker-b", vec![join_peer("broker-a", a)], b))
        .expect("broker b");

    let hits_a = Arc::new(AtomicUsize::new(0));
    let hits_b = Arc::new(AtomicUsize::new(0));
    let (ha, hb) = (hits_a.clone(), hits_b.clone());
    let w_a = WorkerThread::spawn_action(
        "act.a",
        Arc::new(move |body| {
            ha.fetch_add(1, Ordering::SeqCst);
            vec![("RESULT".into(), [b"a:", body].concat())]
        }),
        &connect_addr(&broker_a.action.backend_bind),
    )
    .expect("worker a");
    let w_b = WorkerThread::spawn_action(
        "act.b",
        Arc::new(move |body| {
            hb.fetch_add(1, Ordering::SeqCst);
            vec![("RESULT".into(), [b"b:", body].concat())]
        }),
        &connect_addr(&broker_b.action.backend_bind),
    )
    .expect("worker b");

    // One call each (no retries): a retried goal would count as a second execution.
    // A goal that arrives before the route is known waits in the broker's pending queue.
    let once = |broker: &RobotBusBroker, action: &str, body: &[u8]| {
        client(broker)
            .send_goal(action, body, None, Some(Duration::from_secs(6)))
            .expect("goal")
    };
    thread::sleep(Duration::from_millis(500));
    assert_eq!(result_of(&once(&broker_a, "act.b", b"x")), b"b:x");
    assert_eq!(result_of(&once(&broker_b, "act.a", b"y")), b"a:y");
    thread::sleep(Duration::from_millis(300));
    assert_eq!(hits_a.load(Ordering::SeqCst), 1, "act.a handled once");
    assert_eq!(hits_b.load(Ordering::SeqCst), 1, "act.b handled once");

    w_a.stop();
    w_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn two_brokers_joining_one_hub_reach_each_other_through_it() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(3);
    let (a, b, c) = (&ports[0], &ports[1], &ports[2]);

    let hub = RobotBusBroker::start(config("hub", Vec::new(), b)).expect("hub");
    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("hub", b)], a))
        .expect("broker a");
    let broker_c = RobotBusBroker::start(config("broker-c", vec![join_peer("hub", b)], c))
        .expect("broker c");
    let w_a = worker(&broker_a, "act.a", b"a");
    let w_c = worker(&broker_c, "act.c", b"c");

    assert_eq!(
        result_of(&goal_until(&broker_a, "act.c", b"1")),
        b"done:c:1"
    );
    assert_eq!(
        result_of(&goal_until(&broker_c, "act.a", b"2")),
        b"done:a:2"
    );

    w_a.stop();
    w_c.stop();
    broker_a.stop().expect("stop a");
    broker_c.stop().expect("stop c");
    hub.stop().expect("stop hub");
}

#[test]
fn restarted_joined_broker_is_found_again() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("broker-b", b)], a))
        .expect("broker a");
    let broker_b = RobotBusBroker::start(config("broker-b", Vec::new(), b)).expect("broker b");
    let w_b = worker(&broker_b, "act.b", b"b");
    assert_eq!(
        result_of(&goal_until(&broker_a, "act.b", b"1")),
        b"done:b:1"
    );

    w_b.stop();
    broker_b.stop().expect("stop b");

    // B comes back with nothing configured; A rejoins on its own.
    let broker_b = RobotBusBroker::start(config("broker-b", Vec::new(), b)).expect("broker b2");
    let w_b = worker(&broker_b, "act.b", b"b2");
    assert_eq!(
        result_of(&goal_until(&broker_a, "act.b", b"2")),
        b"done:b2:2"
    );
    let w_a = worker(&broker_a, "act.a", b"a");
    assert_eq!(
        result_of(&goal_until(&broker_b, "act.a", b"3")),
        b"done:a:3"
    );

    w_a.stop();
    w_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}

#[test]
fn leaving_broker_takes_its_actions_with_it() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_b = RobotBusBroker::start(config("broker-b", Vec::new(), b)).expect("broker b");
    let broker_a = RobotBusBroker::start(config("broker-a", vec![join_peer("broker-b", b)], a))
        .expect("broker a");
    let w_a = worker(&broker_a, "act.a", b"a");
    let client_b = client(&broker_b);
    assert_eq!(result_of(&goal_until(&broker_b, "act.a", b"1")), b"done:a:1");

    w_a.stop();
    broker_a.stop().expect("stop a");

    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if client_b
            .send_goal("act.a", b"2", None, Some(Duration::from_millis(500)))
            .is_err()
        {
            break;
        }
        assert!(Instant::now() < deadline, "act.a still routed after A left");
        thread::sleep(Duration::from_millis(100));
    }
    broker_b.stop().expect("stop b");
}

#[test]
fn legacy_peers_still_work_with_mutual_configuration() {
    let _guard = lock_brokers();
    let ports = alloc_action_broker_ports(2);
    let (a, b) = (&ports[0], &ports[1]);

    let broker_a = RobotBusBroker::start(config("broker-a", vec![legacy_peer("broker-b", b)], a))
        .expect("broker a");
    let broker_b = RobotBusBroker::start(config("broker-b", vec![legacy_peer("broker-a", a)], b))
        .expect("broker b");
    let w_a = worker(&broker_a, "act.a", b"a");
    let w_b = worker(&broker_b, "act.b", b"b");

    assert_eq!(
        result_of(&goal_until(&broker_a, "act.b", b"1")),
        b"done:b:1"
    );
    assert_eq!(
        result_of(&goal_until(&broker_b, "act.a", b"2")),
        b"done:a:2"
    );

    w_a.stop();
    w_b.stop();
    broker_a.stop().expect("stop a");
    broker_b.stop().expect("stop b");
}
