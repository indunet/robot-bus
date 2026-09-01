"""Generated mapper for `nav2_msgs/msg/CostmapFilterInfo`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def costmap_filter_info_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import CostmapFilterInfo as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.type = msg.type
    bus.filter_mask_topic = str(msg.filter_mask_topic)
    bus.base = msg.base
    bus.multiplier = msg.multiplier
    return bus


def costmap_filter_info_to_ros(bus):
    from nav2_msgs.msg import CostmapFilterInfo as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.type = bus.type
    out.filter_mask_topic = str(bus.filter_mask_topic)
    out.base = bus.base
    out.multiplier = bus.multiplier
    return out


class Nav2MsgsCostmapFilterInfoMapper:
    def ros_msg_type(self):
        from nav2_msgs.msg import CostmapFilterInfo as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return costmap_filter_info_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import CostmapFilterInfo as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return costmap_filter_info_to_ros(bus)
