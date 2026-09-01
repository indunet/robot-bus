"""Generated mapper for `geometry_msgs/msg/Pose2D`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def pose2_d_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import Pose2D as BusMsg

    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    bus.theta = msg.theta
    return bus


def pose2_d_to_ros(bus):
    from geometry_msgs.msg import Pose2D as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    out.theta = bus.theta
    return out


class GeometryMsgsPose2DMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import Pose2D as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return pose2_d_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import Pose2D as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return pose2_d_to_ros(bus)
