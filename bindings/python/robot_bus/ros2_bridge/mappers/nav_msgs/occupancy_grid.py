"""Generated mapper for `nav_msgs/msg/OccupancyGrid`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.nav_msgs.map_meta_data import map_meta_data_to_bus, map_meta_data_to_ros

def occupancy_grid_to_bus(msg):
    from robot_bus.nav_msgs.msg.v1 import OccupancyGrid as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.info.CopyFrom(map_meta_data_to_bus(msg.info))
    bus.data = _convert.i8_seq_to_bytes(msg.data)
    return bus


def occupancy_grid_to_ros(bus):
    from nav_msgs.msg import OccupancyGrid as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.info = map_meta_data_to_ros(bus.info)
    out.data = _convert.bytes_to_i8_seq(bus.data)
    return out


class NavMsgsOccupancyGridMapper:
    def ros_msg_type(self):
        from nav_msgs.msg import OccupancyGrid as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return occupancy_grid_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav_msgs.msg.v1 import OccupancyGrid as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return occupancy_grid_to_ros(bus)
