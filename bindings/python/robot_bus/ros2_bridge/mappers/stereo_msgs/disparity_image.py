"""Generated mapper for `stereo_msgs/msg/DisparityImage`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.sensor_msgs.image import image_to_bus, image_to_ros
from robot_bus.ros2_bridge.mappers.sensor_msgs.region_of_interest import region_of_interest_to_bus, region_of_interest_to_ros

def disparity_image_to_bus(msg):
    from robot_bus.stereo_msgs.msg.v1 import DisparityImage as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.image.CopyFrom(image_to_bus(msg.image))
    bus.f = msg.f
    bus.t = msg.t
    bus.valid_window.CopyFrom(region_of_interest_to_bus(msg.valid_window))
    bus.min_disparity = msg.min_disparity
    bus.max_disparity = msg.max_disparity
    bus.delta_d = msg.delta_d
    return bus


def disparity_image_to_ros(bus):
    from stereo_msgs.msg import DisparityImage as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.image = image_to_ros(bus.image)
    out.f = bus.f
    out.t = bus.t
    out.valid_window = region_of_interest_to_ros(bus.valid_window)
    out.min_disparity = bus.min_disparity
    out.max_disparity = bus.max_disparity
    out.delta_d = bus.delta_d
    return out


class StereoMsgsDisparityImageMapper:
    def ros_msg_type(self):
        from stereo_msgs.msg import DisparityImage as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return disparity_image_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.stereo_msgs.msg.v1 import DisparityImage as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return disparity_image_to_ros(bus)
