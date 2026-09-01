"""Generated mapper for `sensor_msgs/msg/TimeReference`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import time_to_bus, time_to_ros

def time_reference_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import TimeReference as BusMsg

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
    def ros_msg_type(self):
        from sensor_msgs.msg import TimeReference as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return time_reference_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import TimeReference as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return time_reference_to_ros(bus)
