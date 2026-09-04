"""Generated mapper for `nav2_msgs/msg/Particle`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import Particle as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def particle_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import Particle as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return particle_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return particle_to_ros(bus)
