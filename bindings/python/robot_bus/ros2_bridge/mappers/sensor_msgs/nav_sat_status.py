"""Generated mapper for `sensor_msgs/msg/NavSatStatus`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import NavSatStatus as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def nav_sat_status_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.status = int(msg.status)
    bus.service = int(msg.service)
    return bus


def nav_sat_status_to_ros(bus):
    from sensor_msgs.msg import NavSatStatus as RosMsg

    out = RosMsg()
    out.status = int(bus.status)
    out.service = int(bus.service)
    return out


class SensorMsgsNavSatStatusMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import NavSatStatus as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return nav_sat_status_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return nav_sat_status_to_ros(bus)
