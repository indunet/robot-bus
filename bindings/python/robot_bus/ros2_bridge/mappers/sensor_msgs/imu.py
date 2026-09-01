"""Generated mapper for `sensor_msgs/msg/Imu`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.quaternion import quaternion_to_bus, quaternion_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros

def imu_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import Imu as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.orientation.CopyFrom(quaternion_to_bus(msg.orientation))
    bus.orientation_covariance.extend(list(msg.orientation_covariance))
    bus.angular_velocity.CopyFrom(vector3_to_bus(msg.angular_velocity))
    bus.angular_velocity_covariance.extend(list(msg.angular_velocity_covariance))
    bus.linear_acceleration.CopyFrom(vector3_to_bus(msg.linear_acceleration))
    bus.linear_acceleration_covariance.extend(list(msg.linear_acceleration_covariance))
    return bus


def imu_to_ros(bus):
    from sensor_msgs.msg import Imu as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.orientation = quaternion_to_ros(bus.orientation)
    out.orientation_covariance = list(bus.orientation_covariance)
    out.angular_velocity = vector3_to_ros(bus.angular_velocity)
    out.angular_velocity_covariance = list(bus.angular_velocity_covariance)
    out.linear_acceleration = vector3_to_ros(bus.linear_acceleration)
    out.linear_acceleration_covariance = list(bus.linear_acceleration_covariance)
    return out


class SensorMsgsImuMapper:
    def type_name(self) -> str:
        return "sensor_msgs/msg/Imu"

    def ros_msg_type(self):
        from sensor_msgs.msg import Imu as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return imu_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import Imu as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return imu_to_ros(bus)
