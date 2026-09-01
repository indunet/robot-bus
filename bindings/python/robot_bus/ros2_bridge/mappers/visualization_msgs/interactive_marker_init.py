"""Generated mapper for `visualization_msgs/msg/InteractiveMarkerInit`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.visualization_msgs.interactive_marker import interactive_marker_to_bus, interactive_marker_to_ros

def interactive_marker_init_to_bus(msg):
    from robot_bus.visualization_msgs.msg.v1 import InteractiveMarkerInit as BusMsg

    bus = BusMsg()
    bus.server_id = str(msg.server_id)
    bus.seq_num = msg.seq_num
    bus.markers.extend([interactive_marker_to_bus(x) for x in msg.markers])
    return bus


def interactive_marker_init_to_ros(bus):
    from visualization_msgs.msg import InteractiveMarkerInit as RosMsg

    out = RosMsg()
    out.server_id = str(bus.server_id)
    out.seq_num = bus.seq_num
    out.markers = [interactive_marker_to_ros(x) for x in bus.markers]
    return out


class VisualizationMsgsInteractiveMarkerInitMapper:
    def type_name(self) -> str:
        return "visualization_msgs/msg/InteractiveMarkerInit"

    def ros_msg_type(self):
        from visualization_msgs.msg import InteractiveMarkerInit as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return interactive_marker_init_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.visualization_msgs.msg.v1 import InteractiveMarkerInit as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return interactive_marker_init_to_ros(bus)
