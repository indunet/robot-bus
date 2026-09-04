"""Generated mapper for `foxglove_msgs/msg/FrameTransform`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector3 import vector3_to_bus, vector3_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.quaternion import quaternion_to_bus, quaternion_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import FrameTransform as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def frame_transform_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.parent_frame_id = str(msg.parent_frame_id)
    bus.child_frame_id = str(msg.child_frame_id)
    bus.translation.CopyFrom(vector3_to_bus(msg.translation))
    bus.rotation.CopyFrom(quaternion_to_bus(msg.rotation))
    return bus


def frame_transform_to_ros(bus):
    from foxglove_msgs.msg import FrameTransform as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.parent_frame_id = str(bus.parent_frame_id)
    out.child_frame_id = str(bus.child_frame_id)
    out.translation = vector3_to_ros(bus.translation)
    out.rotation = quaternion_to_ros(bus.rotation)
    return out


class FoxgloveMsgsFrameTransformMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import FrameTransform as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return frame_transform_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return frame_transform_to_ros(bus)
