mod support;

use std::sync::Arc;
use std::time::Duration;

use robot_bus::action_bus::{ActionClient, ActionKind};
use robot_bus::errors::BusError;
use robot_bus::worker_thread::WorkerThread;
use support::BrokerProcess;

#[test]
fn action_goal_feedback_result() {
    let broker = BrokerProcess::spawn_action();
    let handler: Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> = Arc::new(|body| {
        vec![
            ("FEEDBACK".into(), b"step-1".to_vec()),
            ("FEEDBACK".into(), b"step-2".to_vec()),
            ("RESULT".into(), [b"done:", body].concat()),
        ]
    });
    let worker =
        WorkerThread::spawn_action("act.demo", handler, &broker.backend_endpoint).expect("worker");
    std::thread::sleep(Duration::from_millis(100));
    let client = ActionClient::new(Some(&broker.frontend_endpoint)).expect("client");
    let messages = client
        .send_goal("act.demo", b"fly", None, Some(Duration::from_secs(30)))
        .expect("goal");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].kind, ActionKind::Feedback);
    assert_eq!(messages[0].body, b"step-1");
    assert_eq!(messages[1].kind, ActionKind::Feedback);
    assert_eq!(messages[2].kind, ActionKind::Result);
    assert_eq!(messages[2].body, b"done:fly");
    worker.stop();
}

#[test]
fn action_no_worker() {
    let broker = BrokerProcess::spawn_action();
    let client = ActionClient::new(Some(&broker.frontend_endpoint)).expect("client");
    let err = client
        .send_goal("act.none", b"x", None, Some(Duration::from_secs(2)))
        .expect_err("expected error");
    assert!(matches!(err, BusError::NoWorker { .. }));
}
