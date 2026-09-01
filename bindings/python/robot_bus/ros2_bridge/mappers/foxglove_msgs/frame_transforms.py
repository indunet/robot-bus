"""Generated mapper for `foxglove_msgs/msg/FrameTransforms`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.frame_transform import frame_transform_to_bus, frame_transform_to_ros

def frame_transforms_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import FrameTransforms as BusMsg

    bus = BusMsg()
    bus.transforms.extend([frame_transform_to_bus(x) for x in msg.transforms])
    return bus


def frame_transforms_to_ros(bus):
    from foxglove_msgs.msg import FrameTransforms as RosMsg

    out = RosMsg()
    out.transforms = [frame_transform_to_ros(x) for x in bus.transforms]
    return out


class FoxgloveMsgsFrameTransformsMapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/FrameTransforms"

    def ros_msg_type(self):
        from foxglove_msgs.msg import FrameTransforms as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return frame_transforms_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import FrameTransforms as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return frame_transforms_to_ros(bus)
