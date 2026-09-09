//! Trigger service and Fibonacci action benches (ROS ↔ bus).

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use prost::Message as ProstMessage;
use rclrs::IntoPrimitiveOptions;
use robot_bus::example_interfaces::action::v1::{FibonacciGoal, FibonacciResult};
use robot_bus::ros2_bridge::vendor::std_srvs::srv as ros_std_srvs;
use robot_bus::std_srvs::srv::v1::{TriggerRequest, TriggerResponse};
use robot_bus::{Context, Node, NodeOptions};

use crate::config::{MSG_HWM, rpc_qos};
use crate::pacing::{run_pub_trial, spin_bus};
use crate::support::{ScenarioResult, now_ns};

struct NoopWake;
impl std::task::Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

pub(crate) fn bench_ros_to_bus_trigger(
    ros_node: &rclrs::Node,
    bus_ctx: &Context,
) -> ScenarioResult {
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
        let _ = ros_client.call_then(
            ros_std_srvs::Trigger_Request::default(),
            move |resp: ros_std_srvs::Trigger_Response| {
                let _ = tx.send(resp.success);
            },
        );
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

pub(crate) fn bench_bus_to_ros_trigger(
    ros_node: &rclrs::Node,
    bus_ctx: &Context,
) -> ScenarioResult {
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

pub(crate) fn bench_ros_to_bus_fibonacci(
    ros_node: &rclrs::Node,
    bus_ctx: &Context,
) -> ScenarioResult {
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

pub(crate) fn bench_bus_to_ros_fibonacci(
    ros_node: &rclrs::Node,
    bus_ctx: &Context,
) -> ScenarioResult {
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
                executing.succeeded_with(ros_env::example_interfaces::action::Fibonacci_Result {
                    sequence: vec![0, 1, 1],
                })
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
