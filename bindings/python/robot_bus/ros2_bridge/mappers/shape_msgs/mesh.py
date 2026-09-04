"""Generated mapper for `shape_msgs/msg/Mesh`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.shape_msgs.mesh_triangle import mesh_triangle_to_bus, mesh_triangle_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.shape_msgs.msg.v1 import Mesh as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def mesh_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.triangles.extend([mesh_triangle_to_bus(x) for x in msg.triangles])
    bus.vertices.extend([point_to_bus(x) for x in msg.vertices])
    return bus


def mesh_to_ros(bus):
    from shape_msgs.msg import Mesh as RosMsg

    out = RosMsg()
    out.triangles = [mesh_triangle_to_ros(x) for x in bus.triangles]
    out.vertices = [point_to_ros(x) for x in bus.vertices]
    return out


class ShapeMsgsMeshMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from shape_msgs.msg import Mesh as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return mesh_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return mesh_to_ros(bus)
