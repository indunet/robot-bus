"""Generated mapper for `sensor_msgs/msg/LaserScan`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def laser_scan_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import LaserScan as BusMsg

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
    def type_name(self) -> str:
        return "sensor_msgs/msg/LaserScan"

    def ros_msg_type(self):
        from sensor_msgs.msg import LaserScan as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return laser_scan_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import LaserScan as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return laser_scan_to_ros(bus)
