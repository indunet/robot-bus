"""Generated mapper for `geometry_msgs/msg/TwistWithCovariance`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist import twist_to_bus, twist_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.geometry_msgs.msg.v1 import TwistWithCovariance as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def twist_with_covariance_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from geometry_msgs.msg import TwistWithCovariance as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return twist_with_covariance_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return twist_with_covariance_to_ros(bus)
