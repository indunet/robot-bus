"""Generated mapper for `apriltag_msgs/msg/AprilTagDetection`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.apriltag_msgs.point import point_to_bus, point_to_ros

def april_tag_detection_to_bus(msg):
    from robot_bus.apriltag_msgs.msg.v1 import AprilTagDetection as BusMsg

    bus = BusMsg()
    bus.family = str(msg.family)
    bus.id = msg.id
    bus.hamming = msg.hamming
    bus.goodness = msg.goodness
    bus.decision_margin = msg.decision_margin
    bus.centre.CopyFrom(point_to_bus(msg.centre))
    bus.corners.extend([point_to_bus(x) for x in msg.corners])
    bus.homography.extend(list(msg.homography))
    return bus


def april_tag_detection_to_ros(bus):
    from apriltag_msgs.msg import AprilTagDetection as RosMsg

    out = RosMsg()
    out.family = str(bus.family)
    out.id = bus.id
    out.hamming = bus.hamming
    out.goodness = bus.goodness
    out.decision_margin = bus.decision_margin
    out.centre = point_to_ros(bus.centre)
    out.corners = [point_to_ros(x) for x in bus.corners]
    out.homography = list(bus.homography)
    return out


class ApriltagMsgsAprilTagDetectionMapper:
    def type_name(self) -> str:
        return "apriltag_msgs/msg/AprilTagDetection"

    def ros_msg_type(self):
        from apriltag_msgs.msg import AprilTagDetection as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return april_tag_detection_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.apriltag_msgs.msg.v1 import AprilTagDetection as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return april_tag_detection_to_ros(bus)
