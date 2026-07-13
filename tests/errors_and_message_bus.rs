mod support;

use std::time::Duration;

use robot_bus::errors::{parse_error_body, BusError};
use robot_bus::message_bus::{Publisher, Subscriber};
use support::MessageProxy;

#[test]
fn parse_no_worker() {
    let err = parse_error_body(b"NO_WORKER\0svc.x").expect("error");
    assert!(matches!(err, BusError::NoWorker { name } if name == "svc.x"));
}

#[test]
fn parse_worker_died() {
    let err = parse_error_body(b"WORKER_DIED\0act.x").expect("error");
    assert!(matches!(err, BusError::WorkerDied { name } if name == "act.x"));
}

#[test]
fn parse_no_goal() {
    let err = parse_error_body(b"NO_GOAL\0g-1").expect("error");
    assert!(matches!(err, BusError::NoGoal { goal_id } if goal_id == "g-1"));
}

#[test]
fn parse_ok_body() {
    assert!(parse_error_body(b"ok").is_none());
}

#[test]
fn publish_subscribe_roundtrip() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    std::thread::sleep(Duration::from_millis(50));
    let sub = Subscriber::new(Some(&proxy.xpub_endpoint)).expect("subscriber");
    sub.subscribe("demo.topic").expect("subscribe");
    std::thread::sleep(Duration::from_millis(150));
    pub_.publish("demo.topic", b"hello").expect("publish");
    let (topic, payload) = sub
        .receive(Some(Duration::from_secs(2)))
        .expect("receive");
    assert_eq!(topic, "demo.topic");
    assert_eq!(payload, b"hello");
}

#[test]
fn topic_filter() {
    let proxy = MessageProxy::spawn();
    let pub_ = Publisher::new(Some(&proxy.xsub_endpoint)).expect("publisher");
    std::thread::sleep(Duration::from_millis(50));
    let sub = Subscriber::new(Some(&proxy.xpub_endpoint)).expect("subscriber");
    sub.subscribe("a.").expect("subscribe");
    std::thread::sleep(Duration::from_millis(150));
    pub_.publish("a.one", b"1").expect("publish a.one");
    pub_.publish("b.two", b"2").expect("publish b.two");
    let (topic, payload) = sub
        .receive(Some(Duration::from_secs(2)))
        .expect("receive");
    assert_eq!(topic, "a.one");
    assert_eq!(payload, b"1");
}
