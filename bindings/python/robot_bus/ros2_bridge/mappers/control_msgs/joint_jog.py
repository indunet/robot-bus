"""Generated mapper for `control_msgs/msg/JointJog`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import JointJog as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def joint_jog_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.joint_names.extend([str(x) for x in msg.joint_names])
    bus.displacements.extend(list(msg.displacements))
    bus.velocities.extend(list(msg.velocities))
    bus.duration = msg.duration
    return bus


def joint_jog_to_ros(bus):
    from control_msgs.msg import JointJog as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.joint_names = [str(x) for x in bus.joint_names]
    out.displacements = list(bus.displacements)
    out.velocities = list(bus.velocities)
    out.duration = bus.duration
    return out


class ControlMsgsJointJogMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import JointJog as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return joint_jog_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_jog_to_ros(bus)
