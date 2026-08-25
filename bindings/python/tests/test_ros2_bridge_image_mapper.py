"""Image mapper: bus→ROS `data` must not be a Python list (Humble Image.data)."""

from __future__ import annotations

try:
    import pytest
except ImportError:  # pragma: no cover

    class pytest:  # type: ignore[no-redef]
        @staticmethod
        def importorskip(name):
            try:
                __import__(name)
            except ImportError as err:
                print(f"skip {name}: {err}")
                raise SystemExit(0) from err


def test_bus_to_ros_data_is_not_list():
    pytest.importorskip("sensor_msgs")
    import array

    from robot_bus.ros2_bridge.mappers.image import SensorMsgsImageMapper
    from robot_bus.sensor_msgs.msg.v1 import Image as BusImage

    bus = BusImage()
    bus.height = 1
    bus.width = 2
    bus.encoding = "rgb8"
    bus.step = 6
    bus.data = b"abcdef"
    ros = SensorMsgsImageMapper().bus_to_ros(bus.SerializeToString())
    assert not isinstance(ros.data, list)
    assert isinstance(ros.data, array.array)
    assert ros.data.typecode == "B"
    assert bytes(ros.data) == b"abcdef"


if __name__ == "__main__":
    try:
        test_bus_to_ros_data_is_not_list()
    except SystemExit:
        pass
    else:
        print("test_ros2_bridge_image_mapper ok")
