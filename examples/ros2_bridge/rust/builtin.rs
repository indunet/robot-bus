//! Built-in Ros2Bridge mappers: String topic, Trigger service, Fibonacci action.
//!
//! ```bash
//! source /opt/ros/humble/setup.bash
//! cargo run --example ros2_bridge_builtin --features ros2
//! ```
//!
//! For the more common *custom* mapper pattern, see `ros2_bridge_custom_add_two_ints`.

use std::time::Duration;

use robot_bus::ros2_bridge::{
    Direction, FibonacciActionMapper, Ros2Bridge, StdMsgsStringMapper, TriggerServiceMapper,
};

fn main() -> robot_bus::Result<()> {
    let mut bridge = Ros2Bridge::new("examples_ros2_bridge_builtin")
        .bus_tcp("localhost")
        .route("/examples/chatter", "/examples/chatter")
        .mapper(StdMsgsStringMapper)
        .direction(Direction::Ros2ToBus)
        .add()?
        .service("/examples/reset", "/examples/reset")
        .mapper(TriggerServiceMapper)
        .timeout(Duration::from_secs(5))
        .add()?
        .action("/examples/fibonacci", "/examples/fibonacci")
        .mapper(FibonacciActionMapper)
        .add()?
        .build()?;

    println!(
        "builtin bridge: /examples/chatter, /examples/reset, /examples/fibonacci \
         (Ros2ToBus; Ctrl+C to stop)"
    );
    bridge.spin()?;
    Ok(())
}
