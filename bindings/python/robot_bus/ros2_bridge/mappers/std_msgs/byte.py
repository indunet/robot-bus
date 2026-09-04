"""Generated mapper for `std_msgs/msg/Byte`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.std_msgs.msg.v1 import Byte as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def byte_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.data = int(msg.data)
    return bus


def byte_to_ros(bus):
    from std_msgs.msg import Byte as RosMsg

    out = RosMsg()
    out.data = int(bus.data)
    return out


class StdMsgsByteMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from std_msgs.msg import Byte as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return byte_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return byte_to_ros(bus)
