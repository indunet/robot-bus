"""Generated mapper for `foxglove_msgs/msg/Pose`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector3 import vector3_to_bus, vector3_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.quaternion import quaternion_to_bus, quaternion_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import Pose as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def pose_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.position.CopyFrom(vector3_to_bus(msg.position))
    bus.orientation.CopyFrom(quaternion_to_bus(msg.orientation))
    return bus


def pose_to_ros(bus):
    from foxglove_msgs.msg import Pose as RosMsg

    out = RosMsg()
    out.position = vector3_to_ros(bus.position)
    out.orientation = quaternion_to_ros(bus.orientation)
    return out


class FoxgloveMsgsPoseMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import Pose as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return pose_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return pose_to_ros(bus)
