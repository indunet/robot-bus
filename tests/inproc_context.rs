//! Same-process inproc requires a shared [`robot_bus::Context`].

mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "console")]
use robot_bus::ConsoleBrokerConfig;
use robot_bus::{
    Context, HighWaterMark, Node, NodeOptions, Publisher, RobotBusBroker, RobotBusConfig,
};
use support::lock_brokers;

fn inproc_broker_config() -> RobotBusConfig {
    // Ephemeral TCP/gRPC ports — defaults (e.g. :15770) collide under parallel cargo test.
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
        RobotBusBroker::start_with_context(ctx.clone(), inproc_broker_config()).expect("broker");
    thread::sleep(Duration::from_millis(150));

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_cb = Arc::clone(&hits);

    let mut sub = Node::inproc_with_context(ctx.clone(), "inproc-sub");
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
