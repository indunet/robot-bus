"""Generated mapper for `sensor_msgs/msg/PointCloud`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point32 import point32_to_bus, point32_to_ros
from robot_bus.ros2_bridge.mappers.sensor_msgs.channel_float32 import channel_float32_to_bus, channel_float32_to_ros

def point_cloud_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import PointCloud as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.points.extend([point32_to_bus(x) for x in msg.points])
    bus.channels.extend([channel_float32_to_bus(x) for x in msg.channels])
    return bus


def point_cloud_to_ros(bus):
    from sensor_msgs.msg import PointCloud as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.points = [point32_to_ros(x) for x in bus.points]
    out.channels = [channel_float32_to_ros(x) for x in bus.channels]
    return out


class SensorMsgsPointCloudMapper:
    def ros_msg_type(self):
        from sensor_msgs.msg import PointCloud as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return point_cloud_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import PointCloud as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return point_cloud_to_ros(bus)
