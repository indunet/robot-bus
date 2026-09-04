"""Generated mapper for `control_msgs/msg/MotionPrimitiveSequence`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.control_msgs.motion_primitive import motion_primitive_to_bus, motion_primitive_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import MotionPrimitiveSequence as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def motion_primitive_sequence_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.motions.extend([motion_primitive_to_bus(x) for x in msg.motions])
    return bus


def motion_primitive_sequence_to_ros(bus):
    from control_msgs.msg import MotionPrimitiveSequence as RosMsg

    out = RosMsg()
    out.motions = [motion_primitive_to_ros(x) for x in bus.motions]
    return out


class ControlMsgsMotionPrimitiveSequenceMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import MotionPrimitiveSequence as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return motion_primitive_sequence_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return motion_primitive_sequence_to_ros(bus)
