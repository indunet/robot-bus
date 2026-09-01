"""Generated mapper for `foxglove_msgs/msg/GeoJSON`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def geo_json_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import GeoJSON as BusMsg

    bus = BusMsg()
    bus.geojson = str(msg.geojson)
    return bus


def geo_json_to_ros(bus):
    from foxglove_msgs.msg import GeoJSON as RosMsg

    out = RosMsg()
    out.geojson = str(bus.geojson)
    return out


class FoxgloveMsgsGeoJsonMapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import GeoJSON as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return geo_json_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import GeoJSON as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return geo_json_to_ros(bus)
