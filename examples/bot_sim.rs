//! Pure Rust bot physics node.
//!
//! Subscribes `/bot1/cmd_vel` ([`Twist`]), integrates a differential-drive pose
//! on an 11×11 world, and publishes `/bot1/pose` ([`Pose2D`]) at 20 Hz.
//!
//! Pair with the console viewer (`bot_sim_viewer`) and control panel
//! (`bot_control_panel`) over a running broker:
//!
//! ```bash
//! cargo run --bin robot_bus_broker
//! cargo run --example bot_sim
//! # then open the console BOT windows /bot_sim + /bot_teleop
//! ```

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use robot_bus::geometry_msgs::msg::v1::{Pose2D, Twist};
use robot_bus::Node;

const CMD_VEL_TOPIC: &str = "/bot1/cmd_vel";
const POSE_TOPIC: &str = "/bot1/pose";
const WORLD_SIZE: f64 = 11.0;
const TICK: Duration = Duration::from_millis(50);
const CMD_TIMEOUT: Duration = Duration::from_millis(400);

struct SimState {
    x: f64,
    y: f64,
    theta: f64,
    linear: f64,
    angular: f64,
    last_cmd: Instant,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            x: WORLD_SIZE / 2.0,
            y: WORLD_SIZE / 2.0,
            theta: 0.0,
            linear: 0.0,
            angular: 0.0,
            last_cmd: Instant::now(),
        }
    }
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.clamp(min, max)
}

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("bot_sim");
    // TopicPublisher is !Send/!Sync (ZMQ socket) — keep publish on this thread.
    let pose_pub = node.create_publisher::<Pose2D>(POSE_TOPIC)?;
    let state = Arc::new(Mutex::new(SimState::default()));

    {
        let state = Arc::clone(&state);
        node.create_subscription::<Twist, _>(
            CMD_VEL_TOPIC,
            move |_topic, twist| {
                let mut s = state.lock().expect("bot_sim state");
                s.linear = twist.linear.as_ref().map(|v| v.x).unwrap_or(0.0);
                s.angular = twist.angular.as_ref().map(|v| v.z).unwrap_or(0.0);
                s.last_cmd = Instant::now();
            },
            None,
        )?;
    }

    eprintln!(
        "bot_sim online — SUB {CMD_VEL_TOPIC} → PUB {POSE_TOPIC} (tick {}ms)",
        TICK.as_millis()
    );

    let mut last_tick = Instant::now();
    loop {
        node.spin_once(Some(Duration::from_millis(5)))?;

        let now = Instant::now();
        if now.duration_since(last_tick) < TICK {
            continue;
        }
        let dt = (now - last_tick).as_secs_f64().min(0.05);
        last_tick = now;

        let pose = {
            let mut s = state.lock().expect("bot_sim state");
            if now.duration_since(s.last_cmd) > CMD_TIMEOUT {
                s.linear = 0.0;
                s.angular = 0.0;
            }

            s.theta += s.angular * dt;
            s.x = clamp(s.x + s.theta.cos() * s.linear * dt, 0.0, WORLD_SIZE);
            s.y = clamp(s.y + s.theta.sin() * s.linear * dt, 0.0, WORLD_SIZE);

            Pose2D {
                x: s.x,
                y: s.y,
                theta: s.theta,
            }
        };

        if let Err(err) = pose_pub.publish(&pose) {
            eprintln!("bot_sim: publish {POSE_TOPIC} failed: {err}");
        }
    }
}
