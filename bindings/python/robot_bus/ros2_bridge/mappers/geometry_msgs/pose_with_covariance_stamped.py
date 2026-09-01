"""Generated mapper for `geometry_msgs/msg/PoseWithCovarianceStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose_with_covariance import pose_with_covariance_to_bus, pose_with_covariance_to_ros

def pose_with_covariance_stamped_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import PoseWithCovarianceStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.pose.CopyFrom(pose_with_covariance_to_bus(msg.pose))
    return bus


def pose_with_covariance_stamped_to_ros(bus):
    from geometry_msgs.msg import PoseWithCovarianceStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.pose = pose_with_covariance_to_ros(bus.pose)
    return out


class GeometryMsgsPoseWithCovarianceStampedMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/PoseWithCovarianceStamped"

    def ros_msg_type(self):
        from geometry_msgs.msg import PoseWithCovarianceStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return pose_with_covariance_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import PoseWithCovarianceStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return pose_with_covariance_stamped_to_ros(bus)
