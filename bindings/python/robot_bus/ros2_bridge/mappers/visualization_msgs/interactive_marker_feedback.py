"""Generated mapper for `visualization_msgs/msg/InteractiveMarkerFeedback`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.visualization_msgs.msg.v1 import InteractiveMarkerFeedback as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def interactive_marker_feedback_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from visualization_msgs.msg import InteractiveMarkerFeedback as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return interactive_marker_feedback_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return interactive_marker_feedback_to_ros(bus)
