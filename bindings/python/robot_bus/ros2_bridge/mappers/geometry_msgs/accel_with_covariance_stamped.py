"""Generated mapper for `geometry_msgs/msg/AccelWithCovarianceStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.accel_with_covariance import accel_with_covariance_to_bus, accel_with_covariance_to_ros

def accel_with_covariance_stamped_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import AccelWithCovarianceStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.accel.CopyFrom(accel_with_covariance_to_bus(msg.accel))
    return bus


def accel_with_covariance_stamped_to_ros(bus):
    from geometry_msgs.msg import AccelWithCovarianceStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.accel = accel_with_covariance_to_ros(bus.accel)
    return out


class GeometryMsgsAccelWithCovarianceStampedMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/AccelWithCovarianceStamped"

    def ros_msg_type(self):
        from geometry_msgs.msg import AccelWithCovarianceStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return accel_with_covariance_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import AccelWithCovarianceStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return accel_with_covariance_stamped_to_ros(bus)
