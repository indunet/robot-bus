"""Generated mapper for `sensor_msgs/msg/JointState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def joint_state_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import JointState as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.name.extend([str(x) for x in msg.name])
    bus.position.extend(list(msg.position))
    bus.velocity.extend(list(msg.velocity))
    bus.effort.extend(list(msg.effort))
    return bus


def joint_state_to_ros(bus):
    from sensor_msgs.msg import JointState as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.name = [str(x) for x in bus.name]
    out.position = list(bus.position)
    out.velocity = list(bus.velocity)
    out.effort = list(bus.effort)
    return out


class SensorMsgsJointStateMapper:
    def ros_msg_type(self):
        from sensor_msgs.msg import JointState as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return joint_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import JointState as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_state_to_ros(bus)
