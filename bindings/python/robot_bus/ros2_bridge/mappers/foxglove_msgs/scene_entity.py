"""Generated mapper for `foxglove_msgs/msg/SceneEntity`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.key_value_pair import key_value_pair_to_bus, key_value_pair_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.arrow_primitive import arrow_primitive_to_bus, arrow_primitive_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.cube_primitive import cube_primitive_to_bus, cube_primitive_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.sphere_primitive import sphere_primitive_to_bus, sphere_primitive_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.cylinder_primitive import cylinder_primitive_to_bus, cylinder_primitive_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.line_primitive import line_primitive_to_bus, line_primitive_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.triangle_list_primitive import triangle_list_primitive_to_bus, triangle_list_primitive_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.text_primitive import text_primitive_to_bus, text_primitive_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.model_primitive import model_primitive_to_bus, model_primitive_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import SceneEntity as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def scene_entity_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.id = str(msg.id)
    bus.lifetime = _convert.duration_to_proto(msg.lifetime)
    bus.frame_locked = msg.frame_locked
    bus.metadata.extend([key_value_pair_to_bus(x) for x in msg.metadata])
    bus.arrows.extend([arrow_primitive_to_bus(x) for x in msg.arrows])
    bus.cubes.extend([cube_primitive_to_bus(x) for x in msg.cubes])
    bus.spheres.extend([sphere_primitive_to_bus(x) for x in msg.spheres])
    bus.cylinders.extend([cylinder_primitive_to_bus(x) for x in msg.cylinders])
    bus.lines.extend([line_primitive_to_bus(x) for x in msg.lines])
    bus.triangles.extend([triangle_list_primitive_to_bus(x) for x in msg.triangles])
    bus.texts.extend([text_primitive_to_bus(x) for x in msg.texts])
    bus.models.extend([model_primitive_to_bus(x) for x in msg.models])
    return bus


def scene_entity_to_ros(bus):
    from foxglove_msgs.msg import SceneEntity as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.id = str(bus.id)
    out.lifetime = _convert.proto_to_duration(bus.lifetime)
    out.frame_locked = bus.frame_locked
    out.metadata = [key_value_pair_to_ros(x) for x in bus.metadata]
    out.arrows = [arrow_primitive_to_ros(x) for x in bus.arrows]
    out.cubes = [cube_primitive_to_ros(x) for x in bus.cubes]
    out.spheres = [sphere_primitive_to_ros(x) for x in bus.spheres]
    out.cylinders = [cylinder_primitive_to_ros(x) for x in bus.cylinders]
    out.lines = [line_primitive_to_ros(x) for x in bus.lines]
    out.triangles = [triangle_list_primitive_to_ros(x) for x in bus.triangles]
    out.texts = [text_primitive_to_ros(x) for x in bus.texts]
    out.models = [model_primitive_to_ros(x) for x in bus.models]
    return out


class FoxgloveMsgsSceneEntityMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import SceneEntity as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return scene_entity_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return scene_entity_to_ros(bus)
