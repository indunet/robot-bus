//! Same-process inproc requires a shared [`robot_bus::Context`].

mod support;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
use robot_bus::{
    ActionKind, ActionMessage, Context, HighWaterMark, Node, NodeOptions, Publisher,
    RobotBusBroker, RobotBusConfig,
};
use support::lock_brokers;

fn inproc_broker_config() -> RobotBusConfig {
    // Ephemeral TCP/gRPC ports — defaults (e.g. :15570) collide under parallel cargo test.
    let mut config = support::ephemeral_robot_bus_config();
    // Keep inproc (+ ipc/tcp) so Node::inproc_with_context can reach the proxy.
    config.message.bind_all_transports = true;
    config.service.bind_all_transports = true;
    config.action.bind_all_transports = true;
    #[cfg(feature = "console")]
    {
        config.console = ConsoleBrokerConfig {
            enabled: false,
            ..ConsoleBrokerConfig::default()
        };
    }
    config
}

#[test]
fn inproc_pubsub_with_shared_context() {
    let _guard = lock_brokers();
    let ctx = Context::new();
    let broker =
        RobotBusBroker::start_with_context(&ctx, inproc_broker_config()).expect("broker");
    thread::sleep(Duration::from_millis(150));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = Arc::clone(&hits);

    let mut sub = Node::inproc_with_context(&ctx, "inproc-sub");
    sub.set_stream_hwm(HighWaterMark {
        snd: 1000,
        rcv: 1000,
    })
    .expect("set_stream_hwm");
    sub.create_subscription_raw(
        "/inproc/demo",
        Arc::new(move |_topic, payload| {
            assert_eq!(payload, b"hello-inproc");
            hits_cb.fetch_add(1, Ordering::SeqCst);
        }),
        None,
    )
    .expect("subscribe");

    let shutdown = sub.shutdown_handle().expect("shutdown handle");
    let xsub = NodeOptions::inproc().message_xsub_endpoint().expect("xsub");
    let pub_ctx = ctx.clone();
    let hits_wait = Arc::clone(&hits);
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let publisher = Publisher::with_shared_context(
            &pub_ctx,
            Some(&xsub),
            HighWaterMark {
                snd: 1000,
                rcv: 1000,
            },
        )
        .expect("publisher");
        let deadline = Instant::now() + Duration::from_secs(5);
        while hits_wait.load(Ordering::SeqCst) == 0 {
            if Instant::now() >= deadline {
                break;
            }
            let _ = publisher.publish("/inproc/demo", b"hello-inproc");
            thread::sleep(Duration::from_millis(20));
        }
        shutdown.shutdown();
    });

    let _ = sub.spin();
    worker.join().expect("publisher thread");
    assert!(
        hits.load(Ordering::SeqCst) > 0,
        "expected at least one inproc message"
    );

    broker.stop().expect("stop broker");
}

#[test]
fn inproc_action_goal_handle() {
    let _guard = lock_brokers();
    let ctx = Context::new();
    let broker =
        RobotBusBroker::start_with_context(&ctx, inproc_broker_config()).expect("broker");
    thread::sleep(Duration::from_millis(150));

    let mut server = Node::inproc_with_context(&ctx, "inproc-action-server");
    server
        .create_action_server_raw(
            "/inproc/action",
            Arc::new(|body| {
                vec![
                    ("FEEDBACK".into(), [b"step:", body].concat()),
                    ("RESULT".into(), [b"done:", body].concat()),
                ]
            }),
            None,
        )
        .expect("create_action_server_raw");
    server.start().expect("start");
    thread::sleep(Duration::from_millis(100));

    let mut client_node = Node::inproc_with_context(&ctx, "inproc-action-client");
    let client = client_node
        .create_action_client_raw("/inproc/action")
        .expect("create_action_client_raw");
    let feedbacks = Arc::new(Mutex::new(Vec::new()));
    let callback_feedbacks = Arc::clone(&feedbacks);
    let goal = client
        .send_goal(
            b"move",
            None,
            Some(Duration::from_secs(3)),
            Some(Arc::new(move |message: &ActionMessage| {
                callback_feedbacks
                    .lock()
                    .expect("feedback mutex")
                    .push(message.body.clone());
            })),
        )
        .expect("send_goal");
    assert_eq!(goal.action_name(), "/inproc/action");
    assert!(!goal.goal_id().is_empty());
    let result = goal.wait_result().expect("wait_result");
    assert_eq!(result.kind, ActionKind::Result);
    assert_eq!(result.body, b"done:move");
    assert_eq!(
        *feedbacks.lock().expect("feedback mutex"),
        vec![b"step:move".to_vec()]
    );

    server.shutdown().expect("shutdown");
    server.stop().expect("stop");
    server.wait().expect("wait");
    broker.stop().expect("stop broker");
}
