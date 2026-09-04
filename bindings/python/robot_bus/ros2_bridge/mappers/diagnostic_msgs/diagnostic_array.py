"""Generated mapper for `diagnostic_msgs/msg/DiagnosticArray`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.diagnostic_msgs.diagnostic_status import diagnostic_status_to_bus, diagnostic_status_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.diagnostic_msgs.msg.v1 import DiagnosticArray as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def diagnostic_array_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.status.extend([diagnostic_status_to_bus(x) for x in msg.status])
    return bus


def diagnostic_array_to_ros(bus):
    from diagnostic_msgs.msg import DiagnosticArray as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.status = [diagnostic_status_to_ros(x) for x in bus.status]
    return out


class DiagnosticMsgsDiagnosticArrayMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from diagnostic_msgs.msg import DiagnosticArray as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return diagnostic_array_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return diagnostic_array_to_ros(bus)
