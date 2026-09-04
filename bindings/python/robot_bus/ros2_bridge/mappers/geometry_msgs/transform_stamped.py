"""Generated mapper for `geometry_msgs/msg/TransformStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.transform import transform_to_bus, transform_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.geometry_msgs.msg.v1 import TransformStamped as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def transform_stamped_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.child_frame_id = str(msg.child_frame_id)
    bus.transform.CopyFrom(transform_to_bus(msg.transform))
    return bus


def transform_stamped_to_ros(bus):
    from geometry_msgs.msg import TransformStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.child_frame_id = str(bus.child_frame_id)
    out.transform = transform_to_ros(bus.transform)
    return out


class GeometryMsgsTransformStampedMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from geometry_msgs.msg import TransformStamped as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return transform_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return transform_stamped_to_ros(bus)
