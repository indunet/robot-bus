"""Generated mapper for `control_msgs/msg/SingleDOFState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def single_dof_state_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import SingleDOFState as BusMsg

    bus = BusMsg()
    bus.name = str(msg.name)
    bus.reference = msg.reference
    bus.feedback = msg.feedback
    bus.feedback_dot = msg.feedback_dot
    bus.error = msg.error
    bus.error_dot = msg.error_dot
    bus.time_step = msg.time_step
    bus.output = msg.output
    return bus


def single_dof_state_to_ros(bus):
    from control_msgs.msg import SingleDOFState as RosMsg

    out = RosMsg()
    out.name = str(bus.name)
    out.reference = bus.reference
    out.feedback = bus.feedback
    out.feedback_dot = bus.feedback_dot
    out.error = bus.error
    out.error_dot = bus.error_dot
    out.time_step = bus.time_step
    out.output = bus.output
    return out


class ControlMsgsSingleDofStateMapper:
    def type_name(self) -> str:
        return "control_msgs/msg/SingleDOFState"

    def ros_msg_type(self):
        from control_msgs.msg import SingleDOFState as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return single_dof_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import SingleDOFState as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return single_dof_state_to_ros(bus)
