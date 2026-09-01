"""Generated mapper for `foxglove_msgs/msg/JointState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def joint_state_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import JointState as BusMsg

    bus = BusMsg()
    bus.name = str(msg.name)
    bus.position = msg.position
    bus.velocity = msg.velocity
    bus.acceleration = msg.acceleration
    bus.effort = msg.effort
    return bus


def joint_state_to_ros(bus):
    from foxglove_msgs.msg import JointState as RosMsg

    out = RosMsg()
    out.name = str(bus.name)
    out.position = bus.position
    out.velocity = bus.velocity
    out.acceleration = bus.acceleration
    out.effort = bus.effort
    return out


class FoxgloveMsgsJointStateMapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/JointState"

    def ros_msg_type(self):
        from foxglove_msgs.msg import JointState as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return joint_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import JointState as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_state_to_ros(bus)
