"""Generated mapper for `geometry_msgs/msg/Inertia`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros

def inertia_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import Inertia as BusMsg

    bus = BusMsg()
    bus.m = msg.m
    bus.com.CopyFrom(vector3_to_bus(msg.com))
    bus.ixx = msg.ixx
    bus.ixy = msg.ixy
    bus.ixz = msg.ixz
    bus.iyy = msg.iyy
    bus.iyz = msg.iyz
    bus.izz = msg.izz
    return bus


def inertia_to_ros(bus):
    from geometry_msgs.msg import Inertia as RosMsg

    out = RosMsg()
    out.m = bus.m
    out.com = vector3_to_ros(bus.com)
    out.ixx = bus.ixx
    out.ixy = bus.ixy
    out.ixz = bus.ixz
    out.iyy = bus.iyy
    out.iyz = bus.iyz
    out.izz = bus.izz
    return out


class GeometryMsgsInertiaMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/Inertia"

    def ros_msg_type(self):
        from geometry_msgs.msg import Inertia as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return inertia_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import Inertia as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return inertia_to_ros(bus)
