"""Generated mapper for `geometry_msgs/msg/Quaternion`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def quaternion_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import Quaternion as BusMsg

    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    bus.z = msg.z
    bus.w = msg.w
    return bus


def quaternion_to_ros(bus):
    from geometry_msgs.msg import Quaternion as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    out.z = bus.z
    out.w = bus.w
    return out


class GeometryMsgsQuaternionMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/Quaternion"

    def ros_msg_type(self):
        from geometry_msgs.msg import Quaternion as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return quaternion_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import Quaternion as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return quaternion_to_ros(bus)
