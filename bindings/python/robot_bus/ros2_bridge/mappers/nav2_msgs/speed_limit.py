"""Generated mapper for `nav2_msgs/msg/SpeedLimit`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def speed_limit_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import SpeedLimit as BusMsg

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
    def type_name(self) -> str:
        return "nav2_msgs/msg/SpeedLimit"

    def ros_msg_type(self):
        from nav2_msgs.msg import SpeedLimit as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return speed_limit_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import SpeedLimit as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return speed_limit_to_ros(bus)
