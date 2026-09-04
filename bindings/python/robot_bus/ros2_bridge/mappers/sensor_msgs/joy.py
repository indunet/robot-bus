"""Generated mapper for `sensor_msgs/msg/Joy`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import Joy as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def joy_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.axes.extend(list(msg.axes))
    bus.buttons.extend(list(msg.buttons))
    return bus


def joy_to_ros(bus):
    from sensor_msgs.msg import Joy as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.axes = list(bus.axes)
    out.buttons = list(bus.buttons)
    return out


class SensorMsgsJoyMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import Joy as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return joy_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return joy_to_ros(bus)
