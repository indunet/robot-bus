"""Generated mapper for `control_msgs/msg/JointComponentTolerance`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import JointComponentTolerance as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def joint_component_tolerance_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.joint_name = str(msg.joint_name)
    bus.component = msg.component
    bus.value = msg.value
    return bus


def joint_component_tolerance_to_ros(bus):
    from control_msgs.msg import JointComponentTolerance as RosMsg

    out = RosMsg()
    out.joint_name = str(bus.joint_name)
    out.component = bus.component
    out.value = bus.value
    return out


class ControlMsgsJointComponentToleranceMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import JointComponentTolerance as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return joint_component_tolerance_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_component_tolerance_to_ros(bus)
