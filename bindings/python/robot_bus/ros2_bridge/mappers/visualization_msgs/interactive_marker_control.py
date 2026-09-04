"""Generated mapper for `visualization_msgs/msg/InteractiveMarkerControl`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.quaternion import quaternion_to_bus, quaternion_to_ros
from robot_bus.ros2_bridge.mappers.visualization_msgs.marker import marker_to_bus, marker_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.visualization_msgs.msg.v1 import InteractiveMarkerControl as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def interactive_marker_control_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.name = str(msg.name)
    bus.orientation.CopyFrom(quaternion_to_bus(msg.orientation))
    bus.orientation_mode = msg.orientation_mode
    bus.interaction_mode = msg.interaction_mode
    bus.always_visible = msg.always_visible
    bus.markers.extend([marker_to_bus(x) for x in msg.markers])
    bus.independent_marker_orientation = msg.independent_marker_orientation
    bus.description = str(msg.description)
    return bus


def interactive_marker_control_to_ros(bus):
    from visualization_msgs.msg import InteractiveMarkerControl as RosMsg

    out = RosMsg()
    out.name = str(bus.name)
    out.orientation = quaternion_to_ros(bus.orientation)
    out.orientation_mode = bus.orientation_mode
    out.interaction_mode = bus.interaction_mode
    out.always_visible = bus.always_visible
    out.markers = [marker_to_ros(x) for x in bus.markers]
    out.independent_marker_orientation = bus.independent_marker_orientation
    out.description = str(bus.description)
    return out


class VisualizationMsgsInteractiveMarkerControlMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from visualization_msgs.msg import InteractiveMarkerControl as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return interactive_marker_control_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return interactive_marker_control_to_ros(bus)
