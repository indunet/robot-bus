"""Generated mapper for `nav2_msgs/msg/VoxelGrid`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point32 import point32_to_bus, point32_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import VoxelGrid as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def voxel_grid_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.data.extend(list(msg.data))
    bus.origin.CopyFrom(point32_to_bus(msg.origin))
    bus.resolutions.CopyFrom(vector3_to_bus(msg.resolutions))
    bus.size_x = msg.size_x
    bus.size_y = msg.size_y
    bus.size_z = msg.size_z
    return bus


def voxel_grid_to_ros(bus):
    from nav2_msgs.msg import VoxelGrid as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.data = list(bus.data)
    out.origin = point32_to_ros(bus.origin)
    out.resolutions = vector3_to_ros(bus.resolutions)
    out.size_x = bus.size_x
    out.size_y = bus.size_y
    out.size_z = bus.size_z
    return out


class Nav2MsgsVoxelGridMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import VoxelGrid as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return voxel_grid_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return voxel_grid_to_ros(bus)
