use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use robot_bus::{Context, MultiThreadedExecutor, Node, NodeOptions, QosProfile, RobotBusBroker};

use crate::common::{ephemeral_robot_bus_config, lock_brokers};

const WAIT: Duration = Duration::from_secs(4);

fn receive(node: &mut Node, rx: &mpsc::Receiver<Vec<u8>>) -> Vec<u8> {
    let deadline = Instant::now() + WAIT;
    loop {
        if let Ok(payload) = rx.try_recv() {
            return payload;
        }
        assert!(Instant::now() < deadline, "topic callback timed out");
        node.spin_once(Some(Duration::from_millis(50))).unwrap();
    }
}

fn subscribe(node: &mut Node, topic: &str, qos: QosProfile) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    node.create_subscription_raw_with_qos(
        topic,
        qos,
        Arc::new(move |bytes| {
            let _ = tx.send(bytes.to_vec());
        }),
        None,
    )
    .unwrap();
    rx
}

#[test]
fn transient_local_late_join_targets_only_the_new_subscription() {
    let _guard = lock_brokers();
    let context = Context::new();
    let mut config = ephemeral_robot_bus_config();
    config.message.bind_all_transports = true;
    let _broker = RobotBusBroker::start_with_context(&context, config).unwrap();
    let qos = QosProfile::keep_last(4).transient_local();
    let topic = "/state/pose";
    let mut publisher_node = Node::inproc_with_context(&context, "state-writer");
    let publisher = publisher_node
        .create_publisher_raw_with_qos(topic, qos)
        .unwrap();
    publisher.publish(b"old").unwrap();
    publisher.publish(b"current").unwrap();

    let mut first = Node::inproc_with_context(&context, "first-reader");
    let first_rx = subscribe(&mut first, topic, qos);
    assert_eq!(receive(&mut first, &first_rx), b"current");

    let mut volatile = Node::inproc_with_context(&context, "volatile-reader");
    let volatile_rx = subscribe(&mut volatile, topic, QosProfile::DEFAULT);
    let mut second = Node::inproc_with_context(&context, "second-reader");
    let second_rx = subscribe(&mut second, topic, qos);
    assert_eq!(receive(&mut second, &second_rx), b"current");
    first.spin_once(Some(Duration::from_millis(50))).unwrap();
    volatile.spin_once(Some(Duration::from_millis(50))).unwrap();
    assert!(first_rx.try_recv().is_err());
    assert!(volatile_rx.try_recv().is_err());

    publisher.publish(b"updated").unwrap();
    assert_eq!(receive(&mut first, &first_rx), b"updated");
    assert_eq!(receive(&mut second, &second_rx), b"updated");
    assert_eq!(receive(&mut volatile, &volatile_rx), b"updated");

    drop(publisher);
    let mut late = Node::inproc_with_context(&context, "after-publisher-drop");
    let late_rx = subscribe(&mut late, topic, qos);
    late.spin_once(Some(Duration::from_millis(100))).unwrap();
    assert!(late_rx.try_recv().is_err());
}

#[test]
fn transient_local_replays_after_tcp_broker_restart() {
    let _guard = lock_brokers();
    let mut config = ephemeral_robot_bus_config();
    let broker = RobotBusBroker::start(config.clone()).unwrap();
    config.message.xsub_bind = format!("tcp://127.0.0.1:{}", broker.discover.message_xsub_port);
    config.message.xpub_bind = format!("tcp://127.0.0.1:{}", broker.discover.message_xpub_port);
    let options = NodeOptions {
        message_xsub: Some(config.message.xsub_bind.clone()),
        message_xpub: Some(config.message.xpub_bind.clone()),
        ..NodeOptions::tcp()
    };
    let qos = QosProfile::keep_last(2).transient_local();
    let mut publisher_node = Node::with_options("state-writer", options.clone());
    let publisher = publisher_node
        .create_publisher_raw_with_qos("/state", qos)
        .unwrap();
    publisher.publish(b"before").unwrap();
    let mut subscriber = Node::with_options("state-reader", options);
    let rx = subscribe(&mut subscriber, "/state", qos);
    assert_eq!(receive(&mut subscriber, &rx), b"before");

    broker.stop().unwrap();
    publisher.publish(b"while disconnected").unwrap();
    let _restarted = RobotBusBroker::start(config).unwrap();
    assert_eq!(receive(&mut subscriber, &rx), b"while disconnected");
}

#[test]
fn native_keep_last_replaces_waiting_callbacks_while_consumer_is_running() {
    let _guard = lock_brokers();
    for (multi_threaded, background) in [(false, false), (true, false), (true, true)] {
        let context = Context::new();
        let mut config = ephemeral_robot_bus_config();
        config.message.bind_all_transports = true;
        config.message.snd_hwm = 128;
        config.message.rcv_hwm = 128;
        let _broker = RobotBusBroker::start_with_context(&context, config).unwrap();
        let mut publisher_node = Node::inproc_with_context(&context, "burst-writer");
        let publisher = publisher_node.create_publisher_raw("/burst").unwrap();
        let mut subscriber = Node::inproc_with_context(&context, "slow-reader");
        let executor = MultiThreadedExecutor::with_context(context.clone(), 2);
        if multi_threaded {
            executor.add_node(&mut subscriber).unwrap();
        }
        let (observed_tx, observed_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        subscriber
            .create_subscription_raw_with_qos(
                "/burst",
                QosProfile::keep_last(2),
                Arc::new(move |payload| {
                    observed_tx.send(payload[0]).unwrap();
                    if payload[0] == 0 {
                        release_rx.lock().unwrap().recv_timeout(WAIT).unwrap();
                    }
                }),
                None,
            )
            .unwrap();
        let shutdown = subscriber.shutdown_handle().unwrap();
        let spin = if background {
            subscriber.start().unwrap();
            None
        } else {
            Some(thread::spawn(move || subscriber.spin().unwrap()))
        };
        thread::sleep(Duration::from_millis(80));
        publisher.publish(&[0]).unwrap();
        assert_eq!(observed_rx.recv_timeout(WAIT).unwrap(), 0);
        for sample in 1..=10 {
            publisher.publish(&[sample]).unwrap();
            thread::sleep(Duration::from_millis(5));
        }
        thread::sleep(Duration::from_millis(40));
        release_tx.send(()).unwrap();
        let remaining = [
            observed_rx.recv_timeout(WAIT).unwrap(),
            observed_rx.recv_timeout(WAIT).unwrap(),
        ];
        shutdown.shutdown();
        if let Some(spin) = spin {
            spin.join().unwrap();
        } else {
            executor.wait().unwrap();
        }
        assert_eq!(remaining, [9, 10]);
        assert!(observed_rx.try_recv().is_err());
    }
}
