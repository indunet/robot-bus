//! WebSocket RPC scenarios.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::support::{LatencyStats, ScenarioResult};
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{HighWaterMark, Node, Publisher, QosProfile, Subscriber};
use robot_bus_broker::{RobotBusBroker};

use crate::pacing::{TrialReceiver, WARMUP, make_payload, measure_pubsub, msg_hwm, rpc_warmup};

pub fn bench_ws_publish(broker: &RobotBusBroker, url: &str) -> ScenarioResult {
    let transport = "ws";
    let scenario = "message Publish";
    let topic = "perf/ws/pub";

    let mut node = Node::ws_at("perf-ws-pub", url);
    let publisher = match node.create_publisher_raw(topic) {
        Ok(p) => p,
        Err(err) => {
            return ScenarioResult::skipped(transport, scenario, format!("publisher: {err}"));
        }
    };

    let hwm = HighWaterMark {
        snd: msg_hwm(),
        rcv: msg_hwm(),
    };
    let sub = match Subscriber::with_hwm(Some(&broker.discover.message_xpub), hwm) {
        Ok(s) => s,
        Err(err) => {
            return ScenarioResult::skipped(transport, scenario, format!("subscriber: {err}"));
        }
    };
    if let Err(err) = sub.subscribe(topic) {
        return ScenarioResult::skipped(transport, scenario, format!("subscribe: {err}"));
    }

    let receiver = Arc::new(TrialReceiver::default());
    let running = Arc::new(AtomicBool::new(true));
    let recv = thread::spawn({
        let receiver = Arc::clone(&receiver);
        let running = Arc::clone(&running);
        move || {
            while running.load(Ordering::Relaxed) {
                if let Ok((_, payload)) = sub.receive(Some(Duration::from_millis(50))) {
                    receiver.receive(&payload);
                }
            }
        }
    });
    thread::sleep(Duration::from_millis(400));
    let result = measure_pubsub(transport, scenario, &receiver, |payload| {
        publisher.publish(payload)
    });
    running.store(false, Ordering::Relaxed);
    recv.join().expect("WS publish subscriber thread");
    result
}

pub fn bench_ws_subscribe(broker: &RobotBusBroker, url: &str) -> ScenarioResult {
    let transport = "ws";
    let scenario = "message Subscribe";
    let topic = "perf/ws/msg";
    let receiver = Arc::new(TrialReceiver::default());
    let mut node = Node::ws_at("perf-ws-sub", url);
    if let Err(err) = node.create_subscription_raw_with_qos(
        topic,
        QosProfile::keep_last(msg_hwm()),
        Arc::new({
            let receiver = Arc::clone(&receiver);
            move |payload| receiver.receive(payload)
        }),
        None,
    ) {
        return ScenarioResult::skipped(transport, scenario, format!("subscribe: {err}"));
    }

    let hwm = HighWaterMark {
        snd: msg_hwm(),
        rcv: msg_hwm(),
    };
    let publisher = match Publisher::with_hwm(Some(&broker.discover.message_xsub), hwm) {
        Ok(p) => p,
        Err(err) => {
            return ScenarioResult::skipped(transport, scenario, format!("publisher: {err}"));
        }
    };
    let shutdown = match node.shutdown_handle() {
        Ok(h) => h,
        Err(err) => {
            return ScenarioResult::skipped(transport, scenario, format!("shutdown handle: {err}"));
        }
    };
    let recv = thread::spawn(move || node.spin());
    thread::sleep(Duration::from_millis(400));
    let result = measure_pubsub(transport, scenario, &receiver, |payload| {
        publisher.publish(topic, payload)
    });
    shutdown.shutdown();
    match recv.join().expect("WS subscriber thread") {
        Ok(()) => result,
        Err(err) => ScenarioResult::skipped(transport, scenario, format!("subscriber spin: {err}")),
    }
}

pub fn bench_ws_service(broker: &RobotBusBroker, url: &str, n: usize) -> ScenarioResult {
    let transport = "ws";
    let scenario = "service Call";
    let name = "perf.ws.echo";

    let handler: Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync> = Arc::new(|body| body.to_vec());
    let worker = match WorkerThread::spawn_service(name, handler, &broker.service.backend_bind) {
        Ok(w) => w,
        Err(err) => return ScenarioResult::skipped(transport, scenario, format!("worker: {err}")),
    };
    thread::sleep(Duration::from_millis(150));

    let mut node = Node::ws_at("perf-ws-cli", url);
    let client = match node.create_client_raw(name) {
        Ok(c) => c,
        Err(err) => {
            worker.stop();
            return ScenarioResult::skipped(transport, scenario, format!("create_client: {err}"));
        }
    };

    let payload = make_payload(0);
    for _ in 0..rpc_warmup(WARMUP) {
        let _ = client.call(&payload, Some(Duration::from_secs(2)));
    }

    let mut samples = Vec::with_capacity(n);
    let t0 = Instant::now();
    let mut received = 0usize;
    let mut attempted = 0;
    for _ in 0..n {
        attempted += 1;
        let start = Instant::now();
        match client.call(&payload, Some(Duration::from_secs(5))) {
            Ok(_) => {
                samples.push(start.elapsed().as_nanos() as u64);
                received += 1;
            }
            Err(err) => {
                eprintln!("{scenario} attempt {attempted} failed: {err}");
                if received == 0 {
                    worker.stop();
                    return ScenarioResult::skipped(
                        transport,
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

    ScenarioResult::ok_rpc(
        transport,
        scenario,
        attempted,
        received,
        elapsed,
        LatencyStats::from_ns(samples),
    )
}

pub fn bench_ws_action(broker: &RobotBusBroker, url: &str, n: usize) -> ScenarioResult {
    let transport = "ws";
    let scenario = "action SendGoal";
    let name = "perf.ws.act";

    let handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> = Arc::new(|body| {
        vec![
            ("FEEDBACK".into(), b"f".to_vec()),
            ("RESULT".into(), body.to_vec()),
        ]
    });
    let worker = match WorkerThread::spawn_action(name, handler, &broker.action.backend_bind) {
        Ok(w) => w,
        Err(err) => return ScenarioResult::skipped(transport, scenario, format!("worker: {err}")),
    };
    thread::sleep(Duration::from_millis(150));

    let mut node = Node::ws_at("perf-ws-act", url);
    let client = match node.create_action_client_raw(name) {
        Ok(c) => c,
        Err(err) => {
            worker.stop();
            return ScenarioResult::skipped(
                transport,
                scenario,
                format!("create_action_client: {err}"),
            );
        }
    };

    let payload = make_payload(0);
    for _ in 0..rpc_warmup(WARMUP.min(10)) {
        let _ = client.send_goal_and_wait(&payload, None, Some(Duration::from_secs(2)));
    }

    let mut samples = Vec::with_capacity(n);
    let t0 = Instant::now();
    let mut received = 0usize;
    let mut attempted = 0;
    for _ in 0..n {
        attempted += 1;
        let start = Instant::now();
        match client.send_goal_and_wait(&payload, None, Some(Duration::from_secs(5))) {
            Ok(_) => {
                samples.push(start.elapsed().as_nanos() as u64);
                received += 1;
            }
            Err(err) => {
                eprintln!("{scenario} attempt {attempted} failed: {err}");
                if received == 0 {
                    worker.stop();
                    return ScenarioResult::skipped(
                        transport,
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

    ScenarioResult::ok_rpc(
        transport,
        scenario,
        attempted,
        received,
        elapsed,
        LatencyStats::from_ns(samples),
    )
}
