"""Generated mapper for `geometry_msgs/msg/PointStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.geometry_msgs.msg.v1 import PointStamped as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def point_stamped_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.point.CopyFrom(point_to_bus(msg.point))
    return bus


def point_stamped_to_ros(bus):
    from geometry_msgs.msg import PointStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.point = point_to_ros(bus.point)
    return out


class GeometryMsgsPointStampedMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from geometry_msgs.msg import PointStamped as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return point_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return point_stamped_to_ros(bus)
