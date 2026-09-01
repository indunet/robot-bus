"""Generated mapper for `geometry_msgs/msg/PoseWithCovariance`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros

def pose_with_covariance_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import PoseWithCovariance as BusMsg

    bus = BusMsg()
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.covariance.extend(list(msg.covariance))
    return bus


def pose_with_covariance_to_ros(bus):
    from geometry_msgs.msg import PoseWithCovariance as RosMsg

    out = RosMsg()
    out.pose = pose_to_ros(bus.pose)
    out.covariance = list(bus.covariance)
    return out


class GeometryMsgsPoseWithCovarianceMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/PoseWithCovariance"

    def ros_msg_type(self):
        from geometry_msgs.msg import PoseWithCovariance as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return pose_with_covariance_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import PoseWithCovariance as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return pose_with_covariance_to_ros(bus)
