//! Cross-language interop peer for `tests/interop/`.
//!
//! Roles used by the diversified matrix:
//!   pub         — publish Imu (scenario: Rust → Python)
//!   svc-client  — call SetBool (scenario: Java → Rust)

use std::env;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::std_srvs::srv::v1::{SetBool, SetBoolRequest};
use robot_bus::{Node, NodeOptions};

const TOPIC: &str = "/interop/imu";
const SERVICE: &str = "/interop/set_bool";
const EXPECT_Z: f64 = 0.42;

fn main() -> Result<()> {
    let role = env::var("ROBOT_BUS_INTEROP_ROLE").context("ROBOT_BUS_INTEROP_ROLE")?;
    let options = node_options_from_env()?;
    match role.as_str() {
        "pub" => run_pub(options),
        "svc-client" => run_svc_client(options),
        other => bail!("unknown ROBOT_BUS_INTEROP_ROLE: {other}"),
    }
}

fn require_env(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("missing env {key}"))
}

fn node_options_from_env() -> Result<NodeOptions> {
    Ok(NodeOptions {
        message_xsub: Some(require_env("ROBOT_BUS_MESSAGE_XSUB")?),
        message_xpub: Some(require_env("ROBOT_BUS_MESSAGE_XPUB")?),
        service_frontend: Some(require_env("ROBOT_BUS_SERVICE_FRONTEND")?),
        service_backend: Some(require_env("ROBOT_BUS_SERVICE_BACKEND")?),
        action_frontend: Some(require_env("ROBOT_BUS_ACTION_FRONTEND")?),
        action_backend: Some(require_env("ROBOT_BUS_ACTION_BACKEND")?),
        ..NodeOptions::default()
    })
}

fn run_pub(options: NodeOptions) -> Result<()> {
    let mut node = Node::with_options("interop_rust_pub", options);
    let pub_ = node
        .create_publisher::<Imu>(TOPIC)
        .context("create_publisher")?;
    thread::sleep(Duration::from_millis(400));
    for _ in 0..5 {
        pub_.publish(&Imu {
            angular_velocity: Some(Vector3 {
                x: 0.0,
                y: 0.0,
                z: EXPECT_Z,
            }),
            ..Default::default()
        })?;
        thread::sleep(Duration::from_millis(50));
    }
    println!("READY");
    Ok(())
}

fn run_svc_client(options: NodeOptions) -> Result<()> {
    thread::sleep(Duration::from_millis(400));
    let mut node = Node::with_options("interop_rust_svc_client", options);
    let client = node
        .create_client::<SetBool>(SERVICE)
        .context("create_client")?;
    let resp = client
        .call(&SetBoolRequest { data: true }, Some(Duration::from_secs(5)))
        .context("call")?;
    let ok_msg = resp.message == "set:true" || resp.message == "set:True";
    if !resp.success || !ok_msg {
        bail!("unexpected SetBool response: {resp:?}");
    }
    println!("READY");
    Ok(())
}
