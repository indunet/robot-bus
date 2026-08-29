//! Subscribe to `/examples/imu` (typed `sensor_msgs/msg/Imu`).
//!
//! Run `robot-bus-broker` first, then this listener, then `topic_imu_talker`.

use robot_bus::Node;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("examples_imu_listener");
    let _sub = node.create_subscription::<Imu, _>(
        "/examples/imu",
        |imu| {
            let z = imu
                .linear_acceleration
                .as_ref()
                .map(|v: &Vector3| v.z)
                .unwrap_or(0.0);
            println!("linear_acceleration.z={z}");
        },
        None,
    )?;
    println!("listening on /examples/imu (Ctrl+C to stop)");
    node.spin()?;
    Ok(())
}
