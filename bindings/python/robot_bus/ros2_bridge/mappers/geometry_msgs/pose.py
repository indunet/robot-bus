"""Generated mapper for `geometry_msgs/msg/Pose`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.quaternion import quaternion_to_bus, quaternion_to_ros

def pose_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import Pose as BusMsg

    bus = BusMsg()
    bus.position.CopyFrom(point_to_bus(msg.position))
    bus.orientation.CopyFrom(quaternion_to_bus(msg.orientation))
    return bus


def pose_to_ros(bus):
    from geometry_msgs.msg import Pose as RosMsg

    out = RosMsg()
    out.position = point_to_ros(bus.position)
    out.orientation = quaternion_to_ros(bus.orientation)
    return out


class GeometryMsgsPoseMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/Pose"

    def ros_msg_type(self):
        from geometry_msgs.msg import Pose as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return pose_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import Pose as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return pose_to_ros(bus)
