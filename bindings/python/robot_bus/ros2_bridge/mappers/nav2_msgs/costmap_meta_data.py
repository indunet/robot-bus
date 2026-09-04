"""Generated mapper for `nav2_msgs/msg/CostmapMetaData`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import time_to_bus, time_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import CostmapMetaData as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def costmap_meta_data_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.map_load_time.CopyFrom(time_to_bus(msg.map_load_time))
    bus.update_time.CopyFrom(time_to_bus(msg.update_time))
    bus.resolution = msg.resolution
    bus.size_x = msg.size_x
    bus.size_y = msg.size_y
    bus.origin.CopyFrom(pose_to_bus(msg.origin))
    bus.layer = str(msg.layer)
    return bus


def costmap_meta_data_to_ros(bus):
    from nav2_msgs.msg import CostmapMetaData as RosMsg

    out = RosMsg()
    out.map_load_time = time_to_ros(bus.map_load_time)
    out.update_time = time_to_ros(bus.update_time)
    out.resolution = bus.resolution
    out.size_x = bus.size_x
    out.size_y = bus.size_y
    out.origin = pose_to_ros(bus.origin)
    out.layer = str(bus.layer)
    return out


class Nav2MsgsCostmapMetaDataMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import CostmapMetaData as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return costmap_meta_data_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return costmap_meta_data_to_ros(bus)
