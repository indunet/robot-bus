"""Generated mapper for `geometry_msgs/msg/Accel`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.geometry_msgs.msg.v1 import Accel as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def accel_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.linear.CopyFrom(vector3_to_bus(msg.linear))
    bus.angular.CopyFrom(vector3_to_bus(msg.angular))
    return bus


def accel_to_ros(bus):
    from geometry_msgs.msg import Accel as RosMsg

    out = RosMsg()
    out.linear = vector3_to_ros(bus.linear)
    out.angular = vector3_to_ros(bus.angular)
    return out


class GeometryMsgsAccelMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from geometry_msgs.msg import Accel as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return accel_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return accel_to_ros(bus)
