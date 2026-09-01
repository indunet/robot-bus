"""Generated mapper for `control_msgs/msg/MotionPrimitive`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.control_msgs.motion_argument import motion_argument_to_bus, motion_argument_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose_stamped import pose_stamped_to_bus, pose_stamped_to_ros

def motion_primitive_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import MotionPrimitive as BusMsg

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
    def ros_msg_type(self):
        from control_msgs.msg import MotionPrimitive as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return motion_primitive_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import MotionPrimitive as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return motion_primitive_to_ros(bus)
