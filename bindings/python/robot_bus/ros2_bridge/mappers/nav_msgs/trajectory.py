"""Generated mapper for `nav_msgs/msg/Trajectory`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.nav_msgs.trajectory_point import trajectory_point_to_bus, trajectory_point_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav_msgs.msg.v1 import Trajectory as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def trajectory_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.points.extend([trajectory_point_to_bus(x) for x in msg.points])
    return bus


def trajectory_to_ros(bus):
    from nav_msgs.msg import Trajectory as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.points = [trajectory_point_to_ros(x) for x in bus.points]
    return out


class NavMsgsTrajectoryMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav_msgs.msg import Trajectory as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return trajectory_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return trajectory_to_ros(bus)
