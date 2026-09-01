"""Generated mapper for `sensor_msgs/msg/CameraInfo`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.sensor_msgs.region_of_interest import region_of_interest_to_bus, region_of_interest_to_ros

def camera_info_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import CameraInfo as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.height = msg.height
    bus.width = msg.width
    bus.distortion_model = str(msg.distortion_model)
    bus.d.extend(list(msg.d))
    bus.k.extend(list(msg.k))
    bus.r.extend(list(msg.r))
    bus.p.extend(list(msg.p))
    bus.binning_x = msg.binning_x
    bus.binning_y = msg.binning_y
    bus.roi.CopyFrom(region_of_interest_to_bus(msg.roi))
    return bus


def camera_info_to_ros(bus):
    from sensor_msgs.msg import CameraInfo as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.height = bus.height
    out.width = bus.width
    out.distortion_model = str(bus.distortion_model)
    out.d = list(bus.d)
    out.k = list(bus.k)
    out.r = list(bus.r)
    out.p = list(bus.p)
    out.binning_x = bus.binning_x
    out.binning_y = bus.binning_y
    out.roi = region_of_interest_to_ros(bus.roi)
    return out


class SensorMsgsCameraInfoMapper:
    def type_name(self) -> str:
        return "sensor_msgs/msg/CameraInfo"

    def ros_msg_type(self):
        from sensor_msgs.msg import CameraInfo as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return camera_info_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import CameraInfo as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return camera_info_to_ros(bus)
