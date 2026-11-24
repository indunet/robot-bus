//! Replies produced on worker threads must leave the executor without waiting
//! for the next socket event or the poll timeout.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::common::{BrokerLockGuard, ephemeral_robot_bus_config, lock_brokers};
use robot_bus::action_bus::{ActionClient, ActionKind};
use robot_bus::service_bus::ServiceClient;
use robot_bus::{CallbackGroup, Executor};
use robot_bus_broker::{RobotBusBroker};

/// Default heartbeat timeouts: the short ones in `BrokerProcess` expire before
/// the executor's 2.5 s worker heartbeat.
struct Broker {
    service_frontend: String,
    service_backend: String,
    action_frontend: String,
    action_backend: String,
    _broker: RobotBusBroker,
    _guard: BrokerLockGuard,
}

impl Broker {
    fn spawn() -> Self {
        let guard = lock_brokers();
        let broker = RobotBusBroker::start(ephemeral_robot_bus_config()).expect("start broker");
        Self {
            service_frontend: broker.service.frontend_bind.clone(),
            service_backend: broker.service.backend_bind.clone(),
            action_frontend: broker.action.frontend_bind.clone(),
            action_backend: broker.action.backend_bind.clone(),
            _broker: broker,
            _guard: guard,
        }
    }
}

/// Well below the 250 ms executor poll timeout, with headroom for loaded CI.
const MAX_MEDIAN: Duration = Duration::from_millis(100);
/// Handlers finish after the executor has already flushed once and gone back to poll.
const HANDLER_WORK: Duration = Duration::from_millis(5);

fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() / 2]
}

#[test]
fn action_result_is_sent_without_waiting_for_poll_timeout() {
    let broker = Broker::spawn();
    let mut executor = Executor::new();
    executor
        .register_action(
            "wake.action",
            Arc::new(|body| {
                thread::sleep(HANDLER_WORK);
                vec![
                    ("FEEDBACK".into(), b"f".to_vec()),
                    ("RESULT".into(), body.to_vec()),
                ]
            }),
            CallbackGroup::mutually_exclusive(),
            Some(&broker.action_backend),
            None,
            None,
        )
        .expect("register action");
    executor.start().expect("start");
    thread::sleep(Duration::from_millis(200));

    let client = ActionClient::new(Some(&broker.action_frontend)).expect("client");
    let mut samples = Vec::new();
    for index in 0..7 {
        let start = Instant::now();
        let messages = client
            .send_goal("wake.action", b"go", None, Some(Duration::from_secs(5)))
            .expect("goal");
        let elapsed = start.elapsed();
        assert_eq!(messages.last().expect("result").kind, ActionKind::Result);
        assert_eq!(messages.last().expect("result").body, b"go");
        if index > 0 {
            samples.push(elapsed);
        }
    }
    executor.stop();

    let median = median(samples);
    assert!(median < MAX_MEDIAN, "action median latency {median:?}");
}

#[test]
fn pooled_service_reply_is_sent_without_waiting_for_poll_timeout() {
    let broker = Broker::spawn();
    let mut executor = Executor::with_worker_pool(2);
    executor
        .register_service(
            "wake.service",
            Arc::new(|body| {
                thread::sleep(HANDLER_WORK);
                body.to_vec()
            }),
            CallbackGroup::reentrant(),
            Some(&broker.service_backend),
            None,
            None,
        )
        .expect("register service");
    executor.start().expect("start");
    thread::sleep(Duration::from_millis(200));

    let client = ServiceClient::new(Some(&broker.service_frontend)).expect("client");
    let mut samples = Vec::new();
    for index in 0..7 {
        let start = Instant::now();
        let reply = client
            .call("wake.service", b"ping", None, Some(Duration::from_secs(5)))
            .expect("call");
        let elapsed = start.elapsed();
        assert_eq!(reply, b"ping");
        if index > 0 {
            samples.push(elapsed);
        }
    }
    executor.stop();

    let median = median(samples);
    assert!(median < MAX_MEDIAN, "service median latency {median:?}");
}

#[test]
fn shutdown_interrupts_blocking_poll() {
    let broker = Broker::spawn();
    let mut samples = Vec::new();
    for _ in 0..5 {
        let mut executor = Executor::new();
        executor
            .register_service(
                "wake.idle",
                Arc::new(|body| body.to_vec()),
                CallbackGroup::mutually_exclusive(),
                Some(&broker.service_backend),
                None,
                None,
            )
            .expect("register service");
        executor.start().expect("start");
        thread::sleep(Duration::from_millis(30));
        let handle = executor.shutdown_handle();
        let start = Instant::now();
        handle.shutdown();
        executor.wait();
        samples.push(start.elapsed());
        executor.stop();
    }
    let slowest = samples.into_iter().max().expect("samples");
    assert!(slowest < MAX_MEDIAN, "slowest shutdown {slowest:?}");
}
