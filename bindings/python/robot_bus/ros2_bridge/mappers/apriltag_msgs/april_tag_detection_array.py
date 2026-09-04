"""Generated mapper for `apriltag_msgs/msg/AprilTagDetectionArray`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.apriltag_msgs.april_tag_detection import april_tag_detection_to_bus, april_tag_detection_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.apriltag_msgs.msg.v1 import AprilTagDetectionArray as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def april_tag_detection_array_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.detections.extend([april_tag_detection_to_bus(x) for x in msg.detections])
    return bus


def april_tag_detection_array_to_ros(bus):
    from apriltag_msgs.msg import AprilTagDetectionArray as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.detections = [april_tag_detection_to_ros(x) for x in bus.detections]
    return out


class ApriltagMsgsAprilTagDetectionArrayMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from apriltag_msgs.msg import AprilTagDetectionArray as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return april_tag_detection_array_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return april_tag_detection_array_to_ros(bus)
