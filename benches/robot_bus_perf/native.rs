//! Native ZMQ transport scenarios.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use robot_bus::{Context, HighWaterMark, Publisher};
use crate::support::{node_for, now_ns, LatencyStats, ScenarioResult};

use crate::pacing::{
    GoodputTrial, MSG_HWM, WARMUP, find_max_goodput, goodput_settle, goodput_trial_secs,
    make_payload, max_loss_pct, msg_latency_samples, publish_paced, publish_paced_for, read_ts,
    trial_msg_count, wait_until,
};

pub fn bench_pubsub(ctx: &Context, transport: &str) -> ScenarioResult {
    let scenario = "message pub/sub";
    let transport = transport.to_string();
    let topic = format!("perf/{transport}/msg");

    let latencies = Arc::new(Mutex::new(Vec::<u64>::with_capacity(msg_latency_samples())));
    let count = Arc::new(AtomicUsize::new(0));
    let record_latency = Arc::new(AtomicBool::new(true));

    let mut sub = node_for(ctx, format!("perf-sub-{transport}"), &transport);
    if !sub.wait_for_broker(Some(Duration::from_secs(5))) {
        return ScenarioResult::skipped(&transport, scenario, "wait_for_broker timed out");
    }
    let xsub = match sub.options().message_xsub_endpoint() {
        Ok(ep) => ep,
        Err(err) => {
            return ScenarioResult::skipped(&transport, scenario, format!("xsub endpoint: {err}"));
        }
    };
    let hwm = HighWaterMark {
        snd: MSG_HWM,
        rcv: MSG_HWM,
    };
    if let Err(err) = sub.set_stream_hwm(hwm) {
        return ScenarioResult::skipped(&transport, scenario, format!("set_stream_hwm: {err}"));
    }
    let lat_cb = Arc::clone(&latencies);
    let cnt_cb = Arc::clone(&count);
    let rec_cb = Arc::clone(&record_latency);
    if let Err(err) = sub.create_subscription_raw(
        &topic,
        Arc::new(move |payload| {
            if rec_cb.load(Ordering::Relaxed) {
                if let Some(sent) = read_ts(payload) {
                    let now = now_ns();
                    if now >= sent {
                        lat_cb.lock().unwrap().push(now - sent);
                    }
                }
            }
            cnt_cb.fetch_add(1, Ordering::Relaxed);
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

    let publisher = match Publisher::with_shared_context(ctx, Some(&xsub), hwm) {
        Ok(p) => p,
        Err(err) => {
            return ScenarioResult::skipped(
                &transport,
                scenario,
                format!("publisher failed: {err}"),
            );
        }
    };

    let count_pub = Arc::clone(&count);
    let lat_pub = Arc::clone(&latencies);
    let rec_pub = Arc::clone(&record_latency);
    let topic_pub = topic.clone();
    let transport_pub = transport.clone();
    let latency_samples = msg_latency_samples();
    let settle = goodput_settle();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(250));
        for _ in 0..WARMUP {
            let _ = publisher.publish(&topic_pub, &make_payload(now_ns()));
        }
        // Warmup may drop under modest HWM; do not require full receive.
        thread::sleep(Duration::from_millis(100));
        count_pub.store(0, Ordering::Relaxed);
        lat_pub.lock().unwrap().clear();

        // Phase 1: paced one-way latency (send → wait for that receive).
        rec_pub.store(true, Ordering::Relaxed);
        for _ in 0..latency_samples {
            let before = count_pub.load(Ordering::Relaxed);
            if publisher
                .publish(&topic_pub, &make_payload(now_ns()))
                .is_err()
            {
                shutdown.shutdown();
                return Err("publish failed (latency phase)".to_string());
            }
            if !wait_until(&count_pub, before + 1, Duration::from_secs(2)) {
                shutdown.shutdown();
                return Err("latency sample timed out".to_string());
            }
        }
        let latency = LatencyStats::from_ns(lat_pub.lock().unwrap().clone());

        // Phase 2: binary-search max goodput at loss ≤ threshold.
        rec_pub.store(false, Ordering::Relaxed);
        let goodput = match find_max_goodput(&transport_pub, |rate_hz| {
            count_pub.store(0, Ordering::Relaxed);
            let trial_secs = Duration::from_secs_f64(goodput_trial_secs());
            let (sent, send_elapsed) = match trial_msg_count(rate_hz) {
                Some(n) => publish_paced(&publisher, &topic_pub, n, rate_hz as f64),
                None => publish_paced_for(&publisher, &topic_pub, rate_hz as f64, trial_secs),
            };
            let received_at_send_end = count_pub.load(Ordering::Relaxed);
            thread::sleep(settle);
            let received = count_pub.load(Ordering::Relaxed);
            Ok(GoodputTrial {
                target_hz: rate_hz,
                sent,
                received_at_send_end,
                received,
                elapsed: send_elapsed,
            })
        }) {
            Ok(g) => g,
            Err(note) => {
                shutdown.shutdown();
                return Ok(ScenarioResult::skipped(&transport_pub, scenario, note));
            }
        };
        shutdown.shutdown();

        println!(
            "  … {transport_pub} max goodput ≈ {} Hz target (loss≤{:.1}%)",
            goodput.target_hz,
            max_loss_pct()
        );

        Ok(ScenarioResult::ok_message(
            &transport_pub,
            scenario,
            goodput.sent,
            goodput.received_at_send_end,
            goodput.received,
            goodput.elapsed,
            latency,
        ))
    });

    let _ = sub.spin();
    match worker.join().expect("publisher thread") {
        Ok(result) => result,
        Err(note) => ScenarioResult::skipped(&transport, scenario, note),
    }
}

pub fn bench_service(ctx: &Context, transport: &str, n: usize) -> ScenarioResult {
    let scenario = "service call";
    let transport = transport.to_string();
    let name = format!("perf.{transport}.echo");

    let mut server = node_for(ctx, format!("perf-svc-srv-{transport}"), &transport);
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
    let ctx_cli = ctx.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let mut client_node = node_for(
            &ctx_cli,
            format!("perf-svc-cli-{transport_cli}"),
            &transport_cli,
        );
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
        if let Err(err) = client.call(&payload, Some(Duration::from_millis(500))) {
            shutdown.shutdown();
            return ScenarioResult::skipped(
                &transport_cli,
                scenario,
                format!("call failed: {err}"),
            );
        }
        for _ in 1..WARMUP {
            let _ = client.call(&payload, Some(Duration::from_secs(2)));
        }

        let mut samples = Vec::with_capacity(n);
        let t0 = Instant::now();
        let mut received = 0usize;
        for _ in 0..n {
            let start = Instant::now();
            match client.call(&payload, Some(Duration::from_secs(5))) {
                Ok(_) => {
                    samples.push(start.elapsed().as_nanos() as u64);
                    received += 1;
                }
                Err(err) => {
                    shutdown.shutdown();
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
            n,
            received,
            elapsed,
            LatencyStats::from_ns(samples),
        )
    });

    let _ = server.spin();
    worker.join().expect("service client thread")
}

pub fn bench_action(ctx: &Context, transport: &str, n: usize) -> ScenarioResult {
    let scenario = "action send_goal";
    let transport = transport.to_string();
    let name = format!("perf.{transport}.act");

    let mut server = node_for(ctx, format!("perf-act-srv-{transport}"), &transport);
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
    let ctx_cli = ctx.clone();
    let worker = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let mut client_node = node_for(
            &ctx_cli,
            format!("perf-act-cli-{transport_cli}"),
            &transport_cli,
        );
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
        for _ in 1..WARMUP.min(10) {
            let _ = client.send_goal_and_wait(&payload, None, Some(Duration::from_secs(2)));
        }

        let mut samples = Vec::with_capacity(n);
        let t0 = Instant::now();
        let mut received = 0usize;
        for _ in 0..n {
            let start = Instant::now();
            match client.send_goal_and_wait(&payload, None, Some(Duration::from_secs(5))) {
                Ok(_) => {
                    samples.push(start.elapsed().as_nanos() as u64);
                    received += 1;
                }
                Err(err) => {
                    shutdown.shutdown();
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
            n,
            received,
            elapsed,
            LatencyStats::from_ns(samples),
        )
    });

    let _ = server.spin();
    worker.join().expect("action client thread")
}
