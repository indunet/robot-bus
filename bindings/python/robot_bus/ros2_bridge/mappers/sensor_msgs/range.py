"""Generated mapper for `sensor_msgs/msg/Range`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import Range as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def range_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.radiation_type = msg.radiation_type
    bus.field_of_view = msg.field_of_view
    bus.min_range = msg.min_range
    bus.max_range = msg.max_range
    bus.range = msg.range
    return bus


def range_to_ros(bus):
    from sensor_msgs.msg import Range as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.radiation_type = bus.radiation_type
    out.field_of_view = bus.field_of_view
    out.min_range = bus.min_range
    out.max_range = bus.max_range
    out.range = bus.range
    return out


class SensorMsgsRangeMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import Range as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return range_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return range_to_ros(bus)
