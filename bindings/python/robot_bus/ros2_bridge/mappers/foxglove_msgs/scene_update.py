"""Generated mapper for `foxglove_msgs/msg/SceneUpdate`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.scene_entity_deletion import scene_entity_deletion_to_bus, scene_entity_deletion_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.scene_entity import scene_entity_to_bus, scene_entity_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import SceneUpdate as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def scene_update_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.deletions.extend([scene_entity_deletion_to_bus(x) for x in msg.deletions])
    bus.entities.extend([scene_entity_to_bus(x) for x in msg.entities])
    return bus


def scene_update_to_ros(bus):
    from foxglove_msgs.msg import SceneUpdate as RosMsg

    out = RosMsg()
    out.deletions = [scene_entity_deletion_to_ros(x) for x in bus.deletions]
    out.entities = [scene_entity_to_ros(x) for x in bus.entities]
    return out


class FoxgloveMsgsSceneUpdateMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import SceneUpdate as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return scene_update_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return scene_update_to_ros(bus)
