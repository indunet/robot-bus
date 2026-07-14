//! Integration tests for the service bus broker.
//!
//! Each test spins up a real broker (in-process, ephemeral ports) plus a
//! client (REQ) and worker(s) (DEALER), then verifies end-to-end routing.

use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::ServiceBusBroker;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;
use zmq::{Context as ZmqContext, SocketType};

/// Serialize broker starts: `bind_all` uses fixed inproc/ipc channel names.
static BROKER_LOCK: Mutex<()> = Mutex::new(());

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

/// Spawn a broker via [`ServiceBusBroker::start`] on ephemeral TCP ports.
struct BrokerHandle {
    _guard: MutexGuard<'static, ()>,
    frontend_ep: String,
    backend_ep: String,
    _broker: ServiceBusBroker,
}

fn spawn_broker(mut config: ServiceBusConfig) -> BrokerHandle {
    let guard = BROKER_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    config.frontend_bind = format!("tcp://127.0.0.1:{}", free_port());
    config.backend_bind = format!("tcp://127.0.0.1:{}", free_port());
    let frontend_ep = config.frontend_bind.clone();
    let backend_ep = config.backend_bind.clone();
    let broker = ServiceBusBroker::start(config).expect("start ServiceBusBroker");
    BrokerHandle {
        _guard: guard,
        frontend_ep,
        backend_ep,
        _broker: broker,
    }
}

/// A minimal worker: connect DEALER, send READY, then loop replying to
/// any request it receives. `tag` is prepended to the response body so
/// the test can identify which worker answered.
struct WorkerHandle {
    handle: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

fn spawn_worker(backend_ep: &str, service: &str, tag: &str) -> WorkerHandle {
    let tag = tag.to_string();
    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::DEALER).expect("create DEALER");
    let id = format!("worker-{tag}");
    sock.set_identity(id.as_bytes()).expect("set identity");
    sock.connect(backend_ep).expect("connect backend");

    sock.send_multipart([b"READY".as_ref(), service.as_bytes()], 0)
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
            if frames.len() != 4 {
                continue;
            }
            let client_id = &frames[0];
            let svc = &frames[1];
            let req_id = &frames[2];
            let body = &frames[3];
            let resp_body = format!("{}:{}", tag, String::from_utf8_lossy(body)).into_bytes();
            sock.send_multipart(
                [
                    client_id.as_slice(),
                    svc.as_slice(),
                    req_id.as_slice(),
                    resp_body.as_slice(),
                ],
                0,
            )
            .expect("worker send reply");
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

/// Send a request and wait for a reply (with timeout).
fn request_reply(client: &zmq::Socket, svc: &str, req_id: &str, body: &str) -> Vec<Vec<u8>> {
    client
        .send_multipart(
            [svc.as_bytes(), req_id.as_bytes(), body.as_bytes()],
            0,
        )
        .expect("client send");
    client.set_rcvtimeo(2000).expect("set rcvtimeo");
    client.recv_multipart(0).expect("client recv")
}

fn make_client(frontend_ep: &str) -> zmq::Socket {
    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::REQ).expect("create REQ");
    sock.connect(frontend_ep).expect("connect frontend");
    sock
}

fn default_test_config() -> ServiceBusConfig {
    ServiceBusConfig {
        heartbeat_interval_ms: 200,
        heartbeat_timeout_ms: 600,
        ..ServiceBusConfig::default()
    }
}

#[test]
fn e2e_request_reply() {
    let broker = spawn_broker(default_test_config());
    let _worker = spawn_worker(&broker.backend_ep, "svc.x", "A");
    let client = make_client(&broker.frontend_ep);

    let reply = request_reply(&client, "svc.x", "req-1", "ping");
    // reply: [svc][req_id][body]
    assert_eq!(reply.len(), 3);
    assert_eq!(reply[0], b"svc.x");
    assert_eq!(reply[1], b"req-1");
    assert_eq!(reply[2], b"A:ping".to_vec());
}

#[test]
fn route_by_service_name() {
    let broker = spawn_broker(default_test_config());
    let _w1 = spawn_worker(&broker.backend_ep, "svc.a", "AA");
    let _w2 = spawn_worker(&broker.backend_ep, "svc.b", "BB");
    let client = make_client(&broker.frontend_ep);

    let reply_a = request_reply(&client, "svc.a", "r1", "hello");
    assert_eq!(reply_a[2], b"AA:hello".to_vec());

    let reply_b = request_reply(&client, "svc.b", "r2", "world");
    assert_eq!(reply_b[2], b"BB:world".to_vec());
}

#[test]
fn no_worker_returns_error() {
    let mut cfg = default_test_config();
    cfg.heartbeat_interval_ms = 50;
    let broker = spawn_broker(cfg);
    let client = make_client(&broker.frontend_ep);

    // No worker registered. The broker queues the request; the pending
    // timeout (5s) fires and returns NO_WORKER.
    client
        .send_multipart(
            ["svc.none".as_bytes(), "r1".as_bytes(), "body".as_bytes()],
            0,
        )
        .expect("send");
    client.set_rcvtimeo(8000).expect("rcvtimeo");
    let reply = client.recv_multipart(0).expect("recv");
    assert_eq!(reply.len(), 3);
    assert_eq!(reply[0], b"svc.none");
    assert_eq!(reply[1], b"r1");
    assert!(reply[2].starts_with(b"NO_WORKER"));
    assert!(reply[2].ends_with(b"svc.none"));
}

#[test]
fn ready_registers_multiple_workers_same_service() {
    let broker = spawn_broker(default_test_config());
    let _w1 = spawn_worker(&broker.backend_ep, "svc.m", "M1");
    let _w2 = spawn_worker(&broker.backend_ep, "svc.m", "M2");
    let client = make_client(&broker.frontend_ep);

    let mut tags = std::collections::HashSet::new();
    for i in 0..4 {
        let reply = request_reply(&client, "svc.m", &format!("r{i}"), "x");
        let body = String::from_utf8_lossy(&reply[2]).into_owned();
        let tag = body.split(':').next().unwrap();
        tags.insert(tag.to_string());
    }
    assert!(tags.contains("M1"), "M1 never got a request: {tags:?}");
    assert!(tags.contains("M2"), "M2 never got a request: {tags:?}");
}

#[test]
fn heartbeat_timeout_evicts_worker() {
    let mut cfg = default_test_config();
    cfg.heartbeat_interval_ms = 100;
    cfg.heartbeat_timeout_ms = 300;
    let broker = spawn_broker(cfg);

    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::DEALER).expect("DEALER");
    sock.set_identity(b"worker-dying").expect("identity");
    sock.connect(&broker.backend_ep).expect("connect");
    sock.send_multipart([b"READY".as_ref(), b"svc.die".as_ref()], 0)
        .expect("READY");
    thread::sleep(Duration::from_millis(80));

    let client = make_client(&broker.frontend_ep);
    client.set_rcvtimeo(2000).expect("rcvtimeo");
    client
        .send_multipart(
            ["svc.die".as_bytes(), "r1".as_bytes(), "body".as_bytes()],
            0,
        )
        .expect("send");
    // The worker is not replying; this will timeout.
    let _ = client.recv_multipart(0);

    // Wait past eviction timeout
    thread::sleep(Duration::from_millis(500));

    // New request should now hit NO_WORKER after pending timeout
    let client2 = make_client(&broker.frontend_ep);
    client2.set_rcvtimeo(8000).expect("rcvtimeo");
    client2
        .send_multipart(
            ["svc.die".as_bytes(), "r2".as_bytes(), "body".as_bytes()],
            0,
        )
        .expect("send");
    let reply = client2.recv_multipart(0).expect("recv after eviction");
    assert!(reply[2].starts_with(b"NO_WORKER"));
}

#[test]
fn multiple_services_concurrent() {
    let broker = spawn_broker(default_test_config());
    let _w1 = spawn_worker(&broker.backend_ep, "svc.p", "P");
    let _w2 = spawn_worker(&broker.backend_ep, "svc.q", "Q");
    let client = make_client(&broker.frontend_ep);

    let r1 = request_reply(&client, "svc.p", "1", "a");
    let r2 = request_reply(&client, "svc.q", "2", "b");
    let r3 = request_reply(&client, "svc.p", "3", "c");

    assert_eq!(r1[2], b"P:a".to_vec());
    assert_eq!(r2[2], b"Q:b".to_vec());
    assert_eq!(r3[2], b"P:c".to_vec());
}
