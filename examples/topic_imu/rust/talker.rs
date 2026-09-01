//! Publish a few `sensor_msgs/msg/Imu` messages on `/examples/imu`.
//!
//! Run a broker (`python -m robot_bus.broker` or `cargo run --bin robot_bus_broker`) and `topic_imu_listener` first.

use std::thread;
use std::time::Duration;

use robot_bus::Node;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("examples_imu_talker");
    let pub_ = node.create_publisher::<Imu>("/examples/imu")?;
    // ZMQ slow joiner: give the listener a moment after subscribe.
    thread::sleep(Duration::from_millis(300));

    for i in 0..5 {
        let imu = Imu {
            linear_acceleration: Some(Vector3 {
                x: 0.0,
                y: 0.0,
                z: 9.8 + i as f64 * 0.01,
            }),
            ..Default::default()
        };
        pub_.publish(&imu)?;
        println!("published Imu #{i}");
        thread::sleep(Duration::from_millis(200));
    }
    Ok(())
}
