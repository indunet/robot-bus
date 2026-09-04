"""Generated mapper for `shape_msgs/msg/SolidPrimitive`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.polygon import polygon_to_bus, polygon_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.shape_msgs.msg.v1 import SolidPrimitive as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def solid_primitive_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.type = msg.type
    bus.dimensions.extend(list(msg.dimensions))
    bus.polygon.CopyFrom(polygon_to_bus(msg.polygon))
    return bus


def solid_primitive_to_ros(bus):
    from shape_msgs.msg import SolidPrimitive as RosMsg

    out = RosMsg()
    out.type = bus.type
    out.dimensions = list(bus.dimensions)
    out.polygon = polygon_to_ros(bus.polygon)
    return out


class ShapeMsgsSolidPrimitiveMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from shape_msgs.msg import SolidPrimitive as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return solid_primitive_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return solid_primitive_to_ros(bus)
