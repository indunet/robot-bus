"""Generated mapper for `nav2_msgs/msg/CollisionMonitorState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import CollisionMonitorState as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def collision_monitor_state_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.action_type = msg.action_type
    bus.polygon_name = str(msg.polygon_name)
    return bus


def collision_monitor_state_to_ros(bus):
    from nav2_msgs.msg import CollisionMonitorState as RosMsg

    out = RosMsg()
    out.action_type = bus.action_type
    out.polygon_name = str(bus.polygon_name)
    return out


class Nav2MsgsCollisionMonitorStateMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import CollisionMonitorState as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return collision_monitor_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return collision_monitor_state_to_ros(bus)
