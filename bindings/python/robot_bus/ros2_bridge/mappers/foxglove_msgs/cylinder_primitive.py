"""Generated mapper for `foxglove_msgs/msg/CylinderPrimitive`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector3 import vector3_to_bus, vector3_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import color_to_bus, color_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import CylinderPrimitive as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def cylinder_primitive_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.size.CopyFrom(vector3_to_bus(msg.size))
    bus.bottom_scale = msg.bottom_scale
    bus.top_scale = msg.top_scale
    bus.color.CopyFrom(color_to_bus(msg.color))
    return bus


def cylinder_primitive_to_ros(bus):
    from foxglove_msgs.msg import CylinderPrimitive as RosMsg

    out = RosMsg()
    out.pose = pose_to_ros(bus.pose)
    out.size = vector3_to_ros(bus.size)
    out.bottom_scale = bus.bottom_scale
    out.top_scale = bus.top_scale
    out.color = color_to_ros(bus.color)
    return out


class FoxgloveMsgsCylinderPrimitiveMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import CylinderPrimitive as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return cylinder_primitive_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return cylinder_primitive_to_ros(bus)
