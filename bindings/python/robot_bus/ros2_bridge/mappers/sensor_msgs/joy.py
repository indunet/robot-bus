"""Generated mapper for `sensor_msgs/msg/Joy`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def joy_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import Joy as BusMsg

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
    def type_name(self) -> str:
        return "sensor_msgs/msg/Joy"

    def ros_msg_type(self):
        from sensor_msgs.msg import Joy as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return joy_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import Joy as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return joy_to_ros(bus)
