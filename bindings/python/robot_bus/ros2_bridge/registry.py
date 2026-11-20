"""Look up ROS2 bridge mappers by ROS type name (``"geometry_msgs/msg/Twist"``)."""

from __future__ import annotations

import difflib
import re
import sys
from typing import Any, Optional

_KINDS = ("msg", "srv", "action")
_KIND_LABEL = {"msg": "topic", "srv": "service", "action": "action"}
_KIND_STAGE = {"msg": "a topic route", "srv": ".service()", "action": ".action()"}

# Type segment matched case-insensitively: users mistype ROS casing (ColorRGBA).
_builtin: Optional[dict[str, tuple[str, Any]]] = None
_user: dict[str, tuple[str, Any]] = {}


def _split(type_name: str, kind: Optional[str] = None) -> tuple[str, str, str]:
    parts = str(type_name).strip().strip("/").split("/")
    if len(parts) == 2 and kind is not None:
        parts = [parts[0], kind, parts[1]]
    if len(parts) != 3 or parts[1] not in _KINDS or not parts[0] or not parts[2]:
        raise ValueError(
            f"ros2 bridge: bad ROS type name {type_name!r}; "
            "expected 'pkg/msg/Name', 'pkg/srv/Name' or 'pkg/action/Name'"
        )
    return parts[0], parts[1], parts[2]


def _key(pkg: str, kind: str, name: str) -> str:
    return f"{pkg}/{kind}/{name.lower()}"


def _pascal(pkg: str) -> str:
    return "".join(p[:1].upper() + p[1:] for p in pkg.split("_"))


def _load_builtin() -> dict[str, tuple[str, Any]]:
    global _builtin
    if _builtin is not None:
        return _builtin
    from robot_bus.ros2_bridge import mappers

    table: dict[str, tuple[str, Any]] = {}

    def put(type_name: str, cls: Any) -> None:
        pkg, kind, name = _split(type_name)
        table[_key(pkg, kind, name)] = (type_name, cls)

    put("std_msgs/msg/String", mappers.StdMsgsStringMapper)
    put("sensor_msgs/msg/Image", mappers.SensorMsgsImageMapper)
    for cls in (
        mappers.TriggerServiceMapper,
        mappers.SetBoolServiceMapper,
        mappers.FibonacciActionMapper,
    ):
        put(cls().type_name(), cls)
    for export in mappers.__all__:
        cls = getattr(mappers, export)
        doc = re.match(r"Generated mapper for `([^`]+)`", sys.modules[cls.__module__].__doc__ or "")
        if doc:
            put(doc.group(1), cls)
            continue
        pkg = cls.__module__.split(".")[-2]
        prefix = _pascal(pkg)
        if pkg == "mappers" or not export.startswith(prefix) or not export.endswith("Mapper"):
            continue
        put(f"{pkg}/msg/{export[len(prefix):-len('Mapper')]}", cls)
    _builtin = table
    return table


def register_mapper(type_name: str, mapper: Any = None, *, override: bool = False) -> Any:
    """Make ``.mapper("pkg/msg/Name")`` resolve to a custom mapper.

    ``mapper`` may be a class (instantiated per route) or an instance. Without
    ``mapper`` this works as a class decorator::

        @register_mapper("my_pkg/msg/Sum")
        class SumMapper: ...
    """
    pkg, kind, name = _split(type_name)
    key = _key(pkg, kind, name)

    def add(m: Any) -> Any:
        existing = _user.get(key) or _load_builtin().get(key)
        if existing is not None and existing[1] is not m and not override:
            raise ValueError(
                f"ros2 bridge: mapper for {existing[0]!r} is already registered; "
                "pass override=True to replace it"
            )
        _user[key] = (type_name, m)
        return m

    if mapper is None:
        return add
    add(mapper)
    return mapper


def resolve_mapper(mapper: Any, kind: str) -> Any:
    """Return ``mapper`` as is, or look it up when it is a ROS type name."""
    if not isinstance(mapper, str):
        return mapper
    pkg, got, name = _split(mapper, kind)
    if got != kind:
        raise ValueError(
            f"ros2 bridge: {mapper!r} is a {_KIND_LABEL[got]} type; "
            f"use it on {_KIND_STAGE[got]}, not on a {_KIND_LABEL[kind]} route"
        )
    key = _key(pkg, got, name)
    entry = _user.get(key) or _load_builtin().get(key)
    if entry is None:
        known = sorted(n for n, _ in {**_load_builtin(), **_user}.values() if f"/{kind}/" in n)
        close = difflib.get_close_matches(f"{pkg}/{got}/{name}", known, n=3, cutoff=0.6)
        hint = f" Did you mean: {', '.join(close)}?" if close else ""
        raise ValueError(
            f"ros2 bridge: no mapper registered for {mapper!r}.{hint} "
            "For a custom type, call register_mapper(type_name, MyMapper) "
            "or pass a mapper object to .mapper()."
        )
    found = entry[1]
    return found() if isinstance(found, type) else found
