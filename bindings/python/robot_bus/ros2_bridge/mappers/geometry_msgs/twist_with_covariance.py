"""Generated mapper for `geometry_msgs/msg/TwistWithCovariance`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist import twist_to_bus, twist_to_ros

def twist_with_covariance_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import TwistWithCovariance as BusMsg

    bus = BusMsg()
    bus.twist.CopyFrom(twist_to_bus(msg.twist))
    bus.covariance.extend(list(msg.covariance))
    return bus


def twist_with_covariance_to_ros(bus):
    from geometry_msgs.msg import TwistWithCovariance as RosMsg

    out = RosMsg()
    out.twist = twist_to_ros(bus.twist)
    out.covariance = list(bus.covariance)
    return out


class GeometryMsgsTwistWithCovarianceMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import TwistWithCovariance as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return twist_with_covariance_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import TwistWithCovariance as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return twist_with_covariance_to_ros(bus)
