"""Generated mapper for `nav2_msgs/msg/Particle`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros

def particle_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import Particle as BusMsg

    bus = BusMsg()
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.weight = msg.weight
    return bus


def particle_to_ros(bus):
    from nav2_msgs.msg import Particle as RosMsg

    out = RosMsg()
    out.pose = pose_to_ros(bus.pose)
    out.weight = bus.weight
    return out


class Nav2MsgsParticleMapper:
    def ros_msg_type(self):
        from nav2_msgs.msg import Particle as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return particle_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import Particle as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return particle_to_ros(bus)
