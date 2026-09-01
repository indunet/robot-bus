"""Generated mapper for `geometry_msgs/msg/VelocityStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist import twist_to_bus, twist_to_ros

def velocity_stamped_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import VelocityStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.body_frame_id = str(msg.body_frame_id)
    bus.reference_frame_id = str(msg.reference_frame_id)
    bus.velocity.CopyFrom(twist_to_bus(msg.velocity))
    return bus


def velocity_stamped_to_ros(bus):
    from geometry_msgs.msg import VelocityStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.body_frame_id = str(bus.body_frame_id)
    out.reference_frame_id = str(bus.reference_frame_id)
    out.velocity = twist_to_ros(bus.velocity)
    return out


class GeometryMsgsVelocityStampedMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import VelocityStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return velocity_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import VelocityStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return velocity_stamped_to_ros(bus)
