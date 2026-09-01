"""Generated mapper for `control_msgs/msg/JointJog`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def joint_jog_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import JointJog as BusMsg

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
    def ros_msg_type(self):
        from control_msgs.msg import JointJog as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return joint_jog_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import JointJog as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_jog_to_ros(bus)
