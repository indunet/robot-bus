"""OccupancyGrid-style int8[] ↔ proto bytes without per-cell Python ints."""

from __future__ import annotations

import array
import importlib.util
from pathlib import Path

_CONVERT_PATH = (
    Path(__file__).resolve().parents[1]
    / "robot_bus"
    / "ros2_bridge"
    / "mappers"
    / "_convert.py"
)
_spec = importlib.util.spec_from_file_location("_ros2_bridge_convert", _CONVERT_PATH)
assert _spec is not None and _spec.loader is not None
_convert = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_convert)
i8_seq_to_bytes = _convert.i8_seq_to_bytes
bytes_to_i8_seq = _convert.bytes_to_i8_seq


def test_i8_seq_roundtrip_array():
    src = array.array("b", [0, 1, -1, 100, -128, 127])
    raw = i8_seq_to_bytes(src)
    assert raw == bytes([0, 1, 255, 100, 128, 127])
    back = bytes_to_i8_seq(raw)
    assert isinstance(back, array.array)
    assert back.typecode == "b"
    assert list(back) == list(src)


def test_i8_seq_from_list_and_empty():
    raw = i8_seq_to_bytes([0, -1, 127])
    assert raw == bytes([0, 255, 127])
    assert bytes(bytes_to_i8_seq(b"")) == b""
    assert i8_seq_to_bytes(b"") == b""


def test_bytes_to_i8_seq_is_not_list():
    out = bytes_to_i8_seq(bytes(range(256)))
    assert not isinstance(out, list)
    assert len(out) == 256
    assert out[0] == 0
    assert out[128] == -128
    assert out[255] == -1
