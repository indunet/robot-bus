mod support;

use std::sync::Arc;
use std::time::Duration;

use robot_bus::errors::BusError;
use robot_bus::service_bus::ServiceClient;
use robot_bus::worker_thread::WorkerThread;
use support::BrokerProcess;

#[test]
fn service_request_reply() {
    let broker = BrokerProcess::spawn_service();
    let handler: Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|body| [b"echo:", body].concat());
    let worker = WorkerThread::spawn_service("svc.echo", handler, &broker.backend_endpoint)
        .expect("worker");
    std::thread::sleep(Duration::from_millis(100));
    let client = ServiceClient::new(Some(&broker.frontend_endpoint)).expect("client");
    let reply = client
        .call("svc.echo", b"ping", None, Some(Duration::from_secs(10)))
        .expect("call");
    assert_eq!(reply, b"echo:ping");
    worker.stop();
}

#[test]
fn service_no_worker() {
    let broker = BrokerProcess::spawn_service();
    let client = ServiceClient::new(Some(&broker.frontend_endpoint)).expect("client");
    let err = client
        .call("svc.none", b"x", None, Some(Duration::from_secs(8)))
        .expect_err("expected error");
    assert!(matches!(err, BusError::NoWorker { .. }));
}
