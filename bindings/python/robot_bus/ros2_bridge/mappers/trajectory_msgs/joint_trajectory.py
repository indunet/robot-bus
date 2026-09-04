"""Generated mapper for `trajectory_msgs/msg/JointTrajectory`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.trajectory_msgs.joint_trajectory_point import joint_trajectory_point_to_bus, joint_trajectory_point_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.trajectory_msgs.msg.v1 import JointTrajectory as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def joint_trajectory_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.joint_names.extend([str(x) for x in msg.joint_names])
    bus.points.extend([joint_trajectory_point_to_bus(x) for x in msg.points])
    return bus


def joint_trajectory_to_ros(bus):
    from trajectory_msgs.msg import JointTrajectory as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.joint_names = [str(x) for x in bus.joint_names]
    out.points = [joint_trajectory_point_to_ros(x) for x in bus.points]
    return out


class TrajectoryMsgsJointTrajectoryMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from trajectory_msgs.msg import JointTrajectory as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return joint_trajectory_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_trajectory_to_ros(bus)
