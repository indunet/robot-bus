"""Generated mapper for `foxglove_msgs/msg/FrameTransforms`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.frame_transform import frame_transform_to_bus, frame_transform_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import FrameTransforms as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def frame_transforms_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.transforms.extend([frame_transform_to_bus(x) for x in msg.transforms])
    return bus


def frame_transforms_to_ros(bus):
    from foxglove_msgs.msg import FrameTransforms as RosMsg

    out = RosMsg()
    out.transforms = [frame_transform_to_ros(x) for x in bus.transforms]
    return out


class FoxgloveMsgsFrameTransformsMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import FrameTransforms as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return frame_transforms_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return frame_transforms_to_ros(bus)
