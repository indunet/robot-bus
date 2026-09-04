"""Generated mapper for `foxglove_msgs/msg/Odometry`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector3 import vector3_to_bus, vector3_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.key_value_pair import key_value_pair_to_bus, key_value_pair_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import Odometry as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def odometry_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.body_frame_id = str(msg.body_frame_id)
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.linear_velocity.CopyFrom(vector3_to_bus(msg.linear_velocity))
    bus.angular_velocity.CopyFrom(vector3_to_bus(msg.angular_velocity))
    bus.pose_covariance.extend(list(msg.pose_covariance))
    bus.velocity_covariance.extend(list(msg.velocity_covariance))
    bus.metadata.extend([key_value_pair_to_bus(x) for x in msg.metadata])
    return bus


def odometry_to_ros(bus):
    from foxglove_msgs.msg import Odometry as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.body_frame_id = str(bus.body_frame_id)
    out.pose = pose_to_ros(bus.pose)
    out.linear_velocity = vector3_to_ros(bus.linear_velocity)
    out.angular_velocity = vector3_to_ros(bus.angular_velocity)
    out.pose_covariance = list(bus.pose_covariance)
    out.velocity_covariance = list(bus.velocity_covariance)
    out.metadata = [key_value_pair_to_ros(x) for x in bus.metadata]
    return out


class FoxgloveMsgsOdometryMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import Odometry as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return odometry_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return odometry_to_ros(bus)
