"""Generated mapper for `geometry_msgs/msg/Vector3Stamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.geometry_msgs.msg.v1 import Vector3Stamped as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def vector3_stamped_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.vector.CopyFrom(vector3_to_bus(msg.vector))
    return bus


def vector3_stamped_to_ros(bus):
    from geometry_msgs.msg import Vector3Stamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.vector = vector3_to_ros(bus.vector)
    return out


class GeometryMsgsVector3StampedMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from geometry_msgs.msg import Vector3Stamped as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return vector3_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return vector3_stamped_to_ros(bus)
