//! ROS 2 ↔ robot-bus bridge performance harness.
//!
//! Run: `just perf-ros2-bridge` or
//! `cargo run --release --bin ros2_bridge_perf --features ros2`

#[cfg(feature = "ros2-shim")]
fn main() {
    eprintln!(
        "ros2_bridge_perf requires --features ros2 with a sourced ROS install \
         (not ros2-shim)"
    );
    std::process::exit(2);
}

#[cfg(not(feature = "ros2-shim"))]
fn main() {
    run::main();
}

#[cfg(not(feature = "ros2-shim"))]
#[path = "../robot_bus_perf/support.rs"]
mod support;

#[cfg(not(feature = "ros2-shim"))]
mod run {
    use super::support;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    use prost::Message as ProstMessage;
    use rclrs::{CreateBasicExecutor, IntoPrimitiveOptions, SpinOptions};
    use robot_bus::example_interfaces::action::v1::{FibonacciGoal, FibonacciResult};
    use robot_bus::ros2_bridge::vendor::std_srvs::srv as ros_std_srvs;
    use robot_bus::ros2_bridge::{
        FibonacciActionMapper, Ros2Bridge, SensorMsgsImageMapper, StdMsgsStringMapper, TopicQos,
        TriggerServiceMapper,
    };
    use robot_bus::std_msgs::msg::v1::String as BusString;
    use robot_bus::std_srvs::srv::v1::{TriggerRequest, TriggerResponse};
    use robot_bus::{Context, Node, NodeOptions, QosProfile, RobotBusBroker, ShutdownHandle};
    use ros_env::sensor_msgs::msg::Image as RosImage;
    use ros_env::std_msgs::msg::String as RosString;
    use support::{
        LatencyStats, ScenarioResult, env_f64, env_summary, env_usize, lock_broker, now_ns,
        perf_broker_config,
    };

    const STRING_LEN: usize = 64;
    const MSG_HWM: i32 = 2_048;
    const WARMUP: usize = 20;

    fn image_width() -> u32 {
        env_usize("ROS2_BRIDGE_PERF_IMAGE_WIDTH", 640) as u32
    }

    fn image_height() -> u32 {
        env_usize("ROS2_BRIDGE_PERF_IMAGE_HEIGHT", 480) as u32
    }

    fn max_loss_pct() -> f64 {
        env_f64("ROS2_BRIDGE_PERF_MAX_LOSS_PCT", 1.0)
    }

    fn goodput_trial_secs() -> f64 {
        env_f64("ROS2_BRIDGE_PERF_GOODPUT_TRIAL_SECS", 1.0)
    }

    fn goodput_rate_lo() -> u64 {
        env_usize("ROS2_BRIDGE_PERF_GOODPUT_RATE_LO", 50) as u64
    }

    fn goodput_rate_hi() -> u64 {
        env_usize("ROS2_BRIDGE_PERF_GOODPUT_RATE_HI", 50_000) as u64
    }

    fn msg_latency_samples() -> usize {
        env_usize("ROS2_BRIDGE_PERF_MSG_LATENCY_SAMPLES", 200)
    }

    fn goodput_settle() -> Duration {
        Duration::from_millis(env_usize("ROS2_BRIDGE_PERF_GOODPUT_SETTLE_MS", 100) as u64)
    }

    fn string_payload(ts_ns: u64) -> String {
        format!("{ts_ns:016x}{}", "x".repeat(STRING_LEN.saturating_sub(16)))
    }

    fn parse_ts(data: &str) -> Option<u64> {
        u64::from_str_radix(data.get(..16)?, 16).ok()
    }

    fn wait_until(count: &AtomicUsize, target: usize, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while count.load(Ordering::Relaxed) < target {
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(1));
        }
        true
    }

    fn loss_pct(sent: usize, received: usize) -> f64 {
        if sent == 0 {
            100.0
        } else if received >= sent {
            0.0
        } else {
            100.0 * (sent - received) as f64 / sent as f64
        }
    }

    fn wait_deadline(deadline: Instant) {
        loop {
            let now = Instant::now();
            if now >= deadline {
                return;
            }
            let remain = deadline - now;
            if remain > Duration::from_millis(2) {
                thread::sleep(remain - Duration::from_millis(1));
            } else {
                std::hint::spin_loop();
            }
        }
    }

    struct GoodputTrial {
        target_hz: u64,
        sent: usize,
        received_at_send_end: usize,
        received: usize,
        elapsed: Duration,
    }

    fn trial_sustains_rate(t: &GoodputTrial) -> bool {
        let secs = t.elapsed.as_secs_f64().max(1e-9);
        let pub_rate = t.sent as f64 / secs;
        let sub_rate = t.received_at_send_end as f64 / secs;
        let target = t.target_hz as f64;
        pub_rate >= 0.90 * target && sub_rate >= 0.90 * target
    }

    fn find_max_goodput(
        label: &str,
        mut trial: impl FnMut(u64) -> Result<GoodputTrial, String>,
    ) -> Result<GoodputTrial, String> {
        let max_loss = max_loss_pct();
        let mut lo = goodput_rate_lo();
        let mut hi = goodput_rate_hi().max(lo);
        let mut best: Option<GoodputTrial> = None;
        let rate_lo = lo;
        let rate_hi = hi;
        println!("  … {label} max goodput: binary search {lo}..={hi} Hz, loss≤{max_loss:.1}%");
        while lo <= hi {
            let mid = lo + (hi - lo) / 2;
            let t = trial(mid)?;
            let loss = loss_pct(t.sent, t.received);
            let sustained = trial_sustains_rate(&t);
            println!(
                "  …   try {mid} Hz → sent={} recv_send={} recv_final={} loss={loss:.2}% sustained={sustained}",
                t.sent, t.received_at_send_end, t.received
            );
            if t.received > 0 && loss <= max_loss && sustained {
                best = Some(t);
                lo = mid.saturating_add(1);
            } else if mid == 0 {
                break;
            } else {
                hi = mid - 1;
            }
        }
        best.ok_or_else(|| {
            format!(
                "no rate in {rate_lo}..={rate_hi} Hz met loss≤{max_loss:.1}% at ≥90% of target pace"
            )
        })
    }

    fn make_image(ts_ns: u64) -> RosImage {
        let w = image_width();
        let h = image_height();
        let mut data = vec![0u8; (w * h * 3) as usize];
        data[..8].copy_from_slice(&ts_ns.to_le_bytes());
        RosImage {
            header: Default::default(),
            height: h,
            width: w,
            encoding: "rgb8".into(),
            is_bigendian: 0,
            step: w * 3,
            data,
        }
    }

    fn image_ts(data: &[u8]) -> Option<u64> {
        if data.len() < 8 {
            return None;
        }
        let mut b = [0u8; 8];
        b.copy_from_slice(&data[..8]);
        Some(u64::from_le_bytes(b))
    }

    pub fn main() {
        let _guard = lock_broker();
        let only = std::env::var("ROS2_BRIDGE_PERF_ONLY")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let run_string = only.is_empty() || only == "string" || only == "str";
        let run_image = only.is_empty() || only == "image" || only == "img";
        let run_trigger = only.is_empty() || only == "trigger" || only == "svc";
        let run_fibonacci = only.is_empty() || only == "fibonacci" || only == "fib";

        println!("starting RobotBusBroker…");
        let ctx = Context::new();
        let broker =
            RobotBusBroker::start_with_context(&ctx, perf_broker_config()).expect("broker");
        thread::sleep(Duration::from_millis(300));

        let qos_ros = TopicQos::keep_last(MSG_HWM).best_effort();
        let qos_bus = TopicQos::keep_last(MSG_HWM).best_effort();
        let mut bridge = Ros2Bridge::new("ros2_bridge_perf")
            .bus_tcp("localhost")
            .from_ros("/perf/r2b/str", qos_ros)
            .to_bus("/perf/r2b/str", qos_bus)
            .mapper(StdMsgsStringMapper)
            .add()
            .expect("r2b str")
            .from_bus("/perf/b2r/str", qos_bus)
            .to_ros("/perf/b2r/str", qos_ros)
            .mapper(StdMsgsStringMapper)
            .add()
            .expect("b2r str")
            .from_ros("/perf/r2b/img", qos_ros)
            .to_bus("/perf/r2b/img", qos_bus)
            .mapper(SensorMsgsImageMapper)
            .add()
            .expect("r2b img")
            .from_bus("/perf/b2r/img", qos_bus)
            .to_ros("/perf/b2r/img", qos_ros)
            .mapper(SensorMsgsImageMapper)
            .add()
            .expect("b2r img")
            .service()
            .from_ros("/perf/r2b/trigger", qos_ros)
            .to_bus("/perf/r2b/trigger", qos_bus)
            .mapper(TriggerServiceMapper)
            .add()
            .expect("r2b trigger")
            .service()
            .from_bus("/perf/b2r/trigger", qos_bus)
            .to_ros("/perf/b2r/trigger", qos_ros)
            .mapper(TriggerServiceMapper)
            .add()
            .expect("b2r trigger")
            .action()
            .from_ros("/perf/r2b/fib", qos_ros)
            .to_bus("/perf/r2b/fib", qos_bus)
            .mapper(FibonacciActionMapper)
            .add()
            .expect("r2b fib")
            .action()
            .from_bus("/perf/b2r/fib", qos_bus)
            .to_ros("/perf/b2r/fib", qos_ros)
            .mapper(FibonacciActionMapper)
            .add()
            .expect("b2r fib")
            .build()
            .expect("bridge build");

        let bridge_thread = thread::Builder::new()
            .name("bridge_spin".into())
            .spawn(move || {
                let _ = bridge.spin();
            })
            .expect("bridge thread");

        let ros_ctx = rclrs::Context::default_from_env().expect("rclrs context");
        let mut ros_exec = ros_ctx.create_basic_executor();
        let ros_node = ros_exec
            .create_node("ros2_bridge_perf_peer")
            .expect("ros node");
        let ros_commands = std::sync::Arc::clone(ros_exec.commands());
        let ros_thread = thread::Builder::new()
            .name("ros_peer_spin".into())
            .spawn(move || {
                let _ = ros_exec.spin(SpinOptions::default());
            })
            .expect("ros spin");

        thread::sleep(Duration::from_millis(400));

        let mut results = Vec::new();
        if run_string {
            results.push(bench_ros_to_bus_string(&ros_node, &ctx));
            results.push(bench_bus_to_ros_string(&ros_node, &ctx));
        }
        if run_image {
            results.push(bench_ros_to_bus_image(&ros_node, &ctx));
            results.push(bench_bus_to_ros_image(&ros_node, &ctx));
        }
        if run_trigger {
            results.push(bench_ros_to_bus_trigger(&ros_node, &ctx));
            results.push(bench_bus_to_ros_trigger(&ros_node, &ctx));
        }
        if run_fibonacci {
            results.push(bench_ros_to_bus_fibonacci(&ros_node, &ctx));
            results.push(bench_bus_to_ros_fibonacci(&ros_node, &ctx));
        }

        ros_commands.halt_spinning();
        let _ = ros_thread.join();
        drop(bridge_thread);
        broker.stop().expect("stop broker");

        for r in &results {
            if let Some(note) = &r.note {
                println!("[{}/{}] SKIP: {note}", r.transport, r.scenario);
            } else {
                println!(
                    "[{}/{}] sent={} recv={} pub={:.0}/s sub={:.0}/s delivery={:.1}% p50={:.0}µs p99={:.0}µs",
                    r.transport,
                    r.scenario,
                    r.sent,
                    r.received,
                    r.publish_per_s,
                    r.subscribe_per_s,
                    r.delivery_pct,
                    r.latency.p50_us,
                    r.latency.p99_us,
                );
            }
        }

        write_reports(&results).expect("write reports");
    }

    fn spin_bus(mut node: Node) -> (thread::JoinHandle<()>, ShutdownHandle) {
        let shutdown = node.shutdown_handle().expect("shutdown handle");
        let handle = thread::spawn(move || {
            let _ = node.spin();
        });
        (handle, shutdown)
    }

    fn bench_ros_to_bus_string(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
        let scenario = "string ROS→bus";
        let topic = "/perf/r2b/str";
        let count = Arc::new(AtomicUsize::new(0));
        let latencies = Arc::new(Mutex::new(Vec::<u64>::new()));
        let record = Arc::new(AtomicBool::new(true));

        let mut bus = Node::with_context_options(bus_ctx, "perf_bus_sub_str", NodeOptions::tcp());
        let cnt = Arc::clone(&count);
        let lat = Arc::clone(&latencies);
        let rec = Arc::clone(&record);
        if let Err(err) = bus.create_subscription_raw_with_qos(
            topic,
            QosProfile::keep_last(MSG_HWM),
            Arc::new(move |payload| {
                if rec.load(Ordering::Relaxed) {
                    if let Ok(msg) = BusString::decode(payload) {
                        if let Some(sent) = parse_ts(&msg.data) {
                            let now = now_ns();
                            if now >= sent {
                                lat.lock().unwrap().push(now - sent);
                            }
                        }
                    }
                }
                cnt.fetch_add(1, Ordering::Relaxed);
            }),
            None,
        ) {
            return ScenarioResult::skipped("inproc", scenario, format!("bus sub: {err}"));
        }
        let (_spin, shutdown) = spin_bus(bus);

        let ros_pub = match ros_node
            .create_publisher::<RosString>(topic.keep_last(MSG_HWM as u32).best_effort())
        {
            Ok(p) => p,
            Err(err) => {
                shutdown.shutdown();
                return ScenarioResult::skipped("inproc", scenario, format!("ros pub: {err}"));
            }
        };

        run_pub_trial(scenario, count, latencies, record, shutdown, move |ts| {
            let msg = RosString {
                data: string_payload(ts),
            };
            ros_pub.publish(msg).map_err(|e| e.to_string())
        })
    }

    fn bench_bus_to_ros_string(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
        let scenario = "string bus→ROS";
        let topic = "/perf/b2r/str";
        let count = Arc::new(AtomicUsize::new(0));
        let latencies = Arc::new(Mutex::new(Vec::<u64>::new()));
        let record = Arc::new(AtomicBool::new(true));
        let cnt = Arc::clone(&count);
        let lat = Arc::clone(&latencies);
        let rec = Arc::clone(&record);

        let _ros_sub = match ros_node.create_subscription::<RosString, _>(
            topic.keep_last(MSG_HWM as u32).best_effort(),
            move |msg: RosString| {
                if rec.load(Ordering::Relaxed) {
                    if let Some(sent) = parse_ts(&msg.data) {
                        let now = now_ns();
                        if now >= sent {
                            lat.lock().unwrap().push(now - sent);
                        }
                    }
                }
                cnt.fetch_add(1, Ordering::Relaxed);
            },
        ) {
            Ok(s) => s,
            Err(err) => {
                return ScenarioResult::skipped("inproc", scenario, format!("ros sub: {err}"));
            }
        };

        let mut bus = Node::with_context_options(bus_ctx, "perf_bus_pub_str", NodeOptions::tcp());
        let pub_ = match bus.create_publisher_raw_with_qos(topic, QosProfile::keep_last(MSG_HWM)) {
            Ok(p) => p,
            Err(err) => {
                return ScenarioResult::skipped("inproc", scenario, format!("bus pub: {err}"));
            }
        };
        let (_spin, shutdown) = spin_bus(bus);

        run_pub_trial(scenario, count, latencies, record, shutdown, move |ts| {
            let payload = BusString {
                data: string_payload(ts),
            }
            .encode_to_vec();
            pub_.publish(&payload).map_err(|e| e.to_string())
        })
    }

    fn bench_ros_to_bus_image(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
        let scenario = format!("image {}x{} ROS→bus", image_width(), image_height());
        let topic = "/perf/r2b/img";
        let count = Arc::new(AtomicUsize::new(0));
        let latencies = Arc::new(Mutex::new(Vec::<u64>::new()));
        let record = Arc::new(AtomicBool::new(true));

        let mut bus = Node::with_context_options(bus_ctx, "perf_bus_sub_img", NodeOptions::tcp());
        let cnt = Arc::clone(&count);
        let lat = Arc::clone(&latencies);
        let rec = Arc::clone(&record);
        if let Err(err) = bus.create_subscription_raw_with_qos(
            topic,
            QosProfile::keep_last(MSG_HWM),
            Arc::new(move |payload| {
                if rec.load(Ordering::Relaxed) {
                    if let Ok(msg) = robot_bus::sensor_msgs::msg::v1::Image::decode(payload) {
                        if let Some(sent) = image_ts(&msg.data) {
                            let now = now_ns();
                            if now >= sent {
                                lat.lock().unwrap().push(now - sent);
                            }
                        }
                    }
                }
                cnt.fetch_add(1, Ordering::Relaxed);
            }),
            None,
        ) {
            return ScenarioResult::skipped("inproc", &scenario, format!("bus sub: {err}"));
        }
        let (_spin, shutdown) = spin_bus(bus);

        let ros_pub = match ros_node
            .create_publisher::<RosImage>(topic.keep_last(MSG_HWM as u32).best_effort())
        {
            Ok(p) => p,
            Err(err) => {
                shutdown.shutdown();
                return ScenarioResult::skipped("inproc", &scenario, format!("ros pub: {err}"));
            }
        };

        run_pub_trial(&scenario, count, latencies, record, shutdown, move |ts| {
            ros_pub.publish(make_image(ts)).map_err(|e| e.to_string())
        })
    }

    fn bench_bus_to_ros_image(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
        let scenario = format!("image {}x{} bus→ROS", image_width(), image_height());
        let topic = "/perf/b2r/img";
        let count = Arc::new(AtomicUsize::new(0));
        let latencies = Arc::new(Mutex::new(Vec::<u64>::new()));
        let record = Arc::new(AtomicBool::new(true));
        let cnt = Arc::clone(&count);
        let lat = Arc::clone(&latencies);
        let rec = Arc::clone(&record);

        let _ros_sub = match ros_node.create_subscription::<RosImage, _>(
            topic.keep_last(MSG_HWM as u32).best_effort(),
            move |msg: RosImage| {
                if rec.load(Ordering::Relaxed) {
                    if let Some(sent) = image_ts(&msg.data) {
                        let now = now_ns();
                        if now >= sent {
                            lat.lock().unwrap().push(now - sent);
                        }
                    }
                }
                cnt.fetch_add(1, Ordering::Relaxed);
            },
        ) {
            Ok(s) => s,
            Err(err) => {
                return ScenarioResult::skipped("inproc", &scenario, format!("ros sub: {err}"));
            }
        };

        let mut bus = Node::with_context_options(bus_ctx, "perf_bus_pub_img", NodeOptions::tcp());
        let pub_ = match bus.create_publisher_raw_with_qos(topic, QosProfile::keep_last(MSG_HWM)) {
            Ok(p) => p,
            Err(err) => {
                return ScenarioResult::skipped("inproc", &scenario, format!("bus pub: {err}"));
            }
        };
        let (_spin, shutdown) = spin_bus(bus);

        run_pub_trial(&scenario, count, latencies, record, shutdown, move |ts| {
            let img = make_image(ts);
            let payload = robot_bus::sensor_msgs::msg::v1::Image {
                header: None,
                height: img.height,
                width: img.width,
                encoding: img.encoding.to_string(),
                is_bigendian: img.is_bigendian != 0,
                step: img.step,
                data: img.data.iter().copied().collect(),
            }
            .encode_to_vec();
            pub_.publish(&payload).map_err(|e| e.to_string())
        })
    }

    fn rpc_qos() -> QosProfile {
        QosProfile::keep_last(MSG_HWM)
    }

    struct NoopWake;
    impl std::task::Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    fn bench_ros_to_bus_trigger(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
        let scenario = "trigger ROS→bus";
        let name = "/perf/r2b/trigger";
        let count = Arc::new(AtomicUsize::new(0));
        let latencies = Arc::new(Mutex::new(Vec::<u64>::new()));
        let record = Arc::new(AtomicBool::new(true));

        let mut bus = Node::with_context_options(bus_ctx, "perf_bus_svc_r2b", NodeOptions::tcp());
        if let Err(err) = bus.create_service_raw_with_qos(
            name,
            rpc_qos(),
            Arc::new(|_| {
                TriggerResponse {
                    success: true,
                    message: String::new(),
                }
                .encode_to_vec()
            }),
            None,
        ) {
            return ScenarioResult::skipped("inproc", scenario, format!("bus svc: {err}"));
        }
        let (_spin, shutdown) = spin_bus(bus);

        let ros_client = match ros_node
            .create_client::<ros_std_srvs::Trigger>(name.keep_last(MSG_HWM as u32).best_effort())
        {
            Ok(c) => c,
            Err(err) => {
                shutdown.shutdown();
                return ScenarioResult::skipped("inproc", scenario, format!("ros client: {err}"));
            }
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if ros_client.service_is_ready().unwrap_or(false) {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let cnt = Arc::clone(&count);
        let lat = Arc::clone(&latencies);
        let rec = Arc::clone(&record);
        run_pub_trial(scenario, count, latencies, record, shutdown, move |_ts| {
            let t0 = now_ns();
            let (tx, rx) = mpsc::sync_channel(1);
            let _ = ros_client.call_then(ros_std_srvs::Trigger_Request::default(), move |resp| {
                let _ = tx.send(resp.success);
            });
            match rx.recv_timeout(Duration::from_secs(2)) {
                Ok(_) => {
                    if rec.load(Ordering::Relaxed) {
                        let now = now_ns();
                        if now >= t0 {
                            lat.lock().unwrap().push(now - t0);
                        }
                    }
                    cnt.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
                Err(_) => Err("trigger ROS→bus timed out".into()),
            }
        })
    }

    fn bench_bus_to_ros_trigger(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
        let scenario = "trigger bus→ROS";
        let name = "/perf/b2r/trigger";
        let count = Arc::new(AtomicUsize::new(0));
        let latencies = Arc::new(Mutex::new(Vec::<u64>::new()));
        let record = Arc::new(AtomicBool::new(true));

        let _ros_svc = match ros_node.create_service::<ros_std_srvs::Trigger, _>(
            name.keep_last(MSG_HWM as u32).best_effort(),
            |_req: ros_std_srvs::Trigger_Request| ros_std_srvs::Trigger_Response {
                success: true,
                message: String::new(),
            },
        ) {
            Ok(s) => s,
            Err(err) => {
                return ScenarioResult::skipped("inproc", scenario, format!("ros svc: {err}"));
            }
        };

        let mut bus = Node::with_context_options(bus_ctx, "perf_bus_cli_b2r", NodeOptions::tcp());
        let client = match bus.create_client_raw_with_qos(name, rpc_qos()) {
            Ok(c) => c,
            Err(err) => {
                return ScenarioResult::skipped("inproc", scenario, format!("bus client: {err}"));
            }
        };
        let (_spin, shutdown) = spin_bus(bus);
        thread::sleep(Duration::from_millis(200));

        let cnt = Arc::clone(&count);
        let lat = Arc::clone(&latencies);
        let rec = Arc::clone(&record);
        let req = TriggerRequest {}.encode_to_vec();
        run_pub_trial(scenario, count, latencies, record, shutdown, move |_ts| {
            let t0 = now_ns();
            client
                .call(&req, Some(Duration::from_secs(2)))
                .map(|_| {
                    if rec.load(Ordering::Relaxed) {
                        let now = now_ns();
                        if now >= t0 {
                            lat.lock().unwrap().push(now - t0);
                        }
                    }
                    cnt.fetch_add(1, Ordering::Relaxed);
                })
                .map_err(|e| e.to_string())
        })
    }

    fn bench_ros_to_bus_fibonacci(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
        let scenario = "fibonacci ROS→bus";
        let name = "/perf/r2b/fib";
        let count = Arc::new(AtomicUsize::new(0));
        let latencies = Arc::new(Mutex::new(Vec::<u64>::new()));
        let record = Arc::new(AtomicBool::new(true));

        let mut bus = Node::with_context_options(bus_ctx, "perf_bus_act_r2b", NodeOptions::tcp());
        if let Err(err) = bus.create_action_server_raw_with_qos(
            name,
            rpc_qos(),
            Arc::new(|_body| {
                vec![(
                    "RESULT".into(),
                    FibonacciResult {
                        sequence: vec![0, 1, 1],
                    }
                    .encode_to_vec(),
                )]
            }),
            None,
        ) {
            return ScenarioResult::skipped("inproc", scenario, format!("bus action: {err}"));
        }
        let (_spin, shutdown) = spin_bus(bus);

        let ros_client = match ros_node
            .create_action_client::<ros_env::example_interfaces::action::Fibonacci>(name)
        {
            Ok(c) => c,
            Err(err) => {
                shutdown.shutdown();
                return ScenarioResult::skipped(
                    "inproc",
                    scenario,
                    format!("ros action client: {err}"),
                );
            }
        };

        let cnt = Arc::clone(&count);
        let lat = Arc::clone(&latencies);
        let rec = Arc::clone(&record);
        run_pub_trial(scenario, count, latencies, record, shutdown, move |_ts| {
            let t0 = now_ns();
            let goal = ros_env::example_interfaces::action::Fibonacci_Goal { order: 3 };
            let requested = ros_client
                .try_request_goal(goal)
                .map_err(|e| format!("request_goal: {e}"))?;
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut fut = requested;
            let waker = std::task::Waker::from(std::sync::Arc::new(NoopWake));
            let mut cx = std::task::Context::from_waker(&waker);
            let goal_client = loop {
                match std::pin::Pin::new(&mut fut).poll(&mut cx) {
                    std::task::Poll::Ready(v) => break v,
                    std::task::Poll::Pending => {
                        if Instant::now() >= deadline {
                            return Err("fibonacci ROS→bus accept timed out".into());
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                }
            };
            let Some(gc) = goal_client else {
                return Err("fibonacci ROS→bus rejected".into());
            };
            let mut result_fut = gc.result;
            loop {
                match std::pin::Pin::new(&mut result_fut).poll(&mut cx) {
                    std::task::Poll::Ready(_) => break,
                    std::task::Poll::Pending => {
                        if Instant::now() >= deadline {
                            return Err("fibonacci ROS→bus result timed out".into());
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                }
            }
            if rec.load(Ordering::Relaxed) {
                let now = now_ns();
                if now >= t0 {
                    lat.lock().unwrap().push(now - t0);
                }
            }
            cnt.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })
    }

    fn bench_bus_to_ros_fibonacci(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
        let scenario = "fibonacci bus→ROS";
        let name = "/perf/b2r/fib";
        let count = Arc::new(AtomicUsize::new(0));
        let latencies = Arc::new(Mutex::new(Vec::<u64>::new()));
        let record = Arc::new(AtomicBool::new(true));

        let _ros_server = match ros_node
            .create_action_server::<ros_env::example_interfaces::action::Fibonacci, _>(
                name,
                |requested| async move {
                    let accepted = requested.accept();
                    let executing = match accepted.begin() {
                        rclrs::BeginAcceptedGoal::Execute(e) => e,
                        rclrs::BeginAcceptedGoal::Cancel(c) => {
                            return c.cancelled_with(Default::default());
                        }
                    };
                    executing.succeeded_with(
                        ros_env::example_interfaces::action::Fibonacci_Result {
                            sequence: vec![0, 1, 1],
                        },
                    )
                },
            ) {
            Ok(s) => s,
            Err(err) => {
                return ScenarioResult::skipped(
                    "inproc",
                    scenario,
                    format!("ros action server: {err}"),
                );
            }
        };

        let mut bus = Node::with_context_options(bus_ctx, "perf_bus_act_cli", NodeOptions::tcp());
        let client = match bus.create_action_client_raw_with_qos(name, rpc_qos()) {
            Ok(c) => c,
            Err(err) => {
                return ScenarioResult::skipped(
                    "inproc",
                    scenario,
                    format!("bus action client: {err}"),
                );
            }
        };
        let (_spin, shutdown) = spin_bus(bus);
        thread::sleep(Duration::from_millis(300));
        let goal = FibonacciGoal { order: 3 }.encode_to_vec();
        let cnt = Arc::clone(&count);
        let lat = Arc::clone(&latencies);
        let rec = Arc::clone(&record);
        run_pub_trial(scenario, count, latencies, record, shutdown, move |_ts| {
            let t0 = now_ns();
            client
                .send_goal_and_wait(&goal, None, Some(Duration::from_secs(3)))
                .map(|_| {
                    if rec.load(Ordering::Relaxed) {
                        let now = now_ns();
                        if now >= t0 {
                            lat.lock().unwrap().push(now - t0);
                        }
                    }
                    cnt.fetch_add(1, Ordering::Relaxed);
                })
                .map_err(|e| e.to_string())
        })
    }

    fn run_pub_trial(
        scenario: &str,
        count: Arc<AtomicUsize>,
        latencies: Arc<Mutex<Vec<u64>>>,
        record: Arc<AtomicBool>,
        shutdown: ShutdownHandle,
        publish: impl Fn(u64) -> Result<(), String>,
    ) -> ScenarioResult {
        thread::sleep(Duration::from_millis(250));
        for _ in 0..WARMUP {
            let _ = publish(now_ns());
        }
        thread::sleep(Duration::from_millis(100));
        count.store(0, Ordering::Relaxed);
        latencies.lock().unwrap().clear();

        record.store(true, Ordering::Relaxed);
        let samples = msg_latency_samples();
        for _ in 0..samples {
            let before = count.load(Ordering::Relaxed);
            if publish(now_ns()).is_err() {
                shutdown.shutdown();
                return ScenarioResult::skipped("inproc", scenario, "publish failed (latency)");
            }
            if !wait_until(&count, before + 1, Duration::from_secs(5)) {
                shutdown.shutdown();
                return ScenarioResult::skipped("inproc", scenario, "latency sample timed out");
            }
        }
        let latency = LatencyStats::from_ns(latencies.lock().unwrap().clone());

        record.store(false, Ordering::Relaxed);
        let settle = goodput_settle();
        let trial_secs = Duration::from_secs_f64(goodput_trial_secs());
        let goodput = match find_max_goodput(scenario, |rate_hz| {
            count.store(0, Ordering::Relaxed);
            let interval = Duration::from_secs_f64(1.0 / (rate_hz as f64).max(1.0));
            let t0 = Instant::now();
            let deadline = t0 + trial_secs;
            let mut next = t0;
            let mut sent = 0usize;
            while Instant::now() < deadline {
                if publish(now_ns()).is_err() {
                    break;
                }
                sent += 1;
                next += interval;
                wait_deadline(next);
            }
            let send_elapsed = t0.elapsed();
            let received_at_send_end = count.load(Ordering::Relaxed);
            thread::sleep(settle);
            Ok(GoodputTrial {
                target_hz: rate_hz,
                sent,
                received_at_send_end,
                received: count.load(Ordering::Relaxed),
                elapsed: send_elapsed,
            })
        }) {
            Ok(g) => g,
            Err(err) => {
                shutdown.shutdown();
                return ScenarioResult::skipped("inproc", scenario, err);
            }
        };
        shutdown.shutdown();
        ScenarioResult::ok_message(
            "inproc",
            scenario,
            goodput.sent,
            goodput.received_at_send_end,
            goodput.received,
            goodput.elapsed,
            latency,
        )
    }

    fn write_reports(results: &[ScenarioResult]) -> std::io::Result<()> {
        let root = support::env_summary();
        let env = root;
        let zh = render(results, &env, true);
        let en = render(results, &env, false);
        let repo = std::env::current_dir()?;
        std::fs::create_dir_all(repo.join("docs/zh"))?;
        std::fs::create_dir_all(repo.join("docs/en"))?;
        std::fs::write(repo.join("docs/zh/ros2-bridge-perf-report.md"), zh)?;
        std::fs::write(repo.join("docs/en/ros2-bridge-perf-report.md"), en)?;
        println!("wrote docs/zh/ros2-bridge-perf-report.md and docs/en/ros2-bridge-perf-report.md");
        Ok(())
    }

    fn render(results: &[ScenarioResult], env_lines: &[String], zh: bool) -> String {
        let mut md = String::new();
        if zh {
            md.push_str("[English](../en/ros2-bridge-perf-report.md) | 中文\n\n");
            md.push_str("# ROS 2 Bridge 性能测试报告\n\n");
            md.push_str("由 `just perf-ros2-bridge`（`ros2_bridge_perf`）生成，**不进 CI**。\n\n");
            md.push_str("## 环境\n\n");
        } else {
            md.push_str("English | [中文](../zh/ros2-bridge-perf-report.md)\n\n");
            md.push_str("# ROS 2 Bridge performance report\n\n");
            md.push_str(
                "Generated by `just perf-ros2-bridge` (`ros2_bridge_perf`). **Not a CI gate.**\n\n",
            );
            md.push_str("## Environment\n\n");
        }
        for line in env_lines {
            md.push_str(&format!("- {line}\n"));
        }
        md.push_str(&format!(
            "- Image: {}x{} rgb8\n",
            image_width(),
            image_height()
        ));
        md.push_str(&format!(
            "- String payload: {STRING_LEN} bytes; QoS KeepLast({MSG_HWM}) best_effort\n"
        ));
        if zh {
            md.push_str("\n## 方法\n\n");
            md.push_str("- 进程内 broker + `Ros2Bridge`；ROS 与 bus 各一条 peer。\n");
            md.push_str("- 场景：64B `std_msgs/String`；Image 默认 640×480 rgb8；Trigger / Fibonacci 双向（KeepLast 与 topic 档一致）。\n");
            md.push_str("- 吞吐：限速发送约 1s，二分搜索丢包 ≤ 1% 的最大可持续速率。\n");
            md.push_str("- 延迟：另做限速抽样（发一条等收到再发）。\n\n");
            md.push_str("## 结果\n\n");
        } else {
            md.push_str("\n## Method\n\n");
            md.push_str("- In-process broker + `Ros2Bridge`; one ROS peer and one bus peer.\n");
            md.push_str("- Scenarios: 64B `std_msgs/String`; Image 640×480 rgb8 default; Trigger / Fibonacci both directions (KeepLast matches topic benches).\n");
            md.push_str("- Goodput: paced ~1s trials, binary search max rate with loss ≤ 1%.\n");
            md.push_str("- Latency: separate paced samples (send one, wait, repeat).\n\n");
            md.push_str("## Results\n\n");
        }
        md.push_str(
            "| scenario | sent | recv | pub/s | sub/s | delivery | p50 (µs) | p99 (µs) |\n",
        );
        md.push_str(
            "|----------|------|------|-------|-------|----------|----------|----------|\n",
        );
        for r in results {
            if r.note.is_some() {
                md.push_str(&format!(
                    "| {} | — | — | — | — | SKIP: {} |\n",
                    r.scenario,
                    r.note.as_deref().unwrap_or("")
                ));
            } else {
                md.push_str(&format!(
                    "| {} | {} | {} | {:.0} | {:.0} | {:.1}% | {:.0} | {:.0} |\n",
                    r.scenario,
                    r.sent,
                    r.received,
                    r.publish_per_s,
                    r.subscribe_per_s,
                    r.delivery_pct,
                    r.latency.p50_us,
                    r.latency.p99_us,
                ));
            }
        }
        md.push('\n');
        md
    }
}
