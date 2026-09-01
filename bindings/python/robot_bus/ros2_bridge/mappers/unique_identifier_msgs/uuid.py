"""Generated mapper for `unique_identifier_msgs/msg/UUID`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def uuid_to_bus(msg):
    from robot_bus.unique_identifier_msgs.msg.v1 import UUID as BusMsg

    bus = BusMsg()
    bus.uuid = bytes(msg.uuid)
    return bus


def uuid_to_ros(bus):
    from unique_identifier_msgs.msg import UUID as RosMsg

    out = RosMsg()
    out.uuid = bytes(bus.uuid)
    return out


class UniqueIdentifierMsgsUuidMapper:
    def ros_msg_type(self):
        from unique_identifier_msgs.msg import UUID as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return uuid_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.unique_identifier_msgs.msg.v1 import UUID as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return uuid_to_ros(bus)
