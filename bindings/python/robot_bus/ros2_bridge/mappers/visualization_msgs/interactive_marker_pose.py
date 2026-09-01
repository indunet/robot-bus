"""Generated mapper for `visualization_msgs/msg/InteractiveMarkerPose`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros

def interactive_marker_pose_to_bus(msg):
    from robot_bus.visualization_msgs.msg.v1 import InteractiveMarkerPose as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.name = str(msg.name)
    return bus


def interactive_marker_pose_to_ros(bus):
    from visualization_msgs.msg import InteractiveMarkerPose as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.pose = pose_to_ros(bus.pose)
    out.name = str(bus.name)
    return out


class VisualizationMsgsInteractiveMarkerPoseMapper:
    def type_name(self) -> str:
        return "visualization_msgs/msg/InteractiveMarkerPose"

    def ros_msg_type(self):
        from visualization_msgs.msg import InteractiveMarkerPose as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return interactive_marker_pose_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.visualization_msgs.msg.v1 import InteractiveMarkerPose as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return interactive_marker_pose_to_ros(bus)
