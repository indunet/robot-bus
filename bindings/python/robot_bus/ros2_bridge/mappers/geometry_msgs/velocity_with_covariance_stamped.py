"""Generated mapper for `geometry_msgs/msg/VelocityWithCovarianceStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist_with_covariance import twist_with_covariance_to_bus, twist_with_covariance_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.geometry_msgs.msg.v1 import VelocityWithCovarianceStamped as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def velocity_with_covariance_stamped_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.body_frame_id = str(msg.body_frame_id)
    bus.reference_frame_id = str(msg.reference_frame_id)
    bus.velocity.CopyFrom(twist_with_covariance_to_bus(msg.velocity))
    return bus


def velocity_with_covariance_stamped_to_ros(bus):
    from geometry_msgs.msg import VelocityWithCovarianceStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.body_frame_id = str(bus.body_frame_id)
    out.reference_frame_id = str(bus.reference_frame_id)
    out.velocity = twist_with_covariance_to_ros(bus.velocity)
    return out


class GeometryMsgsVelocityWithCovarianceStampedMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from geometry_msgs.msg import VelocityWithCovarianceStamped as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return velocity_with_covariance_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return velocity_with_covariance_stamped_to_ros(bus)
