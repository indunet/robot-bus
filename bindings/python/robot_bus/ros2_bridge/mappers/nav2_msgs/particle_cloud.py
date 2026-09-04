"""Generated mapper for `nav2_msgs/msg/ParticleCloud`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.nav2_msgs.particle import particle_to_bus, particle_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import ParticleCloud as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def particle_cloud_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import ParticleCloud as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return particle_cloud_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return particle_cloud_to_ros(bus)
