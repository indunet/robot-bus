"""Generated mapper for `trajectory_msgs/msg/MultiDOFJointTrajectoryPoint`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.transform import transform_to_bus, transform_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist import twist_to_bus, twist_to_ros
from robot_bus.ros2_bridge.mappers.builtin_interfaces.duration import duration_to_bus, duration_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.trajectory_msgs.msg.v1 import MultiDOFJointTrajectoryPoint as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def multi_dof_joint_trajectory_point_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.transforms.extend([transform_to_bus(x) for x in msg.transforms])
    bus.velocities.extend([twist_to_bus(x) for x in msg.velocities])
    bus.accelerations.extend([twist_to_bus(x) for x in msg.accelerations])
    bus.time_from_start.CopyFrom(duration_to_bus(msg.time_from_start))
    return bus


def multi_dof_joint_trajectory_point_to_ros(bus):
    from trajectory_msgs.msg import MultiDOFJointTrajectoryPoint as RosMsg

    out = RosMsg()
    out.transforms = [transform_to_ros(x) for x in bus.transforms]
    out.velocities = [twist_to_ros(x) for x in bus.velocities]
    out.accelerations = [twist_to_ros(x) for x in bus.accelerations]
    out.time_from_start = duration_to_ros(bus.time_from_start)
    return out


class TrajectoryMsgsMultiDofJointTrajectoryPointMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from trajectory_msgs.msg import MultiDOFJointTrajectoryPoint as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return multi_dof_joint_trajectory_point_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return multi_dof_joint_trajectory_point_to_ros(bus)
