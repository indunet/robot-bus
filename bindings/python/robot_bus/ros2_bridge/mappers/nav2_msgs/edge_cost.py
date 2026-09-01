"""Generated mapper for `nav2_msgs/msg/EdgeCost`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def edge_cost_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import EdgeCost as BusMsg

    bus = BusMsg()
    bus.edgeid = msg.edgeid
    bus.cost = msg.cost
    return bus


def edge_cost_to_ros(bus):
    from nav2_msgs.msg import EdgeCost as RosMsg

    out = RosMsg()
    out.edgeid = bus.edgeid
    out.cost = bus.cost
    return out


class Nav2MsgsEdgeCostMapper:
    def ros_msg_type(self):
        from nav2_msgs.msg import EdgeCost as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return edge_cost_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import EdgeCost as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return edge_cost_to_ros(bus)
