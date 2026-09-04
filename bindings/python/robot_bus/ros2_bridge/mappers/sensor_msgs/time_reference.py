"""Generated mapper for `sensor_msgs/msg/TimeReference`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import time_to_bus, time_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import TimeReference as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def time_reference_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.time_ref.CopyFrom(time_to_bus(msg.time_ref))
    bus.source = str(msg.source)
    return bus


def time_reference_to_ros(bus):
    from sensor_msgs.msg import TimeReference as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.time_ref = time_to_ros(bus.time_ref)
    out.source = str(bus.source)
    return out


class SensorMsgsTimeReferenceMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import TimeReference as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return time_reference_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return time_reference_to_ros(bus)
