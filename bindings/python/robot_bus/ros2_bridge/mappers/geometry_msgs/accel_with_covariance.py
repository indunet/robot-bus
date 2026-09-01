"""Generated mapper for `geometry_msgs/msg/AccelWithCovariance`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.accel import accel_to_bus, accel_to_ros

def accel_with_covariance_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import AccelWithCovariance as BusMsg

    bus = BusMsg()
    bus.accel.CopyFrom(accel_to_bus(msg.accel))
    bus.covariance.extend(list(msg.covariance))
    return bus


def accel_with_covariance_to_ros(bus):
    from geometry_msgs.msg import AccelWithCovariance as RosMsg

    out = RosMsg()
    out.accel = accel_to_ros(bus.accel)
    out.covariance = list(bus.covariance)
    return out


class GeometryMsgsAccelWithCovarianceMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/AccelWithCovariance"

    def ros_msg_type(self):
        from geometry_msgs.msg import AccelWithCovariance as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return accel_with_covariance_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import AccelWithCovariance as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return accel_with_covariance_to_ros(bus)
