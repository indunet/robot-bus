"""Generated mapper for `std_msgs/msg/Header`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import time_to_bus, time_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.std_msgs.msg.v1 import Header as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def header_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.stamp.CopyFrom(time_to_bus(msg.stamp))
    bus.frame_id = str(msg.frame_id)
    return bus


def header_to_ros(bus):
    from std_msgs.msg import Header as RosMsg

    out = RosMsg()
    out.stamp = time_to_ros(bus.stamp)
    out.frame_id = str(bus.frame_id)
    return out


class StdMsgsHeaderMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from std_msgs.msg import Header as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return header_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return header_to_ros(bus)
