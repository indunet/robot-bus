"""Generated mapper for `unique_identifier_msgs/msg/UUID`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.unique_identifier_msgs.msg.v1 import UUID as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def uuid_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.uuid = bytes(msg.uuid)
    return bus


def uuid_to_ros(bus):
    from unique_identifier_msgs.msg import UUID as RosMsg

    out = RosMsg()
    out.uuid = bytes(bus.uuid)
    return out


class UniqueIdentifierMsgsUuidMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from unique_identifier_msgs.msg import UUID as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return uuid_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return uuid_to_ros(bus)
