"""Generated mapper for `foxglove_msgs/msg/PoseInFrame`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros

def pose_in_frame_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import PoseInFrame as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    return bus


def pose_in_frame_to_ros(bus):
    from foxglove_msgs.msg import PoseInFrame as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.pose = pose_to_ros(bus.pose)
    return out


class FoxgloveMsgsPoseInFrameMapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/PoseInFrame"

    def ros_msg_type(self):
        from foxglove_msgs.msg import PoseInFrame as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return pose_in_frame_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import PoseInFrame as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return pose_in_frame_to_ros(bus)
