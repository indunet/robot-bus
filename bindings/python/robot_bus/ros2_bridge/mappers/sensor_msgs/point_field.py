"""Generated mapper for `sensor_msgs/msg/PointField`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import PointField as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def point_field_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.name = str(msg.name)
    bus.offset = msg.offset
    bus.datatype = int(msg.datatype)
    bus.count = msg.count
    return bus


def point_field_to_ros(bus):
    from sensor_msgs.msg import PointField as RosMsg

    out = RosMsg()
    out.name = str(bus.name)
    out.offset = bus.offset
    out.datatype = int(bus.datatype)
    out.count = bus.count
    return out


class SensorMsgsPointFieldMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import PointField as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return point_field_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return point_field_to_ros(bus)
