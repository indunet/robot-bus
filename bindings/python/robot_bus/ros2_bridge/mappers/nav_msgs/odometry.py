"""Generated mapper for `nav_msgs/msg/Odometry`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose_with_covariance import pose_with_covariance_to_bus, pose_with_covariance_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist_with_covariance import twist_with_covariance_to_bus, twist_with_covariance_to_ros

def odometry_to_bus(msg):
    from robot_bus.nav_msgs.msg.v1 import Odometry as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.child_frame_id = str(msg.child_frame_id)
    bus.pose.CopyFrom(pose_with_covariance_to_bus(msg.pose))
    bus.twist.CopyFrom(twist_with_covariance_to_bus(msg.twist))
    return bus


def odometry_to_ros(bus):
    from nav_msgs.msg import Odometry as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.child_frame_id = str(bus.child_frame_id)
    out.pose = pose_with_covariance_to_ros(bus.pose)
    out.twist = twist_with_covariance_to_ros(bus.twist)
    return out


class NavMsgsOdometryMapper:
    def type_name(self) -> str:
        return "nav_msgs/msg/Odometry"

    def ros_msg_type(self):
        from nav_msgs.msg import Odometry as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return odometry_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav_msgs.msg.v1 import Odometry as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return odometry_to_ros(bus)
