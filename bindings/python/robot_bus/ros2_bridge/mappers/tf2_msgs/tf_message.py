"""Generated mapper for `tf2_msgs/msg/TFMessage`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.transform_stamped import transform_stamped_to_bus, transform_stamped_to_ros

def tf_message_to_bus(msg):
    from robot_bus.tf2_msgs.msg.v1 import TFMessage as BusMsg

    bus = BusMsg()
    bus.transforms.extend([transform_stamped_to_bus(x) for x in msg.transforms])
    return bus


def tf_message_to_ros(bus):
    from tf2_msgs.msg import TFMessage as RosMsg

    out = RosMsg()
    out.transforms = [transform_stamped_to_ros(x) for x in bus.transforms]
    return out


class Tf2MsgsTfMessageMapper:
    def type_name(self) -> str:
        return "tf2_msgs/msg/TFMessage"

    def ros_msg_type(self):
        from tf2_msgs.msg import TFMessage as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return tf_message_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.tf2_msgs.msg.v1 import TFMessage as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return tf_message_to_ros(bus)
