"""Per-bridge drop counters and per-route health for ROS↔bus topic wire."""

from __future__ import annotations

import logging
import time
from typing import Any, Callable, Optional

LOG = logging.getLogger("robot_bus.ros2_bridge")

try:
    from google.protobuf.message import DecodeError as ProtobufDecodeError
except ImportError:  # pragma: no cover - protobuf always present in this package
    ProtobufDecodeError = ()  # type: ignore[misc, assignment]

WARN_INTERVAL_S = 1.0
IDLE_GRACE_S = 15.0
SNAPSHOT_INTERVAL_S = 1.0


def unix_ms() -> int:
    return int(time.time() * 1000)


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


class RouteHealth:
    __slots__ = (
        "rx",
        "tx",
        "convert_fail",
        "decode_fail",
        "publish_fail",
        "last_rx_ms",
        "last_warn_s",
        "idle_latched",
    )

    def __init__(self) -> None:
        self.rx = 0
        self.tx = 0
        self.convert_fail = 0
        self.decode_fail = 0
        self.publish_fail = 0
        self.last_rx_ms = 0
        self.last_warn_s = 0.0
        self.idle_latched = False

    def record_rx(self) -> None:
        self.rx += 1
        self.last_rx_ms = unix_ms()

    def record_tx(self) -> None:
        self.tx += 1

    def record_convert_fail(self) -> None:
        self.convert_fail += 1

    def record_decode_fail(self) -> None:
        self.decode_fail += 1

    def record_publish_fail(self) -> None:
        self.publish_fail += 1

    def should_log_warn(self) -> bool:
        now = time.monotonic()
        if self.last_warn_s and (now - self.last_warn_s) < WARN_INTERVAL_S:
            return False
        self.last_warn_s = now
        return True

    def is_idle(self, enabled: bool, grace_elapsed: bool) -> bool:
        return bool(enabled and grace_elapsed and self.last_rx_ms == 0)

    def take_idle_event(self, enabled: bool, grace_elapsed: bool) -> bool:
        if self.last_rx_ms != 0:
            self.idle_latched = False
            return False
        if not self.is_idle(enabled, grace_elapsed):
            return False
        if self.idle_latched:
            return False
        self.idle_latched = True
        return True


def forward_ros_to_bus(
    stats: DropStats,
    topic: str,
    convert: Callable[[Any], bytes],
    publish: Callable[[bytes], Any],
    msg: Any,
    health: Optional[RouteHealth] = None,
) -> None:
    if health is not None:
        health.record_rx()
    try:
        payload = convert(msg)
    except Exception as err:  # noqa: BLE001
        stats.convert_fail += 1
        if health is not None:
            health.record_convert_fail()
        if health is None or health.should_log_warn():
            LOG.warning("ros→bus %s convert: %s", topic, err)
        return
    try:
        publish(payload)
    except Exception as err:  # noqa: BLE001
        stats.publish_fail += 1
        if health is not None:
            health.record_publish_fail()
        if health is None or health.should_log_warn():
            LOG.warning("ros→bus %s publish: %s", topic, err)
        return
    if health is not None:
        health.record_tx()


def forward_bus_to_ros(
    stats: DropStats,
    topic: str,
    convert: Callable[[bytes], Any],
    publish: Callable[[Any], Any],
    payload: bytes,
    health: Optional[RouteHealth] = None,
) -> None:
    if health is not None:
        health.record_rx()
    try:
        ros_msg = convert(payload)
    except Exception as err:  # noqa: BLE001
        if ProtobufDecodeError and isinstance(err, ProtobufDecodeError):
            stats.decode_fail += 1
            if health is not None:
                health.record_decode_fail()
            if health is None or health.should_log_warn():
                LOG.warning("bus→ros %s decode: %s", topic, err)
        else:
            stats.convert_fail += 1
            if health is not None:
                health.record_convert_fail()
            if health is None or health.should_log_warn():
                LOG.warning("bus→ros %s convert: %s", topic, err)
        return
    try:
        publish(ros_msg)
    except Exception as err:  # noqa: BLE001
        stats.publish_fail += 1
        if health is not None:
            health.record_publish_fail()
        if health is None or health.should_log_warn():
            LOG.warning("bus→ros %s publish: %s", topic, err)
        return
    if health is not None:
        health.record_tx()
