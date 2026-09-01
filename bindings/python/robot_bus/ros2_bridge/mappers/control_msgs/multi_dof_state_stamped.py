"""Generated mapper for `control_msgs/msg/MultiDOFStateStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.control_msgs.single_dof_state import single_dof_state_to_bus, single_dof_state_to_ros

def multi_dof_state_stamped_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import MultiDOFStateStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.dof_states.extend([single_dof_state_to_bus(x) for x in msg.dof_states])
    return bus


def multi_dof_state_stamped_to_ros(bus):
    from control_msgs.msg import MultiDOFStateStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.dof_states = [single_dof_state_to_ros(x) for x in bus.dof_states]
    return out


class ControlMsgsMultiDofStateStampedMapper:
    def type_name(self) -> str:
        return "control_msgs/msg/MultiDOFStateStamped"

    def ros_msg_type(self):
        from control_msgs.msg import MultiDOFStateStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return multi_dof_state_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import MultiDOFStateStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return multi_dof_state_stamped_to_ros(bus)
