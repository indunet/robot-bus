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
    FibonacciActionMapper, Ros2Bridge, StdMsgsStringMapper, TopicQos, TriggerServiceMapper,
};

fn main() -> robot_bus::Result<()> {
    let mut bridge = Ros2Bridge::new("examples_ros2_bridge_builtin")
        .bus_tcp("localhost")
        .from_ros("/examples/chatter", TopicQos::keep_last(10).reliable())
        .to_bus("/examples/chatter", TopicQos::keep_last(8).best_effort())
        .mapper(StdMsgsStringMapper)
        .add()?
        .service()
        .from_ros("/examples/reset", TopicQos::keep_last(10).reliable())
        .to_bus("/examples/reset", TopicQos::keep_last(8).best_effort())
        .mapper(TriggerServiceMapper)
        .timeout(Duration::from_secs(5))
        .add()?
        .action()
        .from_ros("/examples/fibonacci", TopicQos::keep_last(10).reliable())
        .to_bus("/examples/fibonacci", TopicQos::keep_last(8).best_effort())
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
