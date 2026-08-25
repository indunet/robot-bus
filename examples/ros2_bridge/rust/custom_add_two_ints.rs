//! Custom Ros2Bridge: ROS `.srv` + our bus `.proto` + [`TypedServiceMapper`].
//!
//! Two interface definitions (fields must match):
//! - [`examples/ros2_bridge/ros2/my_pkg/srv/AddTwoInts.srv`]
//! - [`examples/ros2_bridge/proto/my_pkg/srv/v1/add_two_ints.proto`]
//!
//! Bus stubs: [`my_pkg`] (prost, matching the `.proto`).
//!
//! Runtime smoke uses system `example_interfaces/srv/AddTwoInts` (identical
//! fields) so you need not colcon-build `my_pkg` first.
//!
//! ```bash
//! source /opt/ros/humble/setup.bash
//! cargo run --example ros2_bridge_custom_add_two_ints --features ros2
//!
//! ros2 service call /examples/add_two_ints example_interfaces/srv/AddTwoInts "{a: 2, b: 40}"
//! ```

#[path = "my_pkg.rs"]
mod my_pkg;

use std::time::Duration;

use prost::Message as ProstMessage;
use ros_env::example_interfaces::srv as ros_srv;
use robot_bus::ros2_bridge::{Direction, Ros2Bridge, TypedServiceMapper};
use robot_bus::Node;

use my_pkg::{AddTwoIntsRequest, AddTwoIntsResponse};

/// Glue: ROS AddTwoInts ↔ bus `my_pkg.srv.v1` protobuf.
#[derive(Clone, Copy, Debug, Default)]
struct AddTwoIntsServiceMapper;

impl TypedServiceMapper for AddTwoIntsServiceMapper {
    type Ros = ros_srv::AddTwoInts;

    fn type_name(&self) -> &str {
        // Smoke: system type with the same fields as ros2/my_pkg/srv/AddTwoInts.srv.
        "example_interfaces/srv/AddTwoInts"
    }

    fn ros_req_to_bus(&self, req: &ros_srv::AddTwoInts_Request) -> robot_bus::Result<Vec<u8>> {
        Ok(AddTwoIntsRequest {
            a: req.a,
            b: req.b,
        }
        .encode_to_vec())
    }

    fn bus_req_to_ros(&self, payload: &[u8]) -> robot_bus::Result<ros_srv::AddTwoInts_Request> {
        let bus = AddTwoIntsRequest::decode(payload).map_err(|e| {
            robot_bus::BusError::Protocol(format!("decode AddTwoIntsRequest: {e}"))
        })?;
        Ok(ros_srv::AddTwoInts_Request {
            a: bus.a,
            b: bus.b,
        })
    }

    fn ros_resp_to_bus(&self, resp: &ros_srv::AddTwoInts_Response) -> robot_bus::Result<Vec<u8>> {
        Ok(AddTwoIntsResponse { sum: resp.sum }.encode_to_vec())
    }

    fn bus_resp_to_ros(&self, payload: &[u8]) -> robot_bus::Result<ros_srv::AddTwoInts_Response> {
        let bus = AddTwoIntsResponse::decode(payload).map_err(|e| {
            robot_bus::BusError::Protocol(format!("decode AddTwoIntsResponse: {e}"))
        })?;
        Ok(ros_srv::AddTwoInts_Response { sum: bus.sum })
    }
}

fn main() -> robot_bus::Result<()> {
    let mut bus = Node::new("examples_add_two_ints_bus");
    bus.create_service_raw(
        "/examples/add_two_ints",
        |body| match AddTwoIntsRequest::decode(body) {
            Ok(req) => AddTwoIntsResponse {
                sum: req.a + req.b,
            }
            .encode_to_vec(),
            Err(err) => {
                log::warn!("decode AddTwoIntsRequest: {err}");
                Vec::new()
            }
        },
        None,
    )?;

    let mut bridge = Ros2Bridge::new("examples_ros2_bridge_custom")
        .bus_tcp("localhost")
        .service("/examples/add_two_ints", "/examples/add_two_ints")
        .mapper(AddTwoIntsServiceMapper)
        .direction(Direction::Ros2ToBus)
        .timeout(Duration::from_secs(5))
        .add()?
        .build()?;

    println!(
        "custom my_pkg AddTwoInts bridge on /examples/add_two_ints \
         (Ros2ToBus + in-process bus server; Ctrl+C to stop)"
    );
    loop {
        let _ = bus.spin_once(Some(Duration::from_millis(10)));
        let _ = bridge.spin_once(Duration::from_millis(10));
    }
}
