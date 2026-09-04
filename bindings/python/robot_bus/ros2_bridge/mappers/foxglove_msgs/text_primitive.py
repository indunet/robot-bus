"""Generated mapper for `foxglove_msgs/msg/TextPrimitive`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import color_to_bus, color_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import TextPrimitive as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def text_primitive_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.billboard = msg.billboard
    bus.font_size = msg.font_size
    bus.scale_invariant = msg.scale_invariant
    bus.color.CopyFrom(color_to_bus(msg.color))
    bus.text = str(msg.text)
    return bus


def text_primitive_to_ros(bus):
    from foxglove_msgs.msg import TextPrimitive as RosMsg

    out = RosMsg()
    out.pose = pose_to_ros(bus.pose)
    out.billboard = bus.billboard
    out.font_size = bus.font_size
    out.scale_invariant = bus.scale_invariant
    out.color = color_to_ros(bus.color)
    out.text = str(bus.text)
    return out


class FoxgloveMsgsTextPrimitiveMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import TextPrimitive as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return text_primitive_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return text_primitive_to_ros(bus)
