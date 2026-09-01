"""Generated mapper for `shape_msgs/msg/Plane`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def plane_to_bus(msg):
    from robot_bus.shape_msgs.msg.v1 import Plane as BusMsg

    bus = BusMsg()
    bus.coef.extend(list(msg.coef))
    return bus


def plane_to_ros(bus):
    from shape_msgs.msg import Plane as RosMsg

    out = RosMsg()
    out.coef = list(bus.coef)
    return out


class ShapeMsgsPlaneMapper:
    def type_name(self) -> str:
        return "shape_msgs/msg/Plane"

    def ros_msg_type(self):
        from shape_msgs.msg import Plane as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return plane_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.shape_msgs.msg.v1 import Plane as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return plane_to_ros(bus)
