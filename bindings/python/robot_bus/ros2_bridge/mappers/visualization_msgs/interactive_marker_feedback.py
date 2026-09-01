"""Generated mapper for `visualization_msgs/msg/InteractiveMarkerFeedback`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros

def interactive_marker_feedback_to_bus(msg):
    from robot_bus.visualization_msgs.msg.v1 import InteractiveMarkerFeedback as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.client_id = str(msg.client_id)
    bus.marker_name = str(msg.marker_name)
    bus.control_name = str(msg.control_name)
    bus.event_type = msg.event_type
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.menu_entry_id = msg.menu_entry_id
    bus.mouse_point.CopyFrom(point_to_bus(msg.mouse_point))
    bus.mouse_point_valid = msg.mouse_point_valid
    return bus


def interactive_marker_feedback_to_ros(bus):
    from visualization_msgs.msg import InteractiveMarkerFeedback as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.client_id = str(bus.client_id)
    out.marker_name = str(bus.marker_name)
    out.control_name = str(bus.control_name)
    out.event_type = bus.event_type
    out.pose = pose_to_ros(bus.pose)
    out.menu_entry_id = bus.menu_entry_id
    out.mouse_point = point_to_ros(bus.mouse_point)
    out.mouse_point_valid = bus.mouse_point_valid
    return out


class VisualizationMsgsInteractiveMarkerFeedbackMapper:
    def type_name(self) -> str:
        return "visualization_msgs/msg/InteractiveMarkerFeedback"

    def ros_msg_type(self):
        from visualization_msgs.msg import InteractiveMarkerFeedback as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return interactive_marker_feedback_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.visualization_msgs.msg.v1 import InteractiveMarkerFeedback as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return interactive_marker_feedback_to_ros(bus)
