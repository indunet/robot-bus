//! Integration tests for the action bus broker.
//!
//! Each test spins up a real broker (in-process, ephemeral ports) plus a
//! client (DEALER) and worker(s) (DEALER), then verifies end-to-end routing
//! of goal/feedback/result and cancel flows.

use robot_bus::broker::action_bus::{ActionBusConfig, run_loop};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use zmq::{Context as ZmqContext, SocketType};

/// Helper: bind a ROUTER to `tcp://127.0.0.1:0` and return (socket, real_endpoint).
fn bind_ephemeral_router(ctx: &ZmqContext) -> (zmq::Socket, String) {
    let sock = ctx.socket(SocketType::ROUTER).expect("create ROUTER");
    sock.bind("tcp://127.0.0.1:0").expect("bind ephemeral");
    let endpoint = match sock.get_last_endpoint().expect("last_endpoint") {
        Ok(s) => s,
        Err(_) => panic!("endpoint not utf8"),
    };
    (sock, endpoint)
}

/// Spawn a broker thread bound to ephemeral ports.
struct BrokerHandle {
    shutdown: Arc<AtomicBool>,
    frontend_ep: String,
    backend_ep: String,
    handle: Option<thread::JoinHandle<()>>,
}

impl Drop for BrokerHandle {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn spawn_broker(config: ActionBusConfig) -> BrokerHandle {
    let ctx = ZmqContext::new();
    let (frontend, frontend_ep) = bind_ephemeral_router(&ctx);
    let (backend, backend_ep) = bind_ephemeral_router(&ctx);
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_broker = shutdown.clone();
    let cfg = config.clone();
    let handle = thread::spawn(move || {
        let _ = run_loop(&frontend, &backend, &cfg, &shutdown_broker, None);
    });
    thread::sleep(Duration::from_millis(50));
    BrokerHandle {
        shutdown,
        frontend_ep,
        backend_ep,
        handle: Some(handle),
    }
}

/// A minimal action worker: connect DEALER, send READY, then on each GOAL
/// emit `n_feedback` FEEDBACK messages followed by one RESULT. `tag` is
/// prepended to each response body so the test can identify which worker.
struct WorkerHandle {
    handle: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

fn spawn_worker(backend_ep: &str, action: &str, tag: &str, n_feedback: usize) -> WorkerHandle {
    let tag = tag.to_string();
    let action = action.to_string();
    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::DEALER).expect("create DEALER");
    let id = format!("worker-{tag}");
    sock.set_identity(id.as_bytes()).expect("set identity");
    sock.connect(backend_ep).expect("connect backend");

    sock.send_multipart([b"READY".as_ref(), action.as_bytes()], 0)
        .expect("send READY");

    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let handle = thread::spawn(move || {
        sock.set_rcvtimeo(100).expect("set rcvtimeo");
        while !stop_clone.load(Ordering::Acquire) {
            let frames = match sock.recv_multipart(0) {
                Ok(f) => f,
                Err(zmq::Error::EAGAIN) => continue,
                Err(e) => panic!("worker recv: {e}"),
            };
            // broker→worker GOAL/CANCEL: [client_id][action][goal_id][kind][body]
            if frames.len() != 5 {
                continue;
            }
            let client_id = &frames[0];
            let act = &frames[1];
            let goal_id = &frames[2];
            let kind = &frames[3];
            let body = &frames[4];
            if kind == b"CANCEL" {
                // Acknowledge the cancel with a RESULT.
                let resp =
                    format!("{}:CANCELLED:{}", tag, String::from_utf8_lossy(body)).into_bytes();
                sock.send_multipart(
                    [
                        client_id.as_slice(),
                        act.as_slice(),
                        goal_id.as_slice(),
                        b"RESULT".as_ref(),
                        resp.as_slice(),
                    ],
                    0,
                )
                .expect("worker send cancel-result");
                continue;
            }
            // kind == GOAL: emit feedback then result.
            for i in 0..n_feedback {
                let fb = format!("{}:fb{}", tag, i).into_bytes();
                sock.send_multipart(
                    [
                        client_id.as_slice(),
                        act.as_slice(),
                        goal_id.as_slice(),
                        b"FEEDBACK".as_ref(),
                        fb.as_slice(),
                    ],
                    0,
                )
                .expect("worker send feedback");
            }
            let res = format!("{}:done:{}", tag, String::from_utf8_lossy(body)).into_bytes();
            sock.send_multipart(
                [
                    client_id.as_slice(),
                    act.as_slice(),
                    goal_id.as_slice(),
                    b"RESULT".as_ref(),
                    res.as_slice(),
                ],
                0,
            )
            .expect("worker send result");
        }
    });
    thread::sleep(Duration::from_millis(80));
    WorkerHandle {
        handle: Some(handle),
        stop,
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// A DEALER client that sends a GOAL and collects all responses until RESULT.
fn send_goal_collect(
    client: &zmq::Socket,
    action: &str,
    goal_id: &str,
    body: &str,
) -> Vec<Vec<Vec<u8>>> {
    client
        .send_multipart(
            [
                action.as_bytes(),
                goal_id.as_bytes(),
                b"GOAL".as_ref(),
                body.as_bytes(),
            ],
            0,
        )
        .expect("client send goal");
    client.set_rcvtimeo(2000).expect("set rcvtimeo");
    let mut responses = Vec::new();
    loop {
        let frames = client.recv_multipart(0).expect("client recv");
        // broker→client: [action][goal_id][kind][body]
        if frames.len() != 4 {
            panic!("unexpected frame count: {frames:?}");
        }
        let is_result = frames[2] == b"RESULT";
        responses.push(frames);
        if is_result {
            break;
        }
    }
    responses
}

fn make_client(frontend_ep: &str) -> zmq::Socket {
    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::DEALER).expect("create DEALER");
    sock.connect(frontend_ep).expect("connect frontend");
    sock
}

fn default_test_config() -> ActionBusConfig {
    ActionBusConfig {
        heartbeat_interval_ms: 200,
        heartbeat_timeout_ms: 600,
        ..ActionBusConfig::default()
    }
}

#[test]
fn e2e_goal_feedback_result() {
    let broker = spawn_broker(default_test_config());
    let _worker = spawn_worker(&broker.backend_ep, "act.x", "A", 3);
    let client = make_client(&broker.frontend_ep);

    let responses = send_goal_collect(&client, "act.x", "g-1", "do-it");
    // 3 feedback + 1 result
    assert_eq!(responses.len(), 4);
    for (i, resp) in responses.iter().enumerate() {
        assert_eq!(resp[0], b"act.x");
        assert_eq!(resp[1], b"g-1");
        if i < 3 {
            assert_eq!(resp[2], b"FEEDBACK");
            assert_eq!(resp[3], format!("A:fb{i}").as_bytes());
        } else {
            assert_eq!(resp[2], b"RESULT");
            assert_eq!(resp[3], b"A:done:do-it".to_vec());
        }
    }
}

#[test]
fn route_by_action_name() {
    let broker = spawn_broker(default_test_config());
    let _w1 = spawn_worker(&broker.backend_ep, "act.a", "AA", 0);
    let _w2 = spawn_worker(&broker.backend_ep, "act.b", "BB", 0);
    let client = make_client(&broker.frontend_ep);

    let ra = send_goal_collect(&client, "act.a", "r1", "hello");
    let rb = send_goal_collect(&client, "act.b", "r2", "world");

    assert_eq!(ra.last().unwrap()[3], b"AA:done:hello".to_vec());
    assert_eq!(rb.last().unwrap()[3], b"BB:done:world".to_vec());
}

#[test]
fn no_worker_returns_error_result() {
    let mut cfg = default_test_config();
    cfg.heartbeat_interval_ms = 50;
    cfg.pending_timeout_ms = 200;
    let broker = spawn_broker(cfg);
    let client = make_client(&broker.frontend_ep);

    // No worker registered. The broker queues the goal; the pending timeout
    // fires and returns NO_WORKER as a RESULT.
    client
        .send_multipart(
            [
                "act.none".as_bytes(),
                "g1".as_bytes(),
                b"GOAL".as_ref(),
                "body".as_bytes(),
            ],
            0,
        )
        .expect("send");
    client.set_rcvtimeo(3000).expect("rcvtimeo");
    let reply = client.recv_multipart(0).expect("recv");
    assert_eq!(reply.len(), 4);
    assert_eq!(reply[0], b"act.none");
    assert_eq!(reply[1], b"g1");
    assert_eq!(reply[2], b"RESULT");
    assert!(reply[3].starts_with(b"NO_WORKER"));
    assert!(reply[3].ends_with(b"act.none"));
}

#[test]
fn ready_registers_multiple_workers_same_action() {
    let broker = spawn_broker(default_test_config());
    let _w1 = spawn_worker(&broker.backend_ep, "act.m", "M1", 0);
    let _w2 = spawn_worker(&broker.backend_ep, "act.m", "M2", 0);
    let client = make_client(&broker.frontend_ep);

    let mut tags = std::collections::HashSet::new();
    for i in 0..4 {
        let responses = send_goal_collect(&client, "act.m", &format!("g{i}"), "x");
        let body = String::from_utf8_lossy(&responses.last().unwrap()[3]).into_owned();
        let tag = body.split(':').next().unwrap();
        tags.insert(tag.to_string());
    }
    assert!(tags.contains("M1"), "M1 never got a goal: {tags:?}");
    assert!(tags.contains("M2"), "M2 never got a goal: {tags:?}");
}

#[test]
fn heartbeat_timeout_evicts_worker_and_reclaims_goal() {
    let mut cfg = default_test_config();
    cfg.heartbeat_interval_ms = 100;
    cfg.heartbeat_timeout_ms = 300;
    let broker = spawn_broker(cfg);

    // Register a worker that will go silent (we never send heartbeats).
    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::DEALER).expect("DEALER");
    sock.set_identity(b"worker-dying").expect("identity");
    sock.connect(&broker.backend_ep).expect("connect");
    sock.send_multipart([b"READY".as_ref(), b"act.die".as_ref()], 0)
        .expect("READY");
    thread::sleep(Duration::from_millis(80));

    let client = make_client(&broker.frontend_ep);
    // Send a goal; the dying worker receives it but never replies.
    client
        .send_multipart(
            [
                "act.die".as_bytes(),
                "g1".as_bytes(),
                b"GOAL".as_ref(),
                "body".as_bytes(),
            ],
            0,
        )
        .expect("send goal");
    client.set_rcvtimeo(5000).expect("rcvtimeo");
    // The worker took the goal but never replies. After the heartbeat timeout
    // fires (~380ms), the broker evicts the worker and synthesizes a
    // WORKER_DIED result for the in-flight goal.
    let reply = client.recv_multipart(0).expect("recv worker-died");
    assert_eq!(reply.len(), 4);
    assert_eq!(reply[0], b"act.die");
    assert_eq!(reply[1], b"g1");
    assert_eq!(reply[2], b"RESULT");
    assert!(reply[3].starts_with(b"WORKER_DIED"));
}

#[test]
fn cancel_routes_to_owning_worker() {
    let mut cfg = default_test_config();
    // Keep the silent worker alive long enough for the cancel handshake.
    cfg.heartbeat_interval_ms = 2000;
    cfg.heartbeat_timeout_ms = 10_000;
    let broker = spawn_broker(cfg);
    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::DEALER).expect("DEALER");
    sock.set_identity(b"worker-cancel").expect("identity");
    sock.connect(&broker.backend_ep).expect("connect");
    sock.send_multipart([b"READY".as_ref(), b"act.c".as_ref()], 0)
        .expect("READY");
    thread::sleep(Duration::from_millis(80));

    let client = make_client(&broker.frontend_ep);
    client
        .send_multipart(
            [
                "act.c".as_bytes(),
                "g1".as_bytes(),
                b"GOAL".as_ref(),
                "go".as_bytes(),
            ],
            0,
        )
        .expect("send goal");

    sock.set_rcvtimeo(2000).expect("rcvtimeo");
    let frames = sock.recv_multipart(0).expect("worker recv goal");
    assert_eq!(frames.len(), 5);
    let client_id = frames[0].clone();
    let act = frames[1].clone();
    let goal_id = frames[2].clone();

    client
        .send_multipart(
            [
                "act.c".as_bytes(),
                "g1".as_bytes(),
                b"CANCEL".as_ref(),
                b"please".as_ref(),
            ],
            0,
        )
        .expect("send cancel");

    let cancel_frames = sock.recv_multipart(0).expect("worker recv cancel");
    assert_eq!(cancel_frames.len(), 5);
    assert_eq!(cancel_frames[3], b"CANCEL");
    sock.send_multipart(
        [
            client_id.as_slice(),
            act.as_slice(),
            goal_id.as_slice(),
            b"RESULT".as_ref(),
            b"cancelled".as_ref(),
        ],
        0,
    )
    .expect("worker send cancel-result");

    client.set_rcvtimeo(2000).expect("rcvtimeo");
    let reply = client.recv_multipart(0).expect("client recv cancel-result");
    assert_eq!(reply[2], b"RESULT");
    assert_eq!(reply[3], b"cancelled".to_vec());
}

#[test]
fn cancel_unknown_goal_returns_no_goal_result() {
    let broker = spawn_broker(default_test_config());
    let _worker = spawn_worker(&broker.backend_ep, "act.u", "U", 0);
    let client = make_client(&broker.frontend_ep);

    // Cancel a goal that was never sent.
    client
        .send_multipart(
            [
                "act.u".as_bytes(),
                "ghost".as_bytes(),
                b"CANCEL".as_ref(),
                b"".as_ref(),
            ],
            0,
        )
        .expect("send cancel");
    client.set_rcvtimeo(2000).expect("rcvtimeo");
    let reply = client.recv_multipart(0).expect("recv");
    assert_eq!(reply[0], b"act.u");
    assert_eq!(reply[1], b"ghost");
    assert_eq!(reply[2], b"RESULT");
    assert!(reply[3].starts_with(b"NO_GOAL"));
}

#[test]
fn cancel_pending_goal_returns_cancelled() {
    let mut cfg = default_test_config();
    cfg.pending_timeout_ms = 5000;
    let broker = spawn_broker(cfg);
    let client = make_client(&broker.frontend_ep);

    // No worker → goal sits in pending.
    client
        .send_multipart(
            [
                "act.pend".as_bytes(),
                "g-pend".as_bytes(),
                b"GOAL".as_ref(),
                b"x".as_ref(),
            ],
            0,
        )
        .expect("send goal");
    thread::sleep(Duration::from_millis(50));

    client
        .send_multipart(
            [
                "act.pend".as_bytes(),
                "g-pend".as_bytes(),
                b"CANCEL".as_ref(),
                b"".as_ref(),
            ],
            0,
        )
        .expect("send cancel");
    client.set_rcvtimeo(2000).expect("rcvtimeo");
    let reply = client.recv_multipart(0).expect("recv cancelled");
    assert_eq!(reply[2], b"RESULT");
    assert!(
        reply[3].starts_with(b"CANCELLED"),
        "got {:?}",
        String::from_utf8_lossy(&reply[3])
    );

    // A late-arriving worker must not receive the cancelled goal.
    let got_goal = Arc::new(AtomicBool::new(false));
    let flag = got_goal.clone();
    let backend = broker.backend_ep.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let handle = thread::spawn(move || {
        let ctx = ZmqContext::new();
        let sock = ctx.socket(SocketType::DEALER).expect("DEALER");
        sock.set_identity(b"worker-late").expect("identity");
        sock.connect(&backend).expect("connect");
        sock.send_multipart([b"READY".as_ref(), b"act.pend".as_ref()], 0)
            .expect("READY");
        sock.set_rcvtimeo(400).expect("rcvtimeo");
        while !stop2.load(Ordering::Acquire) {
            match sock.recv_multipart(0) {
                Ok(frames) if frames.len() == 5 && frames[3] == b"GOAL" => {
                    flag.store(true, Ordering::Release);
                    break;
                }
                Ok(_) | Err(zmq::Error::EAGAIN) => continue,
                Err(_) => break,
            }
        }
    });
    thread::sleep(Duration::from_millis(600));
    stop.store(true, Ordering::Release);
    let _ = handle.join();
    assert!(
        !got_goal.load(Ordering::Acquire),
        "cancelled pending goal was dispatched to a late worker"
    );
}

#[test]
fn late_result_after_worker_died_dropped() {
    let mut cfg = default_test_config();
    cfg.heartbeat_interval_ms = 100;
    cfg.heartbeat_timeout_ms = 300;
    let broker = spawn_broker(cfg);

    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::DEALER).expect("DEALER");
    sock.set_identity(b"worker-late-result").expect("identity");
    sock.connect(&broker.backend_ep).expect("connect");
    sock.send_multipart([b"READY".as_ref(), b"act.late".as_ref()], 0)
        .expect("READY");
    thread::sleep(Duration::from_millis(80));

    let client = make_client(&broker.frontend_ep);
    client
        .send_multipart(
            [
                "act.late".as_bytes(),
                "g1".as_bytes(),
                b"GOAL".as_ref(),
                b"x".as_ref(),
            ],
            0,
        )
        .expect("send goal");

    // Worker receives GOAL but does not reply yet.
    sock.set_rcvtimeo(2000).expect("rcvtimeo");
    let frames = sock.recv_multipart(0).expect("recv goal");
    assert_eq!(frames.len(), 5);
    let client_id = frames[0].clone();

    // Wait for WORKER_DIED from broker (HB eviction).
    client.set_rcvtimeo(5000).expect("rcvtimeo");
    let died = client.recv_multipart(0).expect("recv worker-died");
    assert!(died[3].starts_with(b"WORKER_DIED"));

    // Late RESULT from the dead worker must be fenced (dropped).
    sock.send_multipart(
        [
            client_id.as_slice(),
            b"act.late".as_ref(),
            b"g1".as_ref(),
            b"RESULT".as_ref(),
            b"too-late".as_ref(),
        ],
        0,
    )
    .expect("late result");

    client.set_rcvtimeo(400).expect("rcvtimeo");
    let late = client.recv_multipart(0);
    assert!(
        late.is_err(),
        "client should not receive a late RESULT after WORKER_DIED"
    );
}

#[test]
fn recv_timeout_submits_cancel() {
    use robot_bus::action_bus::ActionClient;

    let broker = spawn_broker(default_test_config());
    let cancel_seen = Arc::new(AtomicBool::new(false));
    let flag = cancel_seen.clone();
    let backend = broker.backend_ep.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let handle = thread::spawn(move || {
        let ctx = ZmqContext::new();
        let sock = ctx.socket(SocketType::DEALER).expect("DEALER");
        sock.set_identity(b"worker-slow").expect("identity");
        sock.connect(&backend).expect("connect");
        sock.send_multipart([b"READY".as_ref(), b"act.slow".as_ref()], 0)
            .expect("READY");
        sock.set_rcvtimeo(100).expect("rcvtimeo");
        let mut client_id: Option<Vec<u8>> = None;
        while !stop2.load(Ordering::Acquire) {
            match sock.recv_multipart(0) {
                Ok(frames) if frames.len() == 5 && frames[3] == b"GOAL" => {
                    client_id = Some(frames[0].clone());
                }
                Ok(frames) if frames.len() == 5 && frames[3] == b"CANCEL" => {
                    flag.store(true, Ordering::Release);
                    if let Some(cid) = &client_id {
                        let _ = sock.send_multipart(
                            [
                                cid.as_slice(),
                                b"act.slow".as_ref(),
                                frames[2].as_slice(),
                                b"RESULT".as_ref(),
                                b"CANCELLED\0act.slow".as_ref(),
                            ],
                            0,
                        );
                    }
                    break;
                }
                Ok(_) | Err(zmq::Error::EAGAIN) => continue,
                Err(_) => break,
            }
        }
    });
    thread::sleep(Duration::from_millis(80));

    let client = ActionClient::new(Some(&broker.frontend_ep)).expect("client");
    let err = client
        .send_goal(
            "act.slow",
            b"x",
            Some("g-to"),
            Some(Duration::from_millis(300)),
        )
        .expect_err("expected timeout");
    assert!(matches!(err, robot_bus::BusError::Timeout(_)), "{err:?}");

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline && !cancel_seen.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(50));
    }
    stop.store(true, Ordering::Release);
    let _ = handle.join();
    assert!(
        cancel_seen.load(Ordering::Acquire),
        "worker did not observe CANCEL after client timeout"
    );
}

#[test]
fn duplicate_goal_id_rejected() {
    let broker = spawn_broker(default_test_config());
    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::DEALER).expect("DEALER");
    sock.set_identity(b"worker-dup").expect("identity");
    sock.connect(&broker.backend_ep).expect("connect");
    sock.send_multipart([b"READY".as_ref(), b"act.dup".as_ref()], 0)
        .expect("READY");
    thread::sleep(Duration::from_millis(80));

    let client = make_client(&broker.frontend_ep);
    client
        .send_multipart(
            [
                "act.dup".as_bytes(),
                "same".as_bytes(),
                b"GOAL".as_ref(),
                b"1".as_ref(),
            ],
            0,
        )
        .expect("first goal");
    // Hold the first goal in-flight at the worker.
    sock.set_rcvtimeo(2000).expect("rcvtimeo");
    let _ = sock.recv_multipart(0).expect("recv first goal");

    client
        .send_multipart(
            [
                "act.dup".as_bytes(),
                "same".as_bytes(),
                b"GOAL".as_ref(),
                b"2".as_ref(),
            ],
            0,
        )
        .expect("second goal");
    client.set_rcvtimeo(2000).expect("rcvtimeo");
    let reply = client.recv_multipart(0).expect("recv reject");
    assert_eq!(reply[2], b"RESULT");
    assert!(reply[3].starts_with(b"NO_GOAL"));
}
