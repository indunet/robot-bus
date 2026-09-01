"""Generated mapper for `foxglove_msgs/msg/ImageAnnotations`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.circle_annotation import circle_annotation_to_bus, circle_annotation_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.points_annotation import points_annotation_to_bus, points_annotation_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.text_annotation import text_annotation_to_bus, text_annotation_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.key_value_pair import key_value_pair_to_bus, key_value_pair_to_ros

def image_annotations_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import ImageAnnotations as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.circles.extend([circle_annotation_to_bus(x) for x in msg.circles])
    bus.points.extend([points_annotation_to_bus(x) for x in msg.points])
    bus.texts.extend([text_annotation_to_bus(x) for x in msg.texts])
    bus.metadata.extend([key_value_pair_to_bus(x) for x in msg.metadata])
    return bus


def image_annotations_to_ros(bus):
    from foxglove_msgs.msg import ImageAnnotations as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.circles = [circle_annotation_to_ros(x) for x in bus.circles]
    out.points = [points_annotation_to_ros(x) for x in bus.points]
    out.texts = [text_annotation_to_ros(x) for x in bus.texts]
    out.metadata = [key_value_pair_to_ros(x) for x in bus.metadata]
    return out


class FoxgloveMsgsImageAnnotationsMapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import ImageAnnotations as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return image_annotations_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import ImageAnnotations as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return image_annotations_to_ros(bus)
