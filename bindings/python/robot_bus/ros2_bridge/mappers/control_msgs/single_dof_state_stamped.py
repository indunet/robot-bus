"""Generated mapper for `control_msgs/msg/SingleDOFStateStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.control_msgs.single_dof_state import single_dof_state_to_bus, single_dof_state_to_ros

def single_dof_state_stamped_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import SingleDOFStateStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.state.CopyFrom(single_dof_state_to_bus(msg.state))
    return bus


def single_dof_state_stamped_to_ros(bus):
    from control_msgs.msg import SingleDOFStateStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.state = single_dof_state_to_ros(bus.state)
    return out


class ControlMsgsSingleDofStateStampedMapper:
    def ros_msg_type(self):
        from control_msgs.msg import SingleDOFStateStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return single_dof_state_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import SingleDOFStateStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return single_dof_state_stamped_to_ros(bus)
