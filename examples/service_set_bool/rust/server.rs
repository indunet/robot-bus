//! Service server for `/examples/set_bool` (`std_srvs/srv/SetBool`).

use robot_bus::std_srvs::srv::v1::{SetBool, SetBoolRequest, SetBoolResponse};
use robot_bus::Node;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("examples_set_bool_server");
    let _svc = node.create_service::<SetBool, _>(
        "/examples/set_bool",
        |req: SetBoolRequest| SetBoolResponse {
            success: true,
            message: format!("set:{}", req.data),
        },
        None,
    )?;
    println!("serving /examples/set_bool (Ctrl+C to stop)");
    node.spin()?;
    Ok(())
}
