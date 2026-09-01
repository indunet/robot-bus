"""Generated mapper for `control_msgs/msg/MultiDOFCommand`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def multi_dof_command_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import MultiDOFCommand as BusMsg

    bus = BusMsg()
    bus.dof_names.extend([str(x) for x in msg.dof_names])
    bus.values.extend(list(msg.values))
    bus.values_dot.extend(list(msg.values_dot))
    return bus


def multi_dof_command_to_ros(bus):
    from control_msgs.msg import MultiDOFCommand as RosMsg

    out = RosMsg()
    out.dof_names = [str(x) for x in bus.dof_names]
    out.values = list(bus.values)
    out.values_dot = list(bus.values_dot)
    return out


class ControlMsgsMultiDofCommandMapper:
    def ros_msg_type(self):
        from control_msgs.msg import MultiDOFCommand as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return multi_dof_command_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import MultiDOFCommand as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return multi_dof_command_to_ros(bus)
