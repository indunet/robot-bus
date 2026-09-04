"""Generated mapper for `nav_msgs/msg/MapMetaData`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import time_to_bus, time_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav_msgs.msg.v1 import MapMetaData as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def map_meta_data_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.map_load_time.CopyFrom(time_to_bus(msg.map_load_time))
    bus.resolution = msg.resolution
    bus.width = msg.width
    bus.height = msg.height
    bus.origin.CopyFrom(pose_to_bus(msg.origin))
    return bus


def map_meta_data_to_ros(bus):
    from nav_msgs.msg import MapMetaData as RosMsg

    out = RosMsg()
    out.map_load_time = time_to_ros(bus.map_load_time)
    out.resolution = bus.resolution
    out.width = bus.width
    out.height = bus.height
    out.origin = pose_to_ros(bus.origin)
    return out


class NavMsgsMapMetaDataMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav_msgs.msg import MapMetaData as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return map_meta_data_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return map_meta_data_to_ros(bus)
