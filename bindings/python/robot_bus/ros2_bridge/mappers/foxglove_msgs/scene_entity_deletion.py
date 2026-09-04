"""Generated mapper for `foxglove_msgs/msg/SceneEntityDeletion`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import SceneEntityDeletion as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def scene_entity_deletion_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.type = int(msg.type)
    bus.id = str(msg.id)
    return bus


def scene_entity_deletion_to_ros(bus):
    from foxglove_msgs.msg import SceneEntityDeletion as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.type = int(bus.type)
    out.id = str(bus.id)
    return out


class FoxgloveMsgsSceneEntityDeletionMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import SceneEntityDeletion as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return scene_entity_deletion_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return scene_entity_deletion_to_ros(bus)
