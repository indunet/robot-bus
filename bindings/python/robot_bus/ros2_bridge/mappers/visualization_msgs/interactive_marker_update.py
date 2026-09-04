"""Generated mapper for `visualization_msgs/msg/InteractiveMarkerUpdate`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.visualization_msgs.interactive_marker import interactive_marker_to_bus, interactive_marker_to_ros
from robot_bus.ros2_bridge.mappers.visualization_msgs.interactive_marker_pose import interactive_marker_pose_to_bus, interactive_marker_pose_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.visualization_msgs.msg.v1 import InteractiveMarkerUpdate as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def interactive_marker_update_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.server_id = str(msg.server_id)
    bus.seq_num = msg.seq_num
    bus.type = msg.type
    bus.markers.extend([interactive_marker_to_bus(x) for x in msg.markers])
    bus.poses.extend([interactive_marker_pose_to_bus(x) for x in msg.poses])
    bus.erases.extend([str(x) for x in msg.erases])
    return bus


def interactive_marker_update_to_ros(bus):
    from visualization_msgs.msg import InteractiveMarkerUpdate as RosMsg

    out = RosMsg()
    out.server_id = str(bus.server_id)
    out.seq_num = bus.seq_num
    out.type = bus.type
    out.markers = [interactive_marker_to_ros(x) for x in bus.markers]
    out.poses = [interactive_marker_pose_to_ros(x) for x in bus.poses]
    out.erases = [str(x) for x in bus.erases]
    return out


class VisualizationMsgsInteractiveMarkerUpdateMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from visualization_msgs.msg import InteractiveMarkerUpdate as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return interactive_marker_update_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return interactive_marker_update_to_ros(bus)
