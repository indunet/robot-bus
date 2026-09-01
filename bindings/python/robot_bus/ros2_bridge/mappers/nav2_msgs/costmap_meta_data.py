"""Generated mapper for `nav2_msgs/msg/CostmapMetaData`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import time_to_bus, time_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros

def costmap_meta_data_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import CostmapMetaData as BusMsg

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
    def ros_msg_type(self):
        from nav2_msgs.msg import CostmapMetaData as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return costmap_meta_data_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import CostmapMetaData as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return costmap_meta_data_to_ros(bus)
