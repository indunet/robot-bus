"""Generated mapper for `nav2_msgs/msg/ParticleCloud`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.nav2_msgs.particle import particle_to_bus, particle_to_ros

def particle_cloud_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import ParticleCloud as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.particles.extend([particle_to_bus(x) for x in msg.particles])
    return bus


def particle_cloud_to_ros(bus):
    from nav2_msgs.msg import ParticleCloud as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.particles = [particle_to_ros(x) for x in bus.particles]
    return out


class Nav2MsgsParticleCloudMapper:
    def ros_msg_type(self):
        from nav2_msgs.msg import ParticleCloud as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return particle_cloud_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import ParticleCloud as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return particle_cloud_to_ros(bus)
