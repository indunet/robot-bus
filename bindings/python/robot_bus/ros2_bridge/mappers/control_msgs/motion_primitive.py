"""Generated mapper for `control_msgs/msg/MotionPrimitive`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.control_msgs.motion_argument import motion_argument_to_bus, motion_argument_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose_stamped import pose_stamped_to_bus, pose_stamped_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import MotionPrimitive as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def motion_primitive_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.type = msg.type
    bus.blend_radius = msg.blend_radius
    bus.additional_arguments.extend([motion_argument_to_bus(x) for x in msg.additional_arguments])
    bus.poses.extend([pose_stamped_to_bus(x) for x in msg.poses])
    bus.joint_positions.extend(list(msg.joint_positions))
    return bus


def motion_primitive_to_ros(bus):
    from control_msgs.msg import MotionPrimitive as RosMsg

    out = RosMsg()
    out.type = bus.type
    out.blend_radius = bus.blend_radius
    out.additional_arguments = [motion_argument_to_ros(x) for x in bus.additional_arguments]
    out.poses = [pose_stamped_to_ros(x) for x in bus.poses]
    out.joint_positions = list(bus.joint_positions)
    return out


class ControlMsgsMotionPrimitiveMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import MotionPrimitive as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return motion_primitive_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return motion_primitive_to_ros(bus)
