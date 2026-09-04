"""Generated mapper for `control_msgs/msg/JointTolerance`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import JointTolerance as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def joint_tolerance_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.name = str(msg.name)
    bus.position = msg.position
    bus.velocity = msg.velocity
    bus.acceleration = msg.acceleration
    return bus


def joint_tolerance_to_ros(bus):
    from control_msgs.msg import JointTolerance as RosMsg

    out = RosMsg()
    out.name = str(bus.name)
    out.position = bus.position
    out.velocity = bus.velocity
    out.acceleration = bus.acceleration
    return out


class ControlMsgsJointToleranceMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import JointTolerance as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return joint_tolerance_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_tolerance_to_ros(bus)
