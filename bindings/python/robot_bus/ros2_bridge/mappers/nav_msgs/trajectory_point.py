"""Generated mapper for `nav_msgs/msg/TrajectoryPoint`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist import twist_to_bus, twist_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.accel import accel_to_bus, accel_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.wrench import wrench_to_bus, wrench_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav_msgs.msg.v1 import TrajectoryPoint as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def trajectory_point_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.velocity.CopyFrom(twist_to_bus(msg.velocity))
    bus.acceleration.CopyFrom(accel_to_bus(msg.acceleration))
    bus.effort.CopyFrom(wrench_to_bus(msg.effort))
    return bus


def trajectory_point_to_ros(bus):
    from nav_msgs.msg import TrajectoryPoint as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.pose = pose_to_ros(bus.pose)
    out.velocity = twist_to_ros(bus.velocity)
    out.acceleration = accel_to_ros(bus.acceleration)
    out.effort = wrench_to_ros(bus.effort)
    return out


class NavMsgsTrajectoryPointMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav_msgs.msg import TrajectoryPoint as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return trajectory_point_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return trajectory_point_to_ros(bus)
