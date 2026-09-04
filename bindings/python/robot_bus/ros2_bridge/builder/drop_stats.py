"""Per-bridge drop counters for ROS↔bus topic wire failures."""

from __future__ import annotations

import logging
from typing import Any, Callable

LOG = logging.getLogger("robot_bus.ros2_bridge")

try:
    from google.protobuf.message import DecodeError as ProtobufDecodeError
except ImportError:  # pragma: no cover - protobuf always present in this package
    ProtobufDecodeError = ()  # type: ignore[misc, assignment]


class DropStats:
    __slots__ = ("convert_fail", "decode_fail", "publish_fail")

    def __init__(self) -> None:
        self.convert_fail = 0
        self.decode_fail = 0
        self.publish_fail = 0

    def snapshot(self) -> dict[str, int]:
        return {
            "convert_fail": self.convert_fail,
            "decode_fail": self.decode_fail,
            "publish_fail": self.publish_fail,
        }


def forward_ros_to_bus(
    stats: DropStats,
    topic: str,
    convert: Callable[[Any], bytes],
    publish: Callable[[bytes], Any],
    msg: Any,
) -> None:
    try:
        payload = convert(msg)
    except Exception as err:  # noqa: BLE001
        LOG.warning("ros→bus %s convert: %s", topic, err)
        stats.convert_fail += 1
        return
    try:
        publish(payload)
    except Exception as err:  # noqa: BLE001
        LOG.warning("ros→bus %s publish: %s", topic, err)
        stats.publish_fail += 1


def forward_bus_to_ros(
    stats: DropStats,
    topic: str,
    convert: Callable[[bytes], Any],
    publish: Callable[[Any], Any],
    payload: bytes,
) -> None:
    try:
        ros_msg = convert(payload)
    except Exception as err:  # noqa: BLE001
        if ProtobufDecodeError and isinstance(err, ProtobufDecodeError):
            LOG.warning("bus→ros %s decode: %s", topic, err)
            stats.decode_fail += 1
        else:
            LOG.warning("bus→ros %s convert: %s", topic, err)
            stats.convert_fail += 1
        return
    try:
        publish(ros_msg)
    except Exception as err:  # noqa: BLE001
        LOG.warning("bus→ros %s publish: %s", topic, err)
        stats.publish_fail += 1
