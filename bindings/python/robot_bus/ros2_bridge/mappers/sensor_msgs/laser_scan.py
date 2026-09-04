"""Generated mapper for `sensor_msgs/msg/LaserScan`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import LaserScan as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def laser_scan_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.angle_min = msg.angle_min
    bus.angle_max = msg.angle_max
    bus.angle_increment = msg.angle_increment
    bus.time_increment = msg.time_increment
    bus.scan_time = msg.scan_time
    bus.range_min = msg.range_min
    bus.range_max = msg.range_max
    bus.ranges.extend(list(msg.ranges))
    bus.intensities.extend(list(msg.intensities))
    return bus


def laser_scan_to_ros(bus):
    from sensor_msgs.msg import LaserScan as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.angle_min = bus.angle_min
    out.angle_max = bus.angle_max
    out.angle_increment = bus.angle_increment
    out.time_increment = bus.time_increment
    out.scan_time = bus.scan_time
    out.range_min = bus.range_min
    out.range_max = bus.range_max
    out.ranges = list(bus.ranges)
    out.intensities = list(bus.intensities)
    return out


class SensorMsgsLaserScanMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import LaserScan as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return laser_scan_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return laser_scan_to_ros(bus)
