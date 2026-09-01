"""Generated mapper for `foxglove_msgs/msg/CompressedPointCloud`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros

def compressed_point_cloud_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import CompressedPointCloud as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.data = bytes(msg.data)
    bus.format = str(msg.format)
    return bus


def compressed_point_cloud_to_ros(bus):
    from foxglove_msgs.msg import CompressedPointCloud as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.pose = pose_to_ros(bus.pose)
    out.data = bytes(bus.data)
    out.format = str(bus.format)
    return out


class FoxgloveMsgsCompressedPointCloudMapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import CompressedPointCloud as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return compressed_point_cloud_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import CompressedPointCloud as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return compressed_point_cloud_to_ros(bus)
