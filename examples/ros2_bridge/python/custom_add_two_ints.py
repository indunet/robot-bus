"""Custom Ros2Bridge: ROS ``.srv`` + our bus ``.proto`` + mapper.

Two interface definitions (fields must match)::

    examples/ros2_bridge/ros2/my_pkg/srv/AddTwoInts.srv
    examples/ros2_bridge/proto/my_pkg/srv/v1/add_two_ints.proto

Also see ``ros2/my_pkg/msg/Sum.msg`` and ``ros2/my_pkg/action/Compute.action``
for the same dual-definition idea on topic / action shapes.

This process hosts a bus server and bridges ROS → bus. Runtime smoke uses
system ``example_interfaces/srv/AddTwoInts`` (identical fields) so you need not
colcon-build ``my_pkg`` first; swap ``type_name`` / ``ros_srv_type`` to
``my_pkg`` once that ROS package exists.

::

    source /opt/ros/humble/setup.bash
    robot-bus-broker
    python3 examples/ros2_bridge/python/custom_add_two_ints.py

    ros2 service call /examples/add_two_ints example_interfaces/srv/AddTwoInts "{a: 2, b: 40}"
"""

from __future__ import annotations

import robot_bus
from example_interfaces.srv import AddTwoInts as RosAddTwoInts
from my_pkg.srv.v1 import add_two_ints_pb2 as pb
from robot_bus.ros2_bridge import Direction, Ros2Bridge


class AddTwoIntsServiceMapper:
    """Glue: ROS AddTwoInts ↔ bus ``my_pkg.srv.v1`` protobuf."""

    def type_name(self) -> str:
        # Smoke: system type with the same fields as ros2/my_pkg/srv/AddTwoInts.srv.
        # Production: return "my_pkg/srv/AddTwoInts" and import that ROS type.
        return "example_interfaces/srv/AddTwoInts"

    def ros_srv_type(self):
        return RosAddTwoInts

    def ros_req_to_bus(self, req) -> bytes:
        return pb.AddTwoIntsRequest(a=int(req.a), b=int(req.b)).SerializeToString()

    def bus_req_to_ros(self, payload: bytes):
        bus = pb.AddTwoIntsRequest()
        bus.ParseFromString(payload)
        out = RosAddTwoInts.Request()
        out.a = int(bus.a)
        out.b = int(bus.b)
        return out

    def ros_resp_to_bus(self, resp) -> bytes:
        return pb.AddTwoIntsResponse(sum=int(resp.sum)).SerializeToString()

    def bus_resp_to_ros(self, payload: bytes):
        bus = pb.AddTwoIntsResponse()
        bus.ParseFromString(payload)
        out = RosAddTwoInts.Response()
        out.sum = int(bus.sum)
        return out


def _on_add(req: pb.AddTwoIntsRequest) -> pb.AddTwoIntsResponse:
    return pb.AddTwoIntsResponse(sum=int(req.a) + int(req.b))


def main() -> None:
    if not robot_bus.ros2_available():
        raise SystemExit(
            "ROS 2 not available: source /opt/ros/humble|jazzy/setup.bash "
            "and install rclpy (just python-dev-ros2)"
        )

    bus = robot_bus.Node("examples_add_two_ints_bus")
    bus.create_service(
        "/examples/add_two_ints",
        _on_add,
        request_type=pb.AddTwoIntsRequest,
        response_type=pb.AddTwoIntsResponse,
    )

    bridge = (
        Ros2Bridge.new("examples_ros2_bridge_custom")
        .bus_tcp("localhost")
        .service("/examples/add_two_ints", "/examples/add_two_ints")
        .mapper(AddTwoIntsServiceMapper())
        .direction(Direction.Ros2ToBus)
        .timeout(5.0)
        .add()
        .build()
    )

    print(
        "custom my_pkg AddTwoInts bridge on /examples/add_two_ints "
        "(Ros2ToBus + in-process bus server; Ctrl+C to stop)"
    )
    try:
        # Bridge spins ROS on a background thread. Drive the bus *server* here;
        # bridge.spin_once is also safe (idle bus client is tolerated).
        while True:
            bus.spin_once(0.01)
            bridge.spin_once(0.01)
    except KeyboardInterrupt:
        bridge.shutdown()
        bus.shutdown()


if __name__ == "__main__":
    main()
