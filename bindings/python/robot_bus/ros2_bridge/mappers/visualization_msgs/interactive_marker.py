"""Generated mapper for `visualization_msgs/msg/InteractiveMarker`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.visualization_msgs.menu_entry import menu_entry_to_bus, menu_entry_to_ros
from robot_bus.ros2_bridge.mappers.visualization_msgs.interactive_marker_control import interactive_marker_control_to_bus, interactive_marker_control_to_ros

def interactive_marker_to_bus(msg):
    from robot_bus.visualization_msgs.msg.v1 import InteractiveMarker as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.name = str(msg.name)
    bus.description = str(msg.description)
    bus.scale = msg.scale
    bus.menu_entries.extend([menu_entry_to_bus(x) for x in msg.menu_entries])
    bus.controls.extend([interactive_marker_control_to_bus(x) for x in msg.controls])
    return bus


def interactive_marker_to_ros(bus):
    from visualization_msgs.msg import InteractiveMarker as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.pose = pose_to_ros(bus.pose)
    out.name = str(bus.name)
    out.description = str(bus.description)
    out.scale = bus.scale
    out.menu_entries = [menu_entry_to_ros(x) for x in bus.menu_entries]
    out.controls = [interactive_marker_control_to_ros(x) for x in bus.controls]
    return out


class VisualizationMsgsInteractiveMarkerMapper:
    def type_name(self) -> str:
        return "visualization_msgs/msg/InteractiveMarker"

    def ros_msg_type(self):
        from visualization_msgs.msg import InteractiveMarker as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return interactive_marker_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.visualization_msgs.msg.v1 import InteractiveMarker as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return interactive_marker_to_ros(bus)
