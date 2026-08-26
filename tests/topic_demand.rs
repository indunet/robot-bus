//! Immediate `/robot_bus/topic_demand` events from topology register/unregister.

#![cfg(feature = "console")]

mod support;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use prost::Message;
use robot_bus::console_topics;
use robot_bus::robot_bus_interfaces::msg::v1::{TopicDemand, TopicStatsList};
use robot_bus::{MessageCallback, Node, NodeOptions, RobotBusBroker};
use support::{ephemeral_robot_bus_config, lock_brokers};

fn console_on_config() -> robot_bus::RobotBusConfig {
    let mut config = ephemeral_robot_bus_config();
    config.console.enabled = true;
    config.console.tank_enabled = false;
    config
}

fn node_options_from_broker(broker: &RobotBusBroker) -> NodeOptions {
    NodeOptions {
        message_xsub: Some(broker.message.xsub_bind.clone()),
        message_xpub: Some(broker.message.xpub_bind.clone()),
        service_frontend: Some(broker.service.frontend_bind.clone()),
        service_backend: Some(broker.service.backend_bind.clone()),
        action_frontend: Some(broker.action.frontend_bind.clone()),
        action_backend: Some(broker.action.backend_bind.clone()),
        ..NodeOptions::default()
    }
}

fn drain(node: &mut Node, n: usize) {
    for _ in 0..n {
        let _ = node.spin_once(Some(Duration::from_millis(20)));
    }
}

fn wait_demand(
    node: &mut Node,
    seen: &Arc<Mutex<Vec<TopicDemand>>>,
    topic: &str,
    want: u32,
    timeout: Duration,
) -> TopicDemand {
    let deadline = Instant::now() + timeout;
    loop {
        {
            let guard = seen.lock().expect("lock");
            if let Some(d) = guard
                .iter()
                .rev()
                .find(|d| d.topic == topic && d.subscribers == want)
            {
                return d.clone();
            }
        }
        assert!(
            Instant::now() < deadline,
            "timeout waiting demand {topic}={want}"
        );
        let _ = node.spin_once(Some(Duration::from_millis(20)));
    }
}

#[test]
fn subscriber_register_unregister_emits_topic_demand() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(console_on_config()).expect("broker");
    let opts = node_options_from_broker(&broker);

    let seen: Arc<Mutex<Vec<TopicDemand>>> = Arc::new(Mutex::new(Vec::new()));
    let seen_cb = Arc::clone(&seen);
    let cb: MessageCallback = Arc::new(move |_t, payload| {
        if let Ok(msg) = TopicDemand::decode(payload) {
            seen_cb.lock().expect("lock").push(msg);
        }
    });

    let mut watcher = Node::with_options("demand-watch", opts.clone());
    let _h = watcher
        .create_subscription_raw(console_topics::TOPIC_DEMAND, cb, None)
        .expect("watch demand");
    drain(&mut watcher, 5);

    let mut listener = Node::with_options("demand-sub", opts.clone());
    let sub = listener
        .create_subscription_raw("/lazy/cam", Arc::new(|_, _| {}), None)
        .expect("subscribe");
    wait_demand(&mut watcher, &seen, "/lazy/cam", 1, Duration::from_secs(3));

    let mut listener2 = Node::with_options("demand-sub-2", opts.clone());
    let sub2 = listener2
        .create_subscription_raw("/lazy/cam", Arc::new(|_, _| {}), None)
        .expect("subscribe 2");
    wait_demand(&mut watcher, &seen, "/lazy/cam", 2, Duration::from_secs(3));

    listener.destroy_subscription(sub).expect("destroy 1");
    wait_demand(&mut watcher, &seen, "/lazy/cam", 1, Duration::from_secs(3));

    listener2.destroy_subscription(sub2).expect("destroy 2");
    wait_demand(&mut watcher, &seen, "/lazy/cam", 0, Duration::from_secs(3));

    broker.stop().expect("stop");
}

#[test]
fn publisher_does_not_count_as_demand() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(console_on_config()).expect("broker");
    let opts = node_options_from_broker(&broker);

    let seen: Arc<Mutex<Vec<TopicDemand>>> = Arc::new(Mutex::new(Vec::new()));
    let seen_cb = Arc::clone(&seen);
    let cb: MessageCallback = Arc::new(move |_t, payload| {
        if let Ok(msg) = TopicDemand::decode(payload) {
            seen_cb.lock().expect("lock").push(msg);
        }
    });
    let mut watcher = Node::with_options("demand-pub-watch", opts.clone());
    let _h = watcher
        .create_subscription_raw(console_topics::TOPIC_DEMAND, cb, None)
        .expect("watch");
    drain(&mut watcher, 5);

    let mut talker = Node::with_options("demand-pub", opts);
    let _pub = talker.create_publisher_raw("/lazy/only_pub").expect("pub");
    drain(&mut watcher, 20);
    drain(&mut talker, 5);

    let hits: Vec<_> = seen
        .lock()
        .expect("lock")
        .iter()
        .filter(|d| d.topic == "/lazy/only_pub")
        .cloned()
        .collect();
    assert!(
        hits.is_empty() || hits.iter().all(|d| d.subscribers == 0),
        "publisher must not look like demand: {hits:?}"
    );

    broker.stop().expect("stop");
}

#[test]
fn topics_snapshot_subscriber_count_matches_demand() {
    let _guard = lock_brokers();
    let broker = RobotBusBroker::start(console_on_config()).expect("broker");
    let opts = node_options_from_broker(&broker);

    let demand: Arc<Mutex<Vec<TopicDemand>>> = Arc::new(Mutex::new(Vec::new()));
    let demand_cb = Arc::clone(&demand);
    let snap: Arc<Mutex<Option<TopicStatsList>>> = Arc::new(Mutex::new(None));
    let snap_cb = Arc::clone(&snap);

    let mut watcher = Node::with_options("demand-snap", opts.clone());
    let _h1 = watcher
        .create_subscription_raw(
            console_topics::TOPIC_DEMAND,
            Arc::new(move |_t, payload| {
                if let Ok(msg) = TopicDemand::decode(payload) {
                    demand_cb.lock().expect("lock").push(msg);
                }
            }),
            None,
        )
        .expect("demand sub");
    let _h2 = watcher
        .create_subscription_raw(
            console_topics::TOPICS,
            Arc::new(move |_t, payload| {
                if let Ok(msg) = TopicStatsList::decode(payload) {
                    *snap_cb.lock().expect("lock") = Some(msg);
                }
            }),
            None,
        )
        .expect("topics sub");

    let mut listener = Node::with_options("demand-snap-sub", opts);
    let _sub = listener
        .create_subscription_raw("/lazy/snap", Arc::new(|_, _| {}), None)
        .expect("sub");
    wait_demand(
        &mut watcher,
        &demand,
        "/lazy/snap",
        1,
        Duration::from_secs(3),
    );

    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let _ = watcher.spin_once(Some(Duration::from_millis(50)));
        let n = snap.lock().expect("lock").as_ref().and_then(|list| {
            list.topics
                .iter()
                .find(|t| t.name == "/lazy/snap")
                .map(|t| t.subscribers)
        });
        if n == Some(1) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "topics snapshot never showed subscribers=1"
        );
    }

    broker.stop().expect("stop");
}
