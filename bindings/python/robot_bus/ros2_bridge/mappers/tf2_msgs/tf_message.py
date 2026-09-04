"""Generated mapper for `tf2_msgs/msg/TFMessage`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.transform_stamped import transform_stamped_to_bus, transform_stamped_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.tf2_msgs.msg.v1 import TFMessage as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def tf_message_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.transforms.extend([transform_stamped_to_bus(x) for x in msg.transforms])
    return bus


def tf_message_to_ros(bus):
    from tf2_msgs.msg import TFMessage as RosMsg

    out = RosMsg()
    out.transforms = [transform_stamped_to_ros(x) for x in bus.transforms]
    return out


class Tf2MsgsTfMessageMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from tf2_msgs.msg import TFMessage as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return tf_message_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return tf_message_to_ros(bus)
