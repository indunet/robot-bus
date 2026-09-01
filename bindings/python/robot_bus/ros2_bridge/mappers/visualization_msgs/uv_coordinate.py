"""Generated mapper for `visualization_msgs/msg/UVCoordinate`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def uv_coordinate_to_bus(msg):
    from robot_bus.visualization_msgs.msg.v1 import UVCoordinate as BusMsg

    bus = BusMsg()
    bus.u = msg.u
    bus.v = msg.v
    return bus


def uv_coordinate_to_ros(bus):
    from visualization_msgs.msg import UVCoordinate as RosMsg

    out = RosMsg()
    out.u = bus.u
    out.v = bus.v
    return out


class VisualizationMsgsUvCoordinateMapper:
    def type_name(self) -> str:
        return "visualization_msgs/msg/UVCoordinate"

    def ros_msg_type(self):
        from visualization_msgs.msg import UVCoordinate as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return uv_coordinate_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.visualization_msgs.msg.v1 import UVCoordinate as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return uv_coordinate_to_ros(bus)
