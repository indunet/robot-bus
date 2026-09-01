"""Generated mapper for `foxglove_msgs/msg/ArrowPrimitive`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import color_to_bus, color_to_ros

def arrow_primitive_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import ArrowPrimitive as BusMsg

    bus = BusMsg()
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.shaft_length = msg.shaft_length
    bus.shaft_diameter = msg.shaft_diameter
    bus.head_length = msg.head_length
    bus.head_diameter = msg.head_diameter
    bus.color.CopyFrom(color_to_bus(msg.color))
    return bus


def arrow_primitive_to_ros(bus):
    from foxglove_msgs.msg import ArrowPrimitive as RosMsg

    out = RosMsg()
    out.pose = pose_to_ros(bus.pose)
    out.shaft_length = bus.shaft_length
    out.shaft_diameter = bus.shaft_diameter
    out.head_length = bus.head_length
    out.head_diameter = bus.head_diameter
    out.color = color_to_ros(bus.color)
    return out


class FoxgloveMsgsArrowPrimitiveMapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import ArrowPrimitive as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return arrow_primitive_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import ArrowPrimitive as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return arrow_primitive_to_ros(bus)
