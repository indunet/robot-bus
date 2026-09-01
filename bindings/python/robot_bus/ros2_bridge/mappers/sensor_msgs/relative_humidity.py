"""Generated mapper for `sensor_msgs/msg/RelativeHumidity`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def relative_humidity_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import RelativeHumidity as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.relative_humidity = msg.relative_humidity
    bus.variance = msg.variance
    return bus


def relative_humidity_to_ros(bus):
    from sensor_msgs.msg import RelativeHumidity as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.relative_humidity = bus.relative_humidity
    out.variance = bus.variance
    return out


class SensorMsgsRelativeHumidityMapper:
    def ros_msg_type(self):
        from sensor_msgs.msg import RelativeHumidity as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return relative_humidity_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import RelativeHumidity as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return relative_humidity_to_ros(bus)
