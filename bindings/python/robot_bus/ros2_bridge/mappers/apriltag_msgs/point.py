"""Generated mapper for `apriltag_msgs/msg/Point`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def point_to_bus(msg):
    from robot_bus.apriltag_msgs.msg.v1 import Point as BusMsg

    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    return bus


def point_to_ros(bus):
    from apriltag_msgs.msg import Point as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    return out


class ApriltagMsgsPointMapper:
    def type_name(self) -> str:
        return "apriltag_msgs/msg/Point"

    def ros_msg_type(self):
        from apriltag_msgs.msg import Point as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return point_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.apriltag_msgs.msg.v1 import Point as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return point_to_ros(bus)
