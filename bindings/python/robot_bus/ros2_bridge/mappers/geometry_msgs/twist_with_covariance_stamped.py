"""Generated mapper for `geometry_msgs/msg/TwistWithCovarianceStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist_with_covariance import twist_with_covariance_to_bus, twist_with_covariance_to_ros

def twist_with_covariance_stamped_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import TwistWithCovarianceStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.twist.CopyFrom(twist_with_covariance_to_bus(msg.twist))
    return bus


def twist_with_covariance_stamped_to_ros(bus):
    from geometry_msgs.msg import TwistWithCovarianceStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.twist = twist_with_covariance_to_ros(bus.twist)
    return out


class GeometryMsgsTwistWithCovarianceStampedMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/TwistWithCovarianceStamped"

    def ros_msg_type(self):
        from geometry_msgs.msg import TwistWithCovarianceStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return twist_with_covariance_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import TwistWithCovarianceStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return twist_with_covariance_stamped_to_ros(bus)
