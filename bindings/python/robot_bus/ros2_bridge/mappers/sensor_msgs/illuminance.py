"""Generated mapper for `sensor_msgs/msg/Illuminance`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def illuminance_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import Illuminance as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.illuminance = msg.illuminance
    bus.variance = msg.variance
    return bus


def illuminance_to_ros(bus):
    from sensor_msgs.msg import Illuminance as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.illuminance = bus.illuminance
    out.variance = bus.variance
    return out


class SensorMsgsIlluminanceMapper:
    def ros_msg_type(self):
        from sensor_msgs.msg import Illuminance as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return illuminance_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import Illuminance as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return illuminance_to_ros(bus)
