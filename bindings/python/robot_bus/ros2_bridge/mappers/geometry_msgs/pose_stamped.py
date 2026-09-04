"""Generated mapper for `geometry_msgs/msg/PoseStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.geometry_msgs.msg.v1 import PoseStamped as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def pose_stamped_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    return bus


def pose_stamped_to_ros(bus):
    from geometry_msgs.msg import PoseStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.pose = pose_to_ros(bus.pose)
    return out


class GeometryMsgsPoseStampedMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from geometry_msgs.msg import PoseStamped as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return pose_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return pose_stamped_to_ros(bus)
