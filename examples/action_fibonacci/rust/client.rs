//! Call `/examples/fibonacci` once.

use std::sync::Arc;
use std::time::Duration;

use robot_bus::example_interfaces::action::v1::{Fibonacci, FibonacciGoal};
use robot_bus::Node;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("examples_fibonacci_client");
    let client = node.create_action_client::<Fibonacci>("/examples/fibonacci")?;
    let _ = client.wait_for_action_server(Some(Duration::from_secs(5)));

    let goal = client.send_goal(
        &FibonacciGoal { order: 5 },
        None,
        Some(Duration::from_secs(10)),
        Some(Arc::new(|feedback| {
            println!("feedback: {:?}", feedback.sequence);
        })),
    )?;
    let result = goal.wait_result()?;
    println!("result: {:?}", result.sequence);
    Ok(())
}
