"""Generated mapper for `foxglove_msgs/msg/SceneUpdate`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.scene_entity_deletion import scene_entity_deletion_to_bus, scene_entity_deletion_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.scene_entity import scene_entity_to_bus, scene_entity_to_ros

def scene_update_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import SceneUpdate as BusMsg

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
    def type_name(self) -> str:
        return "foxglove_msgs/msg/SceneUpdate"

    def ros_msg_type(self):
        from foxglove_msgs.msg import SceneUpdate as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return scene_update_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import SceneUpdate as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return scene_update_to_ros(bus)
