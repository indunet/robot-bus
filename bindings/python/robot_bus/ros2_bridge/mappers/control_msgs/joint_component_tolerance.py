"""Generated mapper for `control_msgs/msg/JointComponentTolerance`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def joint_component_tolerance_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import JointComponentTolerance as BusMsg

    bus = BusMsg()
    bus.joint_name = str(msg.joint_name)
    bus.component = msg.component
    bus.value = msg.value
    return bus


def joint_component_tolerance_to_ros(bus):
    from control_msgs.msg import JointComponentTolerance as RosMsg

    out = RosMsg()
    out.joint_name = str(bus.joint_name)
    out.component = bus.component
    out.value = bus.value
    return out


class ControlMsgsJointComponentToleranceMapper:
    def ros_msg_type(self):
        from control_msgs.msg import JointComponentTolerance as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return joint_component_tolerance_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import JointComponentTolerance as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_component_tolerance_to_ros(bus)
