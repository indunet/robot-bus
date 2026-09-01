"""Generated mapper for `sensor_msgs/msg/CompressedImage`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def compressed_image_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import CompressedImage as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.format = str(msg.format)
    bus.data = bytes(msg.data)
    return bus


def compressed_image_to_ros(bus):
    from sensor_msgs.msg import CompressedImage as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.format = str(bus.format)
    out.data = bytes(bus.data)
    return out


class SensorMsgsCompressedImageMapper:
    def ros_msg_type(self):
        from sensor_msgs.msg import CompressedImage as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return compressed_image_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import CompressedImage as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return compressed_image_to_ros(bus)
