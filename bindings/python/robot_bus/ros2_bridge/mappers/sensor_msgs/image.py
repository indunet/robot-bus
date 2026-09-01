"""Generated mapper for `sensor_msgs/msg/Image`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def image_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import Image as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.height = msg.height
    bus.width = msg.width
    bus.encoding = str(msg.encoding)
    bus.is_bigendian = bool(msg.is_bigendian)
    bus.step = msg.step
    bus.data = bytes(msg.data)
    return bus


def image_to_ros(bus):
    from sensor_msgs.msg import Image as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.height = bus.height
    out.width = bus.width
    out.encoding = str(bus.encoding)
    out.is_bigendian = int(bool(bus.is_bigendian))
    out.step = bus.step
    out.data = bytes(bus.data)
    return out


class SensorMsgsImageMapper:
    def type_name(self) -> str:
        return "sensor_msgs/msg/Image"

    def ros_msg_type(self):
        from sensor_msgs.msg import Image as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return image_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import Image as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return image_to_ros(bus)
