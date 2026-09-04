"""Generated mapper for `nav2_msgs/msg/SpeedLimit`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import SpeedLimit as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def speed_limit_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.percentage = msg.percentage
    bus.speed_limit = msg.speed_limit
    return bus


def speed_limit_to_ros(bus):
    from nav2_msgs.msg import SpeedLimit as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.percentage = bus.percentage
    out.speed_limit = bus.speed_limit
    return out


class Nav2MsgsSpeedLimitMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import SpeedLimit as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return speed_limit_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return speed_limit_to_ros(bus)
