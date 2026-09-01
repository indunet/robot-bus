"""Generated mapper for `foxglove_msgs/msg/LocationFix`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector3 import vector3_to_bus, vector3_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import color_to_bus, color_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.key_value_pair import key_value_pair_to_bus, key_value_pair_to_ros

def location_fix_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import LocationFix as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.latitude = msg.latitude
    bus.longitude = msg.longitude
    bus.altitude = msg.altitude
    bus.position_covariance.extend(list(msg.position_covariance))
    bus.position_covariance_type = int(msg.position_covariance_type)
    bus.heading = msg.heading
    bus.velocity.CopyFrom(vector3_to_bus(msg.velocity))
    bus.color.CopyFrom(color_to_bus(msg.color))
    bus.metadata.extend([key_value_pair_to_bus(x) for x in msg.metadata])
    return bus


def location_fix_to_ros(bus):
    from foxglove_msgs.msg import LocationFix as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.latitude = bus.latitude
    out.longitude = bus.longitude
    out.altitude = bus.altitude
    out.position_covariance = list(bus.position_covariance)
    out.position_covariance_type = int(bus.position_covariance_type)
    out.heading = bus.heading
    out.velocity = vector3_to_ros(bus.velocity)
    out.color = color_to_ros(bus.color)
    out.metadata = [key_value_pair_to_ros(x) for x in bus.metadata]
    return out


class FoxgloveMsgsLocationFixMapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/LocationFix"

    def ros_msg_type(self):
        from foxglove_msgs.msg import LocationFix as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return location_fix_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import LocationFix as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return location_fix_to_ros(bus)
