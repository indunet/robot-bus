"""Generated mapper for `foxglove_msgs/msg/JointStates`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.joint_state import joint_state_to_bus, joint_state_to_ros

def joint_states_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import JointStates as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.joints.extend([joint_state_to_bus(x) for x in msg.joints])
    return bus


def joint_states_to_ros(bus):
    from foxglove_msgs.msg import JointStates as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.joints = [joint_state_to_ros(x) for x in bus.joints]
    return out


class FoxgloveMsgsJointStatesMapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import JointStates as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return joint_states_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import JointStates as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_states_to_ros(bus)
