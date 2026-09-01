"""Generated mapper for `foxglove_msgs/msg/SceneEntityDeletion`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def scene_entity_deletion_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import SceneEntityDeletion as BusMsg

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
    def ros_msg_type(self):
        from foxglove_msgs.msg import SceneEntityDeletion as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return scene_entity_deletion_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import SceneEntityDeletion as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return scene_entity_deletion_to_ros(bus)
