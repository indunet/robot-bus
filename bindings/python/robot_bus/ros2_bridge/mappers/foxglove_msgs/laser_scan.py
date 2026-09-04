"""Generated mapper for `foxglove_msgs/msg/LaserScan`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import LaserScan as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def laser_scan_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.start_angle = msg.start_angle
    bus.end_angle = msg.end_angle
    bus.ranges.extend(list(msg.ranges))
    bus.intensities.extend(list(msg.intensities))
    return bus


def laser_scan_to_ros(bus):
    from foxglove_msgs.msg import LaserScan as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.pose = pose_to_ros(bus.pose)
    out.start_angle = bus.start_angle
    out.end_angle = bus.end_angle
    out.ranges = list(bus.ranges)
    out.intensities = list(bus.intensities)
    return out


class FoxgloveMsgsLaserScanMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import LaserScan as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return laser_scan_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return laser_scan_to_ros(bus)
