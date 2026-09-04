"""Generated mapper for `geometry_msgs/msg/Inertia`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.geometry_msgs.msg.v1 import Inertia as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def inertia_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from geometry_msgs.msg import Inertia as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return inertia_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return inertia_to_ros(bus)
