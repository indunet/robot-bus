"""Generated mapper for `shape_msgs/msg/MeshTriangle`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.shape_msgs.msg.v1 import MeshTriangle as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def mesh_triangle_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.vertex_indices.extend(list(msg.vertex_indices))
    return bus


def mesh_triangle_to_ros(bus):
    from shape_msgs.msg import MeshTriangle as RosMsg

    out = RosMsg()
    out.vertex_indices = list(bus.vertex_indices)
    return out


class ShapeMsgsMeshTriangleMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from shape_msgs.msg import MeshTriangle as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return mesh_triangle_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return mesh_triangle_to_ros(bus)
