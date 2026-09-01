"""Generated mapper for `sensor_msgs/msg/Temperature`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def temperature_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import Temperature as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.temperature = msg.temperature
    bus.variance = msg.variance
    return bus


def temperature_to_ros(bus):
    from sensor_msgs.msg import Temperature as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.temperature = bus.temperature
    out.variance = bus.variance
    return out


class SensorMsgsTemperatureMapper:
    def type_name(self) -> str:
        return "sensor_msgs/msg/Temperature"

    def ros_msg_type(self):
        from sensor_msgs.msg import Temperature as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return temperature_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import Temperature as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return temperature_to_ros(bus)
