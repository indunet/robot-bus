"""Generated mapper for `trajectory_msgs/msg/JointTrajectoryPoint`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.builtin_interfaces.duration import duration_to_bus, duration_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.trajectory_msgs.msg.v1 import JointTrajectoryPoint as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def joint_trajectory_point_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.positions.extend(list(msg.positions))
    bus.velocities.extend(list(msg.velocities))
    bus.accelerations.extend(list(msg.accelerations))
    bus.effort.extend(list(msg.effort))
    bus.time_from_start.CopyFrom(duration_to_bus(msg.time_from_start))
    return bus


def joint_trajectory_point_to_ros(bus):
    from trajectory_msgs.msg import JointTrajectoryPoint as RosMsg

    out = RosMsg()
    out.positions = list(bus.positions)
    out.velocities = list(bus.velocities)
    out.accelerations = list(bus.accelerations)
    out.effort = list(bus.effort)
    out.time_from_start = duration_to_ros(bus.time_from_start)
    return out


class TrajectoryMsgsJointTrajectoryPointMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from trajectory_msgs.msg import JointTrajectoryPoint as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return joint_trajectory_point_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_trajectory_point_to_ros(bus)
