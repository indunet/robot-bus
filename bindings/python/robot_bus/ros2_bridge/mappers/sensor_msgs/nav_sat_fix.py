"""Generated mapper for `sensor_msgs/msg/NavSatFix`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.sensor_msgs.nav_sat_status import nav_sat_status_to_bus, nav_sat_status_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import NavSatFix as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def nav_sat_fix_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.status.CopyFrom(nav_sat_status_to_bus(msg.status))
    bus.latitude = msg.latitude
    bus.longitude = msg.longitude
    bus.altitude = msg.altitude
    bus.position_covariance.extend(list(msg.position_covariance))
    bus.position_covariance_type = msg.position_covariance_type
    return bus


def nav_sat_fix_to_ros(bus):
    from sensor_msgs.msg import NavSatFix as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.status = nav_sat_status_to_ros(bus.status)
    out.latitude = bus.latitude
    out.longitude = bus.longitude
    out.altitude = bus.altitude
    out.position_covariance = list(bus.position_covariance)
    out.position_covariance_type = bus.position_covariance_type
    return out


class SensorMsgsNavSatFixMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import NavSatFix as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return nav_sat_fix_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return nav_sat_fix_to_ros(bus)
