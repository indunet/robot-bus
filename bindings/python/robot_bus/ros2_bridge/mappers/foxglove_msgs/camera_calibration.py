"""Generated mapper for `foxglove_msgs/msg/CameraCalibration`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def camera_calibration_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import CameraCalibration as BusMsg

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
    def ros_msg_type(self):
        from foxglove_msgs.msg import CameraCalibration as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return camera_calibration_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import CameraCalibration as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return camera_calibration_to_ros(bus)
