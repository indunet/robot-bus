//! Call `/examples/set_bool` once.

use std::thread;
use std::time::Duration;

use robot_bus::Node;
use robot_bus::std_srvs::srv::v1::{SetBool, SetBoolRequest};

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("examples_set_bool_client");
    let client = node.create_client::<SetBool>("/examples/set_bool")?;
    let _ = client.wait_for_service(Some(Duration::from_secs(5)));
    thread::sleep(Duration::from_millis(200));

    let resp = client.call(&SetBoolRequest { data: true }, Some(Duration::from_secs(5)))?;
    println!("success={} message={}", resp.success, resp.message);
    Ok(())
}
