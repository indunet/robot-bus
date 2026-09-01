"""Generated mapper for `control_msgs/msg/MotionPrimitiveSequence`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.control_msgs.motion_primitive import motion_primitive_to_bus, motion_primitive_to_ros

def motion_primitive_sequence_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import MotionPrimitiveSequence as BusMsg

    bus = BusMsg()
    bus.motions.extend([motion_primitive_to_bus(x) for x in msg.motions])
    return bus


def motion_primitive_sequence_to_ros(bus):
    from control_msgs.msg import MotionPrimitiveSequence as RosMsg

    out = RosMsg()
    out.motions = [motion_primitive_to_ros(x) for x in bus.motions]
    return out


class ControlMsgsMotionPrimitiveSequenceMapper:
    def ros_msg_type(self):
        from control_msgs.msg import MotionPrimitiveSequence as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return motion_primitive_sequence_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import MotionPrimitiveSequence as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return motion_primitive_sequence_to_ros(bus)
