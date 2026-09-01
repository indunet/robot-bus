"""Generated mapper for `visualization_msgs/msg/MarkerArray`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.visualization_msgs.marker import marker_to_bus, marker_to_ros

def marker_array_to_bus(msg):
    from robot_bus.visualization_msgs.msg.v1 import MarkerArray as BusMsg

    bus = BusMsg()
    bus.markers.extend([marker_to_bus(x) for x in msg.markers])
    return bus


def marker_array_to_ros(bus):
    from visualization_msgs.msg import MarkerArray as RosMsg

    out = RosMsg()
    out.markers = [marker_to_ros(x) for x in bus.markers]
    return out


class VisualizationMsgsMarkerArrayMapper:
    def ros_msg_type(self):
        from visualization_msgs.msg import MarkerArray as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return marker_array_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.visualization_msgs.msg.v1 import MarkerArray as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return marker_array_to_ros(bus)
