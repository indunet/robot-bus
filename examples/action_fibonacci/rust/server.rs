//! Action server for `/examples/fibonacci` (`example_interfaces/action/Fibonacci`).

use robot_bus::Node;
use robot_bus::example_interfaces::action::v1::{
    Fibonacci, FibonacciFeedback, FibonacciGoal, FibonacciResult,
};
use robot_bus::typed::ActionOutcome;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("examples_fibonacci_server");
    let _act = node.create_action_server::<Fibonacci, _>(
        "/examples/fibonacci",
        |goal: FibonacciGoal| {
            let order = goal.order.max(0) as usize;
            let mut seq = Vec::with_capacity(order);
            for i in 0..order {
                if i < 2 {
                    seq.push(i as i32);
                } else {
                    seq.push(seq[i - 1] + seq[i - 2]);
                }
            }
            let feedback_seq = if seq.len() > 1 {
                seq[..seq.len() - 1].to_vec()
            } else {
                seq.clone()
            };
            ActionOutcome {
                feedbacks: vec![FibonacciFeedback {
                    sequence: feedback_seq,
                }],
                result: FibonacciResult { sequence: seq },
            }
        },
        None,
    )?;
    println!("serving /examples/fibonacci (Ctrl+C to stop)");
    node.spin()?;
    Ok(())
}
