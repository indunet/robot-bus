"""Generated mapper for `geometry_msgs/msg/PoseArray`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros

def pose_array_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import PoseArray as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.poses.extend([pose_to_bus(x) for x in msg.poses])
    return bus


def pose_array_to_ros(bus):
    from geometry_msgs.msg import PoseArray as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.poses = [pose_to_ros(x) for x in bus.poses]
    return out


class GeometryMsgsPoseArrayMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import PoseArray as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return pose_array_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import PoseArray as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return pose_array_to_ros(bus)
