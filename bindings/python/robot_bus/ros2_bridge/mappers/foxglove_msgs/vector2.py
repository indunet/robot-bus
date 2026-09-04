"""Generated mapper for `foxglove_msgs/msg/Vector2`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import Vector2 as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def vector2_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    return bus


def vector2_to_ros(bus):
    from foxglove_msgs.msg import Vector2 as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    return out


class FoxgloveMsgsVector2Mapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import Vector2 as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return vector2_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return vector2_to_ros(bus)
