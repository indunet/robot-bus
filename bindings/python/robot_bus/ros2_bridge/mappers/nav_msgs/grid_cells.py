"""Generated mapper for `nav_msgs/msg/GridCells`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros

def grid_cells_to_bus(msg):
    from robot_bus.nav_msgs.msg.v1 import GridCells as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.cell_width = msg.cell_width
    bus.cell_height = msg.cell_height
    bus.cells.extend([point_to_bus(x) for x in msg.cells])
    return bus


def grid_cells_to_ros(bus):
    from nav_msgs.msg import GridCells as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.cell_width = bus.cell_width
    out.cell_height = bus.cell_height
    out.cells = [point_to_ros(x) for x in bus.cells]
    return out


class NavMsgsGridCellsMapper:
    def type_name(self) -> str:
        return "nav_msgs/msg/GridCells"

    def ros_msg_type(self):
        from nav_msgs.msg import GridCells as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return grid_cells_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav_msgs.msg.v1 import GridCells as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return grid_cells_to_ros(bus)
