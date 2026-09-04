"""Generated mapper for `foxglove_msgs/msg/PointCloud`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.packed_element_field import packed_element_field_to_bus, packed_element_field_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import PointCloud as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def point_cloud_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.point_stride = msg.point_stride
    bus.fields.extend([packed_element_field_to_bus(x) for x in msg.fields])
    bus.data = bytes(msg.data)
    return bus


def point_cloud_to_ros(bus):
    from foxglove_msgs.msg import PointCloud as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.pose = pose_to_ros(bus.pose)
    out.point_stride = bus.point_stride
    out.fields = [packed_element_field_to_ros(x) for x in bus.fields]
    out.data = bytes(bus.data)
    return out


class FoxgloveMsgsPointCloudMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import PointCloud as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return point_cloud_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return point_cloud_to_ros(bus)
