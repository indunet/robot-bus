//! ROS 2 ↔ robot-bus bridge performance harness.
//!
//! Sources live under `benches/ros2_bridge_perf/`.
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
#[path = "../robot_bus_perf/support.rs"]
mod support;

#[cfg(not(feature = "ros2-shim"))]
mod config;
#[cfg(not(feature = "ros2-shim"))]
mod pacing;
#[cfg(not(feature = "ros2-shim"))]
mod report;
#[cfg(not(feature = "ros2-shim"))]
mod rpc;
#[cfg(not(feature = "ros2-shim"))]
mod topics;

#[cfg(not(feature = "ros2-shim"))]
fn main() {
    use std::thread;
    use std::time::Duration;

    use rclrs::{CreateBasicExecutor, SpinOptions};
    use robot_bus::ros2_bridge::{
        FibonacciActionMapper, Ros2Bridge, SensorMsgsImageMapper, StdMsgsStringMapper, TopicQos,
        TriggerServiceMapper,
    };
    use robot_bus::{Context, RobotBusBroker};

    use config::MSG_HWM;
    use report::write_reports;
    use rpc::{
        bench_bus_to_ros_fibonacci, bench_bus_to_ros_trigger, bench_ros_to_bus_fibonacci,
        bench_ros_to_bus_trigger,
    };
    use support::{lock_broker, perf_broker_config};
    use topics::{
        bench_bus_to_ros_image, bench_bus_to_ros_string, bench_ros_to_bus_image,
        bench_ros_to_bus_string,
    };

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
    let broker = RobotBusBroker::start_with_context(&ctx, perf_broker_config()).expect("broker");
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
