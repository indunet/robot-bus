"""Generated mapper for `foxglove_msgs/msg/CameraCalibration`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import CameraCalibration as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def camera_calibration_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.width = msg.width
    bus.height = msg.height
    bus.distortion_model = str(msg.distortion_model)
    bus.D.extend(list(msg.D))
    bus.K.extend(list(msg.K))
    bus.R.extend(list(msg.R))
    bus.P.extend(list(msg.P))
    return bus


def camera_calibration_to_ros(bus):
    from foxglove_msgs.msg import CameraCalibration as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.width = bus.width
    out.height = bus.height
    out.distortion_model = str(bus.distortion_model)
    out.D = list(bus.D)
    out.K = list(bus.K)
    out.R = list(bus.R)
    out.P = list(bus.P)
    return out


class FoxgloveMsgsCameraCalibrationMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import CameraCalibration as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return camera_calibration_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return camera_calibration_to_ros(bus)
