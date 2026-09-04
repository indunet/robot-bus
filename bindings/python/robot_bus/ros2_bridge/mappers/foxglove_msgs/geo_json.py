"""Generated mapper for `foxglove_msgs/msg/GeoJSON`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import GeoJSON as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def geo_json_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.geojson = str(msg.geojson)
    return bus


def geo_json_to_ros(bus):
    from foxglove_msgs.msg import GeoJSON as RosMsg

    out = RosMsg()
    out.geojson = str(bus.geojson)
    return out


class FoxgloveMsgsGeoJsonMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import GeoJSON as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return geo_json_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return geo_json_to_ros(bus)
