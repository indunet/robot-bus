"""Generated mapper for `diagnostic_msgs/msg/DiagnosticArray`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.diagnostic_msgs.diagnostic_status import diagnostic_status_to_bus, diagnostic_status_to_ros

def diagnostic_array_to_bus(msg):
    from robot_bus.diagnostic_msgs.msg.v1 import DiagnosticArray as BusMsg

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
    def type_name(self) -> str:
        return "diagnostic_msgs/msg/DiagnosticArray"

    def ros_msg_type(self):
        from diagnostic_msgs.msg import DiagnosticArray as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return diagnostic_array_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.diagnostic_msgs.msg.v1 import DiagnosticArray as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return diagnostic_array_to_ros(bus)
