//! Same-process inproc requires a shared [`robot_bus::Context`].

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::common::lock_brokers;
#[cfg(feature = "console-api")]
use robot_bus::ConsoleBrokerConfig;
use robot_bus::{
    ActionKind, ActionMessage, Context, HighWaterMark, Node, NodeOptions, Publisher,
    RobotBusBroker, RobotBusConfig, Subscriber,
};

fn inproc_broker_config() -> RobotBusConfig {
    // Ephemeral TCP/gRPC ports — defaults (e.g. :15560) collide under parallel cargo test.
    let mut config = crate::common::ephemeral_robot_bus_config();
    // Keep inproc (+ ipc/tcp) so Node::inproc_with_context can reach the proxy.
    config.message.bind_all_transports = true;
    config.service.bind_all_transports = true;
    config.action.bind_all_transports = true;
    #[cfg(feature = "console-api")]
    {
        config.console = ConsoleBrokerConfig {
            enabled: false,
            ..ConsoleBrokerConfig::default()
        };
    }
    config
}

#[test]
fn broker_uses_and_cleans_up_private_ipc_directory() {
    let _guard = lock_brokers();
    let root = tempfile::tempdir().expect("IPC parent directory");
    let context = Context::new();
    let broker =
        RobotBusBroker::start_with_context_in(&context, inproc_broker_config(), Some(root.path()))
            .expect("broker");
    assert!(broker.discover.inproc_prefix.is_some());
    if !robot_bus::transports::ipc_supported() {
        assert!(broker.discover.ipc_dir.is_none());
        assert!(std::fs::read_dir(root.path()).unwrap().next().is_none());
        broker.stop().expect("stop broker");
        return;
    }

    let directory = Path::new(
        broker
            .discover
            .ipc_dir
            .as_deref()
            .expect("advertised IPC directory"),
    )
    .to_owned();
    assert_eq!(directory.parent(), Some(root.path()));
    let endpoints = broker
        .discover
        .to_announcement()
        .expect("broker announcement")
        .apply(NodeOptions::for_discovery("ipc"))
        .expect("discover IPC endpoints");
    {
        let xsub = endpoints.message_xsub_endpoint().expect("IPC XSUB");
        let xpub = endpoints.message_xpub_endpoint().expect("IPC XPUB");
        let publisher = Publisher::new(Some(&xsub)).expect("IPC publisher");
        let subscriber = Subscriber::new(Some(&xpub)).expect("IPC subscriber");
        let topic = "/ipc/tempdir";
        subscriber.subscribe(topic).expect("subscribe");
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            publisher.publish(topic, b"hello-ipc").expect("publish");
            if let Ok(received) = subscriber.receive(Some(Duration::from_millis(50))) {
                assert_eq!(received, (topic.to_owned(), b"hello-ipc".to_vec()));
                break;
            }
            assert!(Instant::now() < deadline, "IPC message was not delivered");
        }
    }
    broker.stop().expect("stop broker");
    assert!(!directory.exists());
}

#[test]
fn inproc_pubsub_with_shared_context() {
    let _guard = lock_brokers();
    let ctx = Context::new();
    let broker = RobotBusBroker::start_with_context(&ctx, inproc_broker_config()).expect("broker");
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
        Arc::new(move |payload| {
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
    let broker = RobotBusBroker::start_with_context(&ctx, inproc_broker_config()).expect("broker");
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
