"""Generated mapper for `geometry_msgs/msg/QuaternionStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.quaternion import quaternion_to_bus, quaternion_to_ros

def quaternion_stamped_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import QuaternionStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.quaternion.CopyFrom(quaternion_to_bus(msg.quaternion))
    return bus


def quaternion_stamped_to_ros(bus):
    from geometry_msgs.msg import QuaternionStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.quaternion = quaternion_to_ros(bus.quaternion)
    return out


class GeometryMsgsQuaternionStampedMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/QuaternionStamped"

    def ros_msg_type(self):
        from geometry_msgs.msg import QuaternionStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return quaternion_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import QuaternionStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return quaternion_stamped_to_ros(bus)
