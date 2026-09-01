"""Generated mapper for `diagnostic_msgs/msg/DiagnosticStatus`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.diagnostic_msgs.key_value import key_value_to_bus, key_value_to_ros

def diagnostic_status_to_bus(msg):
    from robot_bus.diagnostic_msgs.msg.v1 import DiagnosticStatus as BusMsg

    bus = BusMsg()
    bus.level = msg.level
    bus.name = str(msg.name)
    bus.message = str(msg.message)
    bus.hardware_id = str(msg.hardware_id)
    bus.values.extend([key_value_to_bus(x) for x in msg.values])
    return bus


def diagnostic_status_to_ros(bus):
    from diagnostic_msgs.msg import DiagnosticStatus as RosMsg

    out = RosMsg()
    out.level = bus.level
    out.name = str(bus.name)
    out.message = str(bus.message)
    out.hardware_id = str(bus.hardware_id)
    out.values = [key_value_to_ros(x) for x in bus.values]
    return out


class DiagnosticMsgsDiagnosticStatusMapper:
    def ros_msg_type(self):
        from diagnostic_msgs.msg import DiagnosticStatus as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return diagnostic_status_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.diagnostic_msgs.msg.v1 import DiagnosticStatus as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return diagnostic_status_to_ros(bus)
