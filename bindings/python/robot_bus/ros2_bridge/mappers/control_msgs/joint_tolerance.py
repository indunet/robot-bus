"""Generated mapper for `control_msgs/msg/JointTolerance`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def joint_tolerance_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import JointTolerance as BusMsg

    bus = BusMsg()
    bus.name = str(msg.name)
    bus.position = msg.position
    bus.velocity = msg.velocity
    bus.acceleration = msg.acceleration
    return bus


def joint_tolerance_to_ros(bus):
    from control_msgs.msg import JointTolerance as RosMsg

    out = RosMsg()
    out.name = str(bus.name)
    out.position = bus.position
    out.velocity = bus.velocity
    out.acceleration = bus.acceleration
    return out


class ControlMsgsJointToleranceMapper:
    def ros_msg_type(self):
        from control_msgs.msg import JointTolerance as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return joint_tolerance_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import JointTolerance as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_tolerance_to_ros(bus)
