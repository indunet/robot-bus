"""Generated mapper for `control_msgs/msg/SingleDOFStateStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.control_msgs.single_dof_state import single_dof_state_to_bus, single_dof_state_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import SingleDOFStateStamped as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def single_dof_state_stamped_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import SingleDOFStateStamped as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return single_dof_state_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return single_dof_state_stamped_to_ros(bus)
