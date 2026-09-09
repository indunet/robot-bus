//! String and Image topic benches (ROS ↔ bus).

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use prost::Message as ProstMessage;
use rclrs::IntoPrimitiveOptions;
use robot_bus::std_msgs::msg::v1::String as BusString;
use robot_bus::{Context, Node, NodeOptions, QosProfile};
use ros_env::sensor_msgs::msg::Image as RosImage;
use ros_env::std_msgs::msg::String as RosString;

use crate::config::{
    MSG_HWM, image_height, image_ts, image_width, make_image, parse_ts, string_payload,
};
use crate::pacing::{run_pub_trial, spin_bus};
use crate::support::{ScenarioResult, now_ns};

pub(crate) fn bench_ros_to_bus_string(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
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

pub(crate) fn bench_bus_to_ros_string(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
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

pub(crate) fn bench_ros_to_bus_image(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
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

pub(crate) fn bench_bus_to_ros_image(ros_node: &rclrs::Node, bus_ctx: &Context) -> ScenarioResult {
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
