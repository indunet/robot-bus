//! Native ZMQ transport scenarios.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::pacing::{TrialReceiver, WARMUP, make_payload, measure_pubsub, msg_hwm, rpc_warmup};
use crate::support::{LatencyStats, ScenarioResult, env_usize};
use robot_bus::{
    Context, HighWaterMark, MultiThreadedExecutor, Node, NodeOptions, Publisher, QosProfile,
};

pub fn bench_pubsub(
    ctx: &Context,
    transport: &str,
    options: &NodeOptions,
    cdr: bool,
) -> ScenarioResult {
    if std::env::var("ROBOT_BUS_PERF_TOPIC_QOS").as_deref() == Ok("transient-local") {
        return bench_retained(ctx, transport, options, cdr);
    }
    let scenario = "message pub/sub";
    let sub = Node::with_context_options(ctx, format!("perf-sub-{transport}"), options.clone());
    if !sub.wait_for_broker(Some(Duration::from_secs(5))) {
        return ScenarioResult::skipped(transport, scenario, "wait_for_broker timed out");
    }
    let xsub = match sub.options().message_xsub_endpoint() {
        Ok(ep) => ep,
        Err(err) => {
            return ScenarioResult::skipped(transport, scenario, format!("xsub endpoint: {err}"));
        }
    };
    let hwm = HighWaterMark {
        snd: msg_hwm(),
        rcv: msg_hwm(),
    };
    let publisher = match Publisher::with_shared_context(ctx, Some(&xsub), hwm) {
        Ok(p) => p,
        Err(err) => {
            return ScenarioResult::skipped(
                transport,
                scenario,
                format!("publisher failed: {err}"),
            );
        }
    };
    run_pubsub(
        transport,
        scenario,
        sub,
        publisher,
        Duration::from_millis(250),
        TrialReceiver::with_cdr(cdr),
    )
}

fn bench_retained(
    ctx: &Context,
    transport: &str,
    options: &NodeOptions,
    cdr: bool,
) -> ScenarioResult {
    let scenario = "transient-local topic";
    let topic = format!("perf/{transport}/state");
    let qos = QosProfile::keep_last(msg_hwm()).transient_local();
    let mut producer = Node::with_context_options(ctx, "perf-state-publisher", options.clone());
    let publisher = match producer.create_publisher_raw_with_qos(&topic, qos) {
        Ok(publisher) => publisher,
        Err(err) => return ScenarioResult::skipped(transport, scenario, err.to_string()),
    };
    // This write happens before the subscriber exists.
    let snapshot = b"retained probe";
    publisher.publish(snapshot).expect("publish retained probe");
    let mut sub = Node::with_context_options(ctx, "perf-state-subscriber", options.clone());
    let hwm = HighWaterMark::new(msg_hwm(), msg_hwm());
    sub.set_stream_hwm(hwm).expect("set stream HWM");
    // Connect the reader once so the replay measurement reflects subscription
    // propagation instead of native Node's initial socket setup delay.
    let warmup = sub
        .create_subscription_raw("perf/state/connect", Arc::new(|_| {}), None)
        .expect("connect state subscriber");
    sub.destroy_subscription(warmup)
        .expect("remove warmup subscription");
    let receiver = Arc::new(TrialReceiver::with_cdr(cdr));
    let (tx, rx) = std::sync::mpsc::channel();
    let started = Instant::now();
    let handle = sub
        .create_subscription_raw_with_qos(
            &topic,
            qos,
            Arc::new({
                let receiver = Arc::clone(&receiver);
                move |payload| {
                    if payload == snapshot {
                        let _ = tx.send(started.elapsed());
                    } else {
                        receiver.receive(payload);
                    }
                }
            }),
            None,
        )
        .expect("subscribe to retained topic");
    let elapsed = loop {
        if let Ok(elapsed) = rx.try_recv() {
            break elapsed;
        }
        if started.elapsed() > Duration::from_secs(4) {
            return ScenarioResult::skipped(transport, scenario, "late-join replay timed out");
        }
        sub.spin_once(Some(Duration::from_millis(50)))
            .expect("spin for retained probe");
    };
    println!(
        "RAW replay {}",
        serde_json::json!({"transport": transport, "subscribe_to_replay_us": elapsed.as_secs_f64() * 1e6})
    );
    let shutdown = sub.shutdown_handle().expect("shutdown handle");
    let worker = thread::spawn({
        let transport = transport.to_string();
        move || {
            let result = measure_pubsub(&transport, scenario, &receiver, |payload| {
                publisher.publish(payload)
            });
            shutdown.shutdown();
            result
        }
    });
    sub.spin().expect("spin retained subscriber");
    let result = worker.join().expect("state publisher thread");
    sub.destroy_subscription(handle)
        .expect("destroy subscription");
    result
}

/// Pub on `publisher` (one broker) and sub on `sub` (possibly another broker).
pub fn run_pubsub(
    transport: &str,
    scenario: &'static str,
    mut sub: Node,
    publisher: Publisher,
    after_subscribe: Duration,
    receiver: TrialReceiver,
) -> ScenarioResult {
    let transport = transport.to_string();
    let topic = format!("perf/{transport}/msg");

    let workers = env_usize("ROBOT_BUS_PERF_TOPIC_THREADS", 0);
    let _executor = if workers == 0 {
        None
    } else {
        let executor = MultiThreadedExecutor::with_context(sub.context().clone(), workers);
        if let Err(err) = executor.add_node(&mut sub) {
            return ScenarioResult::skipped(&transport, scenario, format!("add topic node: {err}"));
        }
        Some(executor)
    };

    let receiver = Arc::new(receiver);
    let hwm = HighWaterMark {
        snd: msg_hwm(),
        rcv: msg_hwm(),
    };
    if let Err(err) = sub.set_stream_hwm(hwm) {
        return ScenarioResult::skipped(&transport, scenario, format!("set_stream_hwm: {err}"));
    }
    if let Err(err) = sub.create_subscription_raw(
        &topic,
        Arc::new({
            let receiver = Arc::clone(&receiver);
            move |payload| receiver.receive(payload)
        }),
        None,
    ) {
        return ScenarioResult::skipped(&transport, scenario, format!("subscribe failed: {err}"));
    }

    let shutdown = match sub.shutdown_handle() {
        Ok(h) => h,
        Err(err) => {
            return ScenarioResult::skipped(
                &transport,
                scenario,
                format!("shutdown handle: {err}"),
            );
        }
    };

    let worker = thread::spawn(move || {
        thread::sleep(after_subscribe);
        let result = measure_pubsub(&transport, scenario, &receiver, |payload| {
            publisher.publish(&topic, payload)
        });
        shutdown.shutdown();
        result
    });

    let _ = sub.spin();
    worker.join().expect("publisher thread")
}

pub fn bench_service(
    ctx: &Context,
    transport: &str,
    options: &NodeOptions,
    n: usize,
) -> ScenarioResult {
    let scenario = "service call";
    let name = format!("perf.{transport}.echo");
    let server =
        Node::with_context_options(ctx, format!("perf-svc-srv-{transport}"), options.clone());
    let options = options.clone();
    let ctx_cli = ctx.clone();
    let transport_cli = transport.to_string();
    run_service(
        transport,
        scenario,
        &name,
        server,
        Duration::from_millis(200),
        n,
        move || {
            Node::with_context_options(&ctx_cli, format!("perf-svc-cli-{transport_cli}"), options)
        },
    )
}

/// Server on one node; client factory may pin a different broker (federation).
pub fn run_service<F>(
    transport: &str,
    scenario: &'static str,
    name: &str,
    mut server: Node,
    advertise_wait: Duration,
    n: usize,
    make_client: F,
) -> ScenarioResult
where
    F: FnOnce() -> Node + Send + 'static,
{
    let transport = transport.to_string();
    let name = name.to_string();
    if let Err(err) = server.create_service_raw(&name, Arc::new(|body| body.to_vec()), None) {
        return ScenarioResult::skipped(&transport, scenario, format!("create_service: {err}"));
    }

    let shutdown = match server.shutdown_handle() {
        Ok(h) => h,
        Err(err) => {
            return ScenarioResult::skipped(
                &transport,
                scenario,
                format!("shutdown handle: {err}"),
            );
        }
    };

    let transport_cli = transport.clone();
    let worker = thread::spawn(move || {
        thread::sleep(advertise_wait);
        let mut client_node = make_client();
        let client = match client_node.create_client_raw(&name) {
            Ok(c) => c,
            Err(err) => {
                shutdown.shutdown();
                return ScenarioResult::skipped(
                    &transport_cli,
                    scenario,
                    format!("create_client: {err}"),
                );
            }
        };

        let payload = make_payload(0);
        // Fail fast: one probe before full warmup.
        if let Err(err) = client.call(&payload, Some(Duration::from_secs(2))) {
            shutdown.shutdown();
            return ScenarioResult::skipped(
                &transport_cli,
                scenario,
                format!("call failed: {err}"),
            );
        }
        for _ in 1..rpc_warmup(WARMUP) {
            let _ = client.call(&payload, Some(Duration::from_secs(2)));
        }

        let mut samples = Vec::with_capacity(n);
        let t0 = Instant::now();
        let mut received = 0usize;
        let mut attempted = 0;
        for _ in 0..n {
            attempted += 1;
            let start = Instant::now();
            match client.call(&payload, Some(Duration::from_secs(5))) {
                Ok(_) => {
                    samples.push(start.elapsed().as_nanos() as u64);
                    received += 1;
                }
                Err(err) => {
                    shutdown.shutdown();
                    eprintln!("{scenario} attempt {attempted} failed: {err}");
                    if received == 0 {
                        return ScenarioResult::skipped(
                            &transport_cli,
                            scenario,
                            format!("call failed: {err}"),
                        );
                    }
                    break;
                }
            }
        }
        let elapsed = t0.elapsed();
        shutdown.shutdown();

        if received == 0 {
            return ScenarioResult::skipped(&transport_cli, scenario, "0 successful calls");
        }

        ScenarioResult::ok_rpc(
            &transport_cli,
            scenario,
            attempted,
            received,
            elapsed,
            LatencyStats::from_ns(samples),
        )
    });

    let _ = server.spin();
    worker.join().expect("service client thread")
}

pub fn bench_action(
    ctx: &Context,
    transport: &str,
    options: &NodeOptions,
    n: usize,
) -> ScenarioResult {
    let scenario = "action send_goal";
    let name = format!("perf.{transport}.act");
    let server =
        Node::with_context_options(ctx, format!("perf-act-srv-{transport}"), options.clone());
    let options = options.clone();
    let ctx_cli = ctx.clone();
    let transport_cli = transport.to_string();
    run_action(
        transport,
        scenario,
        &name,
        server,
        Duration::from_millis(200),
        n,
        move || {
            Node::with_context_options(&ctx_cli, format!("perf-act-cli-{transport_cli}"), options)
        },
    )
}

/// Action server on one node; client factory may pin a different broker (federation).
pub fn run_action<F>(
    transport: &str,
    scenario: &'static str,
    name: &str,
    mut server: Node,
    advertise_wait: Duration,
    n: usize,
    make_client: F,
) -> ScenarioResult
where
    F: FnOnce() -> Node + Send + 'static,
{
    let transport = transport.to_string();
    let name = name.to_string();
    if let Err(err) = server.create_action_server_raw(
        &name,
        Arc::new(|body| {
            vec![
                ("FEEDBACK".into(), b"f".to_vec()),
                ("RESULT".into(), body.to_vec()),
            ]
        }),
        None,
    ) {
        return ScenarioResult::skipped(
            &transport,
            scenario,
            format!("create_action_server: {err}"),
        );
    }

    let shutdown = match server.shutdown_handle() {
        Ok(h) => h,
        Err(err) => {
            return ScenarioResult::skipped(
                &transport,
                scenario,
                format!("shutdown handle: {err}"),
            );
        }
    };

    let transport_cli = transport.clone();
    let worker = thread::spawn(move || {
        thread::sleep(advertise_wait);
        let mut client_node = make_client();
        let client = match client_node.create_action_client_raw(&name) {
            Ok(c) => c,
            Err(err) => {
                shutdown.shutdown();
                return ScenarioResult::skipped(
                    &transport_cli,
                    scenario,
                    format!("create_action_client: {err}"),
                );
            }
        };

        let payload = make_payload(0);
        if let Err(err) =
            client.send_goal_and_wait(&payload, None, Some(Duration::from_millis(800)))
        {
            shutdown.shutdown();
            return ScenarioResult::skipped(
                &transport_cli,
                scenario,
                format!("send_goal failed: {err}"),
            );
        }
        for _ in 1..rpc_warmup(WARMUP.min(10)) {
            let _ = client.send_goal_and_wait(&payload, None, Some(Duration::from_secs(2)));
        }

        let mut samples = Vec::with_capacity(n);
        let t0 = Instant::now();
        let mut received = 0usize;
        let mut attempted = 0;
        for _ in 0..n {
            attempted += 1;
            let start = Instant::now();
            match client.send_goal_and_wait(&payload, None, Some(Duration::from_secs(5))) {
                Ok(_) => {
                    samples.push(start.elapsed().as_nanos() as u64);
                    received += 1;
                }
                Err(err) => {
                    shutdown.shutdown();
                    eprintln!("{scenario} attempt {attempted} failed: {err}");
                    if received == 0 {
                        return ScenarioResult::skipped(
                            &transport_cli,
                            scenario,
                            format!("send_goal failed: {err}"),
                        );
                    }
                    break;
                }
            }
        }
        let elapsed = t0.elapsed();
        shutdown.shutdown();

        if received == 0 {
            return ScenarioResult::skipped(&transport_cli, scenario, "0 successful goals");
        }

        ScenarioResult::ok_rpc(
            &transport_cli,
            scenario,
            attempted,
            received,
            elapsed,
            LatencyStats::from_ns(samples),
        )
    });

    let _ = server.spin();
    worker.join().expect("action client thread")
}
