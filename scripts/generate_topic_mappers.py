#!/usr/bin/env python3
"""Generate typed ROS↔protobuf topic mappers and ros-env-shim message stubs.

Reads proto/*/msg/v1/*.proto plus existing mapper file headers, then rewrites:

- Rust: src/ros2_bridge/mappers/<pkg>/<msg>.rs (except gold samples)
- Python: bindings/python/robot_bus/ros2_bridge/mappers/<pkg>/<msg>.py
- C++: bindings/cpp/include/robot_bus/ros2_bridge/mappers/<pkg>/<msg>.hpp
- Shim: third_party/ros-env-shim/src/generated_msgs.rs

Service/action builtins (Trigger / SetBool / Fibonacci) stay hand-written.
"""

from __future__ import annotations

import re
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROTO = ROOT / "proto"
MAPPERS = ROOT / "src" / "ros2_bridge" / "mappers"
SHIM_OUT = ROOT / "third_party" / "ros-env-shim" / "src" / "generated_msgs.rs"
GENERATED = ROOT / "src" / "generated"

GOLD = {
    MAPPERS / "std_msgs" / "string.rs",
    MAPPERS / "sensor_msgs" / "image.rs",
    MAPPERS / "nav_msgs" / "occupancy_grid.rs",
}

INT8_BYTES = {
    ("nav_msgs/msg/OccupancyGrid", "data"),
    ("nav_msgs/msg/OccupancyGridUpdate", "data"),
    ("nav2_msgs/msg/Costmap", "data"),
    ("std_msgs/msg/Int8MultiArray", "data"),
}

OCTET_BOOL = {
    ("sensor_msgs/msg/Image", "is_bigendian"),
}

# proto scalar → (ros rust field type in shim, to_bus expr, to_ros expr)
WRAPPER_CAST = {
    ("std_msgs/msg/Int8", "data"): ("i8", "{ros} as i32", "{bus} as i8"),
    ("std_msgs/msg/Int16", "data"): ("i16", "{ros} as i32", "{bus} as i16"),
    ("std_msgs/msg/UInt8", "data"): ("u8", "u32::from({ros})", "{bus} as u8"),
    ("std_msgs/msg/UInt16", "data"): ("u16", "u32::from({ros})", "{bus} as u16"),
    ("std_msgs/msg/Byte", "data"): ("u8", "u32::from({ros})", "{bus} as u8"),
    ("std_msgs/msg/Char", "data"): ("u8", "u32::from({ros})", "{bus} as u8"),
    ("sensor_msgs/msg/PointField", "datatype"): ("u8", "u32::from({ros})", "{bus} as u8"),
    ("sensor_msgs/msg/NavSatStatus", "status"): ("i8", "i32::from({ros})", "{bus} as i8"),
    ("sensor_msgs/msg/NavSatStatus", "service"): ("u16", "u32::from({ros})", "{bus} as u16"),
    ("action_msgs/msg/GoalStatus", "status"): ("i8", "i32::from({ros})", "{bus} as i8"),
}

SKIP_SHIM_PACKAGES = {
    "unique_identifier_msgs",
    "builtin_interfaces",
    "action_msgs",
    "example_interfaces",
    "rosgraph_msgs",
    "rcl_interfaces",
}

SCALAR_ROS = {
    "bool": "bool",
    "int32": "i32",
    "int64": "i64",
    "uint32": "u32",
    "uint64": "u64",
    "sint32": "i32",
    "sint64": "i64",
    "fixed32": "u32",
    "fixed64": "u64",
    "sfixed32": "i32",
    "sfixed64": "i64",
    "float": "f32",
    "double": "f64",
}

RUST_KEYWORDS = {
    "as", "async", "await", "break", "const", "continue", "crate", "dyn",
    "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
}


@dataclass
class Field:
    name: str
    proto_type: str
    repeated: bool
    optional: bool
    is_bytes: bool
    is_string: bool
    is_timestamp: bool
    is_duration: bool
    is_enum: bool
    nested_ros_type: str | None


@dataclass
class Msg:
    ros_type: str
    package: str
    name: str
    proto_stem: str = ""
    fields: list[Field] = field(default_factory=list)


def snake(name: str) -> str:
    name = name.replace("TF", "Tf").replace("UV", "Uv").replace("UUID", "Uuid")
    name = name.replace("DOF", "Dof").replace("JSON", "Json").replace("RGBA", "Rgba")
    name = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
    name = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    return name.replace("__", "_").lower()


def to_upper_camel(name: str) -> str:
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", s)
    parts = re.split(r"[^a-zA-Z0-9]+", s)
    return "".join(p[:1].upper() + p[1:].lower() if p else "" for p in parts)


def to_snake_field(name: str) -> str:
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", s)
    return s.replace("-", "_").lower()


def bus_field(name: str) -> str:
    s = to_snake_field(name)
    return f"r#{s}" if s in RUST_KEYWORDS else s


def ros_field(name: str) -> str:
    s = to_snake_field(name)
    return f"{s}_" if s in RUST_KEYWORDS else s


def collect_enum_names() -> set[str]:
    names: set[str] = set()
    for path in PROTO.rglob("*/msg/v1/*.proto"):
        text = path.read_text()
        for m in re.finditer(r"\benum\s+(\w+)\s*\{", text):
            names.add(m.group(1))
    return names


def load_prost_names() -> dict[str, str]:
    """Proto message name → prost Rust struct name."""
    out: dict[str, str] = {}
    if not GENERATED.is_dir():
        return out
    for path in GENERATED.rglob("*.rs"):
        text = path.read_text()
        for m in re.finditer(
            r'pub struct (\w+)\s*\{.*?const NAME:\s*&\'static str = "([^"]+)"',
            text,
            re.S,
        ):
            out[m.group(2)] = m.group(1)
    return out


def parse_proto_file(path: Path, enum_names: set[str]) -> list[Msg]:
    text = path.read_text()
    pkg_m = re.search(r"package\s+([\w.]+);", text)
    if not pkg_m:
        return []
    parts = pkg_m.group(1).split(".")
    package = parts[0]
    kind = parts[1] if len(parts) > 1 else "msg"
    if kind != "msg":
        return []

    msgs: list[Msg] = []
    # Only top-level messages (depth 0).
    i = 0
    depth = 0
    while i < len(text):
        if text[i] == "{":
            depth += 1
            i += 1
            continue
        if text[i] == "}":
            depth -= 1
            i += 1
            continue
        if depth == 0:
            m = re.match(r"\bmessage\s+(\w+)\s*\{", text[i:])
            if m:
                name = m.group(1)
                start = i + m.end()
                inner_depth = 1
                j = start
                while j < len(text) and inner_depth:
                    if text[j] == "{":
                        inner_depth += 1
                    elif text[j] == "}":
                        inner_depth -= 1
                    j += 1
                body = text[start : j - 1]
                msgs.append(
                    Msg(
                        ros_type=f"{package}/msg/{name}",
                        package=package,
                        name=name,
                        proto_stem=path.stem,
                        fields=parse_fields(body, package, enum_names),
                    )
                )
                i = j
                continue
        i += 1
    return msgs


def parse_fields(body: str, package: str, enum_names: set[str]) -> list[Field]:
    fields: list[Field] = []
    # Strip nested message/enum blocks so their contents are not parsed as fields.
    stripped = []
    i = 0
    while i < len(body):
        m = re.match(r"\b(message|enum)\s+\w+\s*\{", body[i:])
        if m:
            start = i + m.end()
            depth = 1
            j = start
            while j < len(body) and depth:
                if body[j] == "{":
                    depth += 1
                elif body[j] == "}":
                    depth -= 1
                j += 1
            i = j
            continue
        stripped.append(body[i])
        i += 1
    flat = re.sub(r"//.*?$", "", "".join(stripped), flags=re.M)

    for line in flat.split(";"):
        line = line.strip()
        if not line:
            continue
        fm = re.match(
            r"(repeated\s+|optional\s+)?([\w.]+)\s+(\w+)\s*=\s*\d+",
            line,
        )
        if not fm:
            continue
        prefix = (fm.group(1) or "").strip()
        repeated = prefix == "repeated"
        optional = prefix == "optional"
        ptype = fm.group(2)
        fname = fm.group(3)
        is_bytes = ptype == "bytes"
        is_string = ptype == "string"
        is_timestamp = ptype == "google.protobuf.Timestamp"
        is_duration = ptype == "google.protobuf.Duration"
        last = ptype.rsplit(".", 1)[-1]
        is_enum = (not is_timestamp and not is_duration
                   and ptype not in SCALAR_ROS
                   and ptype not in ("bytes", "string", "bool")
                   and last in enum_names)
        nested = None
        if (
            not is_timestamp
            and not is_duration
            and not is_enum
            and ptype not in SCALAR_ROS
            and ptype not in ("bytes", "string", "bool")
        ):
            if "." in ptype:
                segs = ptype.split(".")
                nested = f"{segs[0]}/msg/{segs[-1]}"
            else:
                nested = f"{package}/msg/{ptype}"
        fields.append(
            Field(
                name=fname,
                proto_type=ptype,
                repeated=repeated,
                optional=optional,
                is_bytes=is_bytes,
                is_string=is_string,
                is_timestamp=is_timestamp,
                is_duration=is_duration,
                is_enum=is_enum,
                nested_ros_type=nested,
            )
        )
    return fields


def load_all_msgs(enum_names: set[str]) -> dict[str, Msg]:
    by_type: dict[str, Msg] = {}
    for path in PROTO.rglob("*/msg/v1/*.proto"):
        for msg in parse_proto_file(path, enum_names):
            by_type[msg.ros_type] = msg
    return by_type


def existing_mappers() -> list[tuple[Path, str, str]]:
    out = []
    skip = {"mod.rs", "common.rs", "convert.rs"}
    for path in MAPPERS.rglob("*.rs"):
        if path.name in skip or path.parent.name in {
            "action",
            "service",
            "action_bridges",
            "service_bridges",
        }:
            continue
        if path.name in {"action_bridges.rs", "service_bridges.rs"}:
            continue
        text = path.read_text()
        sm = re.search(r"pub struct (\w+Mapper);", text)
        tm = re.search(r"//! Typed mapper for `([^`]+)`", text)
        if not sm or not tm:
            continue
        out.append((path, sm.group(1), tm.group(1)))
    return out


def convert_fn_name(path: Path) -> str:
    return path.stem


def nested_fn_path(ros_type: str, existing: dict[str, tuple[Path, str]]) -> str | None:
    if ros_type not in existing:
        return None
    path, _struct = existing[ros_type]
    rel = path.relative_to(MAPPERS)
    pkg = rel.parts[0]
    stem = rel.stem
    return f"crate::ros2_bridge::mappers::{pkg}::{stem}::{stem}"


def to_bus_expr(msg: Msg, f: Field, existing: dict[str, tuple[Path, str]]) -> str:
    ros = f"msg.{ros_field(f.name)}"
    key = (msg.ros_type, f.name)
    if key in WRAPPER_CAST:
        return WRAPPER_CAST[key][1].format(ros=ros, bus="unused")
    if key in OCTET_BOOL:
        return f"crate::ros2_bridge::mappers::convert::octet_to_bool({ros})"
    if f.is_timestamp:
        conv = "crate::ros2_bridge::mappers::convert::time_to_timestamp"
        if f.optional:
            return f"{ros}.map({conv})"
        return f"Some({conv}({ros}))"
    if f.is_duration:
        conv = "crate::ros2_bridge::mappers::convert::duration_to_proto"
        if f.optional:
            return f"{ros}.map({conv})"
        return f"Some({conv}({ros}))"
    if f.is_enum:
        return f"{ros} as i32"
    if f.is_bytes and key in INT8_BYTES:
        return f"crate::ros2_bridge::mappers::convert::i8_seq_to_bytes({ros})"
    if f.is_bytes:
        return f"crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec({ros})"
    if f.is_string and f.repeated:
        return f"crate::ros2_bridge::mappers::convert::string_seq({ros})"
    if f.is_string:
        return f"crate::ros2_bridge::mappers::convert::from_ros_string({ros})"
    if f.nested_ros_type:
        base = nested_fn_path(f.nested_ros_type, existing)
        if not base:
            raise SystemExit(
                f"nested {f.nested_ros_type} for {msg.ros_type}.{f.name} has no mapper"
            )
        if f.repeated:
            return f"{ros}.into_iter().map({base}_to_bus).collect()"
        if f.optional:
            return f"{ros}.map({base}_to_bus)"
        return f"Some({base}_to_bus({ros}))"
    if f.repeated:
        if f.proto_type == "double":
            return f"crate::ros2_bridge::mappers::convert::f64_seq({ros})"
        if f.proto_type == "float":
            return f"crate::ros2_bridge::mappers::convert::f32_seq({ros})"
        if f.proto_type == "int32":
            return f"crate::ros2_bridge::mappers::convert::i32_seq({ros})"
        if f.proto_type == "int64":
            return f"crate::ros2_bridge::mappers::convert::i64_seq({ros})"
        if f.proto_type in ("uint32", "fixed32"):
            return f"crate::ros2_bridge::mappers::convert::u32_seq({ros})"
        return f"{ros}.into_iter().collect()"
    if f.optional and f.proto_type in SCALAR_ROS:
        return ros
    # Proto int32/uint32 may be u8/i8/u16/… on distro rust IDL and u32/i32 on the shim.
    if f.proto_type in ("int32", "uint32"):
        return f"{ros}.into()"
    return ros


def to_ros_expr(msg: Msg, f: Field, existing: dict[str, tuple[Path, str]]) -> str:
    bus = f"bus.{bus_field(f.name)}"
    key = (msg.ros_type, f.name)
    if key in WRAPPER_CAST:
        return WRAPPER_CAST[key][2].format(ros="unused", bus=bus)
    if key in OCTET_BOOL:
        return f"crate::ros2_bridge::mappers::convert::bool_to_octet({bus})"
    if f.is_timestamp:
        conv = "crate::ros2_bridge::mappers::convert::timestamp_to_time"
        if f.optional:
            return f"{bus}.map({conv})"
        return f"{conv}({bus}.unwrap_or_default())"
    if f.is_duration:
        conv = "crate::ros2_bridge::mappers::convert::proto_to_duration"
        if f.optional:
            return f"{bus}.map({conv})"
        return f"{conv}({bus}.unwrap_or_default())"
    if f.is_enum:
        return f"{bus} as i32"
    if f.is_bytes and key in INT8_BYTES:
        return f"crate::ros2_bridge::mappers::convert::bytes_to_i8_seq({bus})"
    if f.is_bytes:
        return f"crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq({bus})"
    if f.is_string and f.repeated:
        return f"crate::ros2_bridge::mappers::convert::ros_string_seq({bus})"
    if f.is_string:
        return f"crate::ros2_bridge::mappers::convert::to_ros_string({bus})"
    if f.nested_ros_type:
        base = nested_fn_path(f.nested_ros_type, existing)
        if f.repeated:
            return f"{bus}.into_iter().map({base}_to_ros).collect()"
        if f.optional:
            return f"{bus}.map({base}_to_ros)"
        return f"{base}_to_ros({bus}.unwrap_or_default())"
    if f.repeated and f.proto_type == "double":
        return f"crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq({bus})"
    if f.repeated and f.proto_type in ("uint32", "fixed32"):
        return f"crate::ros2_bridge::mappers::convert::FromU32Seq::from_u32_seq({bus})"
    if f.repeated:
        return bus
    # Infer distro (u8) vs shim (u32) from the ROS field type.
    if f.proto_type in ("int32", "uint32"):
        return f"{bus} as _"
    return bus


def rust_ros_path(ros_type: str) -> str:
    pkg, _, name = ros_type.partition("/msg/")
    return f"ros_env::{pkg}::msg::{name}"


def rust_bus_path(ros_type: str, prost_names: dict[str, str]) -> str:
    pkg, _, name = ros_type.partition("/msg/")
    rust = prost_names.get(name, to_upper_camel(name))
    return f"crate::{pkg}::msg::v1::{rust}"


def emit_mapper(
    path: Path,
    struct: str,
    msg: Msg,
    existing: dict[str, tuple[Path, str]],
    prost_names: dict[str, str],
) -> str:
    fn = convert_fn_name(path)
    ros_ty = rust_ros_path(msg.ros_type)
    bus_ty = rust_bus_path(msg.ros_type, prost_names)
    to_bus_fields = []
    to_ros_fields = []
    for f in msg.fields:
        to_bus_fields.append(f"        {bus_field(f.name)}: {to_bus_expr(msg, f, existing)},")
        to_ros_fields.append(f"        {ros_field(f.name)}: {to_ros_expr(msg, f, existing)},")
    to_bus_body = "\n".join(to_bus_fields)
    to_ros_body = "\n".join(to_ros_fields)
    fn_msg = "msg" if msg.fields else "_msg"
    fn_bus = "bus" if msg.fields else "_bus"
    if msg.fields:
        to_bus_init = f"{bus_ty} {{\n{to_bus_body}\n    }}"
        to_ros_init = f"{ros_ty} {{\n{to_ros_body}\n    }}"
    else:
        to_bus_init = f"{bus_ty} {{}}"
        to_ros_init = f"{ros_ty} {{}}"
    return f'''//! Typed mapper for `{msg.ros_type}`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn {fn}_to_bus({fn_msg}: {ros_ty}) -> {bus_ty} {{
    {to_bus_init}
}}

pub(crate) fn {fn}_to_ros({fn_bus}: {bus_ty}) -> {ros_ty} {{
    {to_ros_init}
}}

#[derive(Clone, Copy, Debug, Default)]
pub struct {struct};

impl TypedTopicMapper for {struct} {{
    type Ros = {ros_ty};
    type Bus = {bus_ty};

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {{
        Ok({fn}_to_bus(msg))
    }}

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {{
        Ok({fn}_to_ros(msg))
    }}
}}
'''


def shim_field_ty(msg: Msg, f: Field) -> str:
    key = (msg.ros_type, f.name)
    if key in WRAPPER_CAST:
        ty = WRAPPER_CAST[key][0]
        return f"Option<{ty}>" if f.optional else ty
    if key in OCTET_BOOL:
        return "u8"
    if f.is_timestamp or f.is_duration:
        inner = "crate::builtin_interfaces::msg::Time" if f.is_timestamp else "crate::builtin_interfaces::msg::Duration"
        if f.repeated:
            return f"Vec<{inner}>"
        if f.optional:
            return f"Option<{inner}>"
        return inner
    if f.is_enum:
        return "Vec<i32>" if f.repeated else "i32"
    if f.is_bytes and key in INT8_BYTES:
        return "Vec<i8>"
    if f.is_bytes and msg.name == "UUID" and f.name == "uuid":
        return "[u8; 16]"
    if f.is_bytes:
        return "Vec<u8>"
    if f.is_string:
        return "Vec<rosidl_runtime_rs::String>" if f.repeated else "rosidl_runtime_rs::String"
    if f.nested_ros_type:
        pkg, _, name = f.nested_ros_type.partition("/msg/")
        inner = f"crate::{pkg}::msg::{name}"
        if f.repeated:
            return f"Vec<{inner}>"
        if f.optional:
            return f"Option<{inner}>"
        return inner
    if f.repeated:
        rust = SCALAR_ROS.get(f.proto_type, "i32")
        return f"Vec<{rust}>"
    if f.optional and f.proto_type in SCALAR_ROS:
        return f"Option<{SCALAR_ROS[f.proto_type]}>"
    if f.proto_type in SCALAR_ROS:
        return SCALAR_ROS[f.proto_type]
    if f.proto_type == "bool":
        return "bool"
    return "i32"


def emit_shim(msgs: list[Msg]) -> str:
    by_pkg: dict[str, list[Msg]] = defaultdict(list)
    for m in msgs:
        if m.package in SKIP_SHIM_PACKAGES:
            continue
        by_pkg[m.package].append(m)

    chunks = [
        "// Generated typed ROS message stubs for `use_ros_shim` (topic mappers).",
        "",
    ]
    for pkg in sorted(by_pkg):
        chunks.append(f"pub mod {pkg} {{")
        chunks.append("    pub mod msg {")
        seen = set()
        for m in sorted(by_pkg[pkg], key=lambda x: x.name):
            if m.name in seen:
                continue
            seen.add(m.name)
            fields = []
            for f in m.fields:
                ty = shim_field_ty(m, f)
                fields.append(f"            pub {ros_field(f.name)}: {ty},")
            field_block = "\n".join(fields)
            chunks.append(
                f"""
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct {m.name} {{
{field_block}
        }}
        impl_rmw!({m.name});
"""
            )
        chunks.append("    }")
        chunks.append("}")
        chunks.append("")
    return "\n".join(chunks)


PY_MAPPERS = ROOT / "bindings" / "python" / "robot_bus" / "ros2_bridge" / "mappers"
CPP_MAPPERS = ROOT / "bindings" / "cpp" / "include" / "robot_bus" / "ros2_bridge" / "mappers"
CPP_CONVERT = CPP_MAPPERS / "convert.hpp"

PY_GOLD_FILES = {
    PY_MAPPERS / "string.py",
    PY_MAPPERS / "image.py",
}
PY_GOLD_TYPES = {
    "std_msgs/msg/String",
    "sensor_msgs/msg/Image",
}
CPP_GOLD_TYPES = set(PY_GOLD_TYPES)

OPTIONAL_CPP_PACKAGES = {
    "nav2_msgs": "ROBOT_BUS_HAS_NAV2_MSGS",
    "control_msgs": "ROBOT_BUS_HAS_CONTROL_MSGS",
    "apriltag_msgs": "ROBOT_BUS_HAS_APRILTAG_MSGS",
    "foxglove_msgs": "ROBOT_BUS_HAS_FOXGLOVE_MSGS",
}

KEEP_PY_MANUAL = (
    "FibonacciActionMapper",
    "SensorMsgsImageMapper",
    "SetBoolServiceMapper",
    "StdMsgsStringMapper",
    "TriggerServiceMapper",
)


def py_mod_path(ros_type: str) -> tuple[str, str]:
    pkg, _, name = ros_type.partition("/msg/")
    return pkg, snake(name)


def nested_py_import(ros_type: str) -> tuple[str, str]:
    pkg, stem = py_mod_path(ros_type)
    return f"robot_bus.ros2_bridge.mappers.{pkg}.{stem}", stem


def py_ros_to_bus_value(msg: Msg, f: Field) -> str:
    ros = f"msg.{f.name}"
    key = (msg.ros_type, f.name)
    if key in WRAPPER_CAST:
        return f"int({ros})"
    if key in OCTET_BOOL:
        return f"bool({ros})"
    if f.is_timestamp:
        return f"_convert.time_to_timestamp({ros})"
    if f.is_duration:
        return f"_convert.duration_to_proto({ros})"
    if f.is_enum:
        return f"int({ros})"
    if f.is_bytes and key in INT8_BYTES:
        return f"_convert.i8_seq_to_bytes({ros})"
    if f.is_bytes:
        return f"bytes({ros})"
    if f.is_string and f.repeated:
        return f"[str(x) for x in {ros}]"
    if f.is_string:
        return f"str({ros})"
    if f.nested_ros_type:
        _, stem = nested_py_import(f.nested_ros_type)
        fn = f"{stem}_to_bus"
        if f.repeated:
            return f"[{fn}(x) for x in {ros}]"
        return f"{fn}({ros})"
    if f.repeated:
        return f"list({ros})"
    return ros


def py_bus_to_ros_value(msg: Msg, f: Field, bus_expr: str) -> str:
    key = (msg.ros_type, f.name)
    if key in WRAPPER_CAST:
        return f"int({bus_expr})"
    if key in OCTET_BOOL:
        return f"int(bool({bus_expr}))"
    if f.is_timestamp:
        return f"_convert.timestamp_to_time({bus_expr})"
    if f.is_duration:
        return f"_convert.proto_to_duration({bus_expr})"
    if f.is_enum:
        return f"int({bus_expr})"
    if f.is_bytes and key in INT8_BYTES:
        return f"_convert.bytes_to_i8_seq({bus_expr})"
    if f.is_bytes:
        return f"bytes({bus_expr})"
    if f.is_string and f.repeated:
        return f"[str(x) for x in {bus_expr}]"
    if f.is_string:
        return f"str({bus_expr})"
    if f.nested_ros_type:
        _, stem = nested_py_import(f.nested_ros_type)
        fn = f"{stem}_to_ros"
        if f.repeated:
            return f"[{fn}(x) for x in {bus_expr}]"
        return f"{fn}({bus_expr})"
    if f.repeated:
        return f"list({bus_expr})"
    return bus_expr


def emit_python(struct: str, msg: Msg, existing: dict[str, tuple[Path, str]]) -> str:
    pkg, stem = py_mod_path(msg.ros_type)
    nested_imports = []
    seen = set()
    for f in msg.fields:
        if f.nested_ros_type and f.nested_ros_type in existing:
            mod, nstem = nested_py_import(f.nested_ros_type)
            if nstem not in seen:
                seen.add(nstem)
                nested_imports.append(
                    f"from {mod} import {nstem}_to_bus, {nstem}_to_ros"
                )
    nested_block = "\n".join(nested_imports)
    to_bus_lines = []
    to_ros_lines = []
    for f in msg.fields:
        val = py_ros_to_bus_value(msg, f)
        if f.nested_ros_type and not f.repeated:
            to_bus_lines.append(f"    bus.{f.name}.CopyFrom({val})")
        elif f.nested_ros_type and f.repeated:
            to_bus_lines.append(f"    bus.{f.name}.extend({val})")
        elif f.repeated and not f.is_bytes:
            to_bus_lines.append(f"    bus.{f.name}.extend({val})")
        else:
            to_bus_lines.append(f"    bus.{f.name} = {val}")
        b = f"bus.{f.name}"
        to_ros_lines.append(f"    out.{f.name} = {py_bus_to_ros_value(msg, f, b)}")
    to_bus_body = "\n".join(to_bus_lines) if to_bus_lines else "    pass"
    to_ros_body = "\n".join(to_ros_lines) if to_ros_lines else "    pass"
    nested_extra = f"\n{nested_block}" if nested_block else ""
    return f'''"""Generated mapper for `{msg.ros_type}`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert{nested_extra}

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.{pkg}.msg.v1 import {msg.name} as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def {stem}_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
{to_bus_body}
    return bus


def {stem}_to_ros(bus):
    from {pkg}.msg import {msg.name} as RosMsg

    out = RosMsg()
{to_ros_body}
    return out


class {struct}:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from {pkg}.msg import {msg.name} as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return {stem}_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return {stem}_to_ros(bus)
'''


def cpp_pkg_guard(package: str) -> str | None:
    return OPTIONAL_CPP_PACKAGES.get(package)


def cpp_ros_include(msg: Msg) -> str:
    _, stem = py_mod_path(msg.ros_type)
    return f"<{msg.package}/msg/{stem}.hpp>"


def cpp_bus_type(msg: Msg) -> str:
    # C++ protobuf namespaces follow proto `package` (e.g. geometry_msgs.msg.v1),
    # not the include prefix `robot_bus/`.
    return f"::{msg.package}::msg::v1::{msg.name}"


def cpp_ros_type(msg: Msg) -> str:
    return f"::{msg.package}::msg::{msg.name}"


def cpp_nested_fn(ros_type: str, existing: dict[str, tuple[Path, str]], kind: str) -> str:
    path, _ = existing[ros_type]
    stem = path.stem
    ns = ros_type.split("/")[0]
    return f"::robot_bus::ros2_bridge_mappers::{ns}::{stem}_{kind}"


def cpp_to_bus_stmt(msg: Msg, f: Field, existing: dict[str, tuple[Path, str]]) -> str:
    ros = f"msg.{f.name}"
    key = (msg.ros_type, f.name)
    setter = f.name
    if key in WRAPPER_CAST:
        return f"  bus.set_{setter}(static_cast<int32_t>({ros}));"
    if key in OCTET_BOOL:
        return f"  bus.set_{setter}({ros} != 0);"
    if f.is_timestamp:
        return f"  *bus.mutable_{setter}() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp({ros});"
    if f.is_duration:
        return f"  *bus.mutable_{setter}() = ::robot_bus::ros2_bridge_mappers::duration_to_proto({ros});"
    if f.is_enum:
        return f"  bus.set_{setter}(static_cast<int32_t>({ros}));"
    if f.is_bytes and key in INT8_BYTES:
        return (
            f"  bus.set_{setter}(reinterpret_cast<const char *>({ros}.data()), {ros}.size());"
        )
    if f.is_bytes:
        return (
            f"  bus.set_{setter}(reinterpret_cast<const char *>({ros}.data()), {ros}.size());"
        )
    if f.is_string and f.repeated:
        return (
            f"  for (const auto &x : {ros}) {{\n    bus.add_{setter}(x.c_str());\n  }}"
        )
    if f.is_string:
        return f"  bus.set_{setter}({ros}.c_str());"
    if f.nested_ros_type:
        fn = cpp_nested_fn(f.nested_ros_type, existing, "to_bus")
        if f.repeated:
            return (
                f"  for (const auto &x : {ros}) {{\n    *bus.add_{setter}() = {fn}(x);\n  }}"
            )
        return f"  *bus.mutable_{setter}() = {fn}({ros});"
    if f.repeated:
        return f"  for (auto x : {ros}) {{\n    bus.add_{setter}(x);\n  }}"
    if f.proto_type == "bool":
        return f"  bus.set_{setter}({ros});"
    return f"  bus.set_{setter}({ros});"


def cpp_to_ros_stmt(msg: Msg, f: Field, existing: dict[str, tuple[Path, str]]) -> str:
    bus = f"bus.{f.name}()"
    key = (msg.ros_type, f.name)
    if key in WRAPPER_CAST:
        cpp_ty = {"i8": "int8_t", "i16": "int16_t", "u8": "uint8_t", "u16": "uint16_t"}.get(
            WRAPPER_CAST[key][0], "int32_t"
        )
        return f"  out.{f.name} = static_cast<{cpp_ty}>({bus});"
    if key in OCTET_BOOL:
        return f"  out.{f.name} = {bus} ? 1 : 0;"
    if f.is_timestamp:
        return f"  out.{f.name} = ::robot_bus::ros2_bridge_mappers::timestamp_to_time({bus});"
    if f.is_duration:
        return f"  out.{f.name} = ::robot_bus::ros2_bridge_mappers::proto_to_duration({bus});"
    if f.is_enum:
        return f"  out.{f.name} = {bus};"
    if f.is_bytes and key in INT8_BYTES:
        return f"  out.{f.name} = ::robot_bus::ros2_bridge_mappers::bytes_to_i8_seq({bus});"
    if f.is_bytes:
        return (
            f"  out.{f.name}.assign({bus}.begin(), {bus}.end());"
        )
    if f.is_string and f.repeated:
        return (
            f"  out.{f.name}.clear();\n"
            f"  for (const auto &x : {bus}) {{\n    out.{f.name}.push_back(x);\n  }}"
        )
    if f.is_string:
        return f"  out.{f.name} = {bus};"
    if f.nested_ros_type:
        fn = cpp_nested_fn(f.nested_ros_type, existing, "to_ros")
        if f.repeated:
            return (
                f"  out.{f.name}.clear();\n"
                f"  for (const auto &x : {bus}) {{\n    out.{f.name}.push_back({fn}(x));\n  }}"
            )
        return f"  out.{f.name} = {fn}({bus});"
    if f.repeated:
        return (
            f"  out.{f.name}.assign({bus}.begin(), {bus}.end());"
        )
    return f"  out.{f.name} = {bus};"


def emit_cpp_convert_hpp() -> str:
    return '''#pragma once

#include <cstdint>
#include <string>
#include <vector>

#ifdef ROBOT_BUS_HAS_ROS2
#include <builtin_interfaces/msg/duration.hpp>
#include <builtin_interfaces/msg/time.hpp>
#include <google/protobuf/duration.pb.h>
#include <google/protobuf/timestamp.pb.h>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {

inline std::string i8_seq_to_bytes(const std::vector<int8_t> &data) {
  if (data.empty()) {
    return {};
  }
  return std::string(reinterpret_cast<const char *>(data.data()), data.size());
}

inline std::vector<int8_t> bytes_to_i8_seq(const std::string &data) {
  if (data.empty()) {
    return {};
  }
  const auto *p = reinterpret_cast<const int8_t *>(data.data());
  return std::vector<int8_t>(p, p + data.size());
}

#ifdef ROBOT_BUS_HAS_ROS2
inline google::protobuf::Timestamp time_to_timestamp(const builtin_interfaces::msg::Time &t) {
  google::protobuf::Timestamp out;
  out.set_seconds(t.sec);
  out.set_nanos(static_cast<int32_t>(t.nanosec));
  return out;
}

inline builtin_interfaces::msg::Time timestamp_to_time(const google::protobuf::Timestamp &t) {
  builtin_interfaces::msg::Time out;
  out.sec = static_cast<int32_t>(t.seconds());
  out.nanosec = static_cast<uint32_t>(t.nanos());
  return out;
}

inline google::protobuf::Duration duration_to_proto(const builtin_interfaces::msg::Duration &d) {
  google::protobuf::Duration out;
  out.set_seconds(d.sec);
  out.set_nanos(d.nanosec);
  return out;
}

inline builtin_interfaces::msg::Duration proto_to_duration(const google::protobuf::Duration &d) {
  builtin_interfaces::msg::Duration out;
  out.sec = static_cast<int32_t>(d.seconds());
  out.nanosec = d.nanos();
  return out;
}
#endif

}  // namespace ros2_bridge_mappers
}  // namespace robot_bus
'''


def emit_cpp(struct: str, msg: Msg, existing: dict[str, tuple[Path, str]]) -> str:
    pkg, stem = py_mod_path(msg.ros_type)
    guard = cpp_pkg_guard(pkg)
    nested_includes = []
    seen = set()
    for f in msg.fields:
        if f.nested_ros_type and f.nested_ros_type in existing:
            npkg, nstem = py_mod_path(f.nested_ros_type)
            key = (npkg, nstem)
            if key not in seen:
                seen.add(key)
                nested_includes.append(
                    f'#include <robot_bus/ros2_bridge/mappers/{npkg}/{nstem}.hpp>'
                )
    nested_inc = "\n".join(nested_includes)
    to_bus_stmts = "\n".join(cpp_to_bus_stmt(msg, f, existing) for f in msg.fields)
    to_ros_stmts = "\n".join(cpp_to_ros_stmt(msg, f, existing) for f in msg.fields)
    bus_ty = cpp_bus_type(msg)
    ros_ty = cpp_ros_type(msg)
    ros_inc = cpp_ros_include(msg)
    has_ros = "defined(ROBOT_BUS_HAS_ROS2)"
    if guard:
        has_ros = f"defined(ROBOT_BUS_HAS_ROS2) && defined({guard})"
    body = f'''#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/{pkg}/msg/v1/{msg.proto_stem}.pb.h>
{nested_inc}

#if {has_ros}
#include {ros_inc}
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {{
namespace ros2_bridge_mappers {{
namespace {pkg} {{

#if {has_ros}
inline {bus_ty} {stem}_to_bus(const {ros_ty} &msg) {{
  {bus_ty} bus;
{to_bus_stmts}
  return bus;
}}

inline {ros_ty} {stem}_to_ros(const {bus_ty} &bus) {{
  {ros_ty} out;
{to_ros_stmts}
  return out;
}}
#endif

}}  // namespace {pkg}
}}  // namespace ros2_bridge_mappers
'''
    if msg.ros_type in CPP_GOLD_TYPES:
        return body + "}  // namespace robot_bus\n"
    body += f'''
#if {has_ros}
class {struct}
    : public TypedTopicMapper<{struct}, {ros_ty}> {{
 public:
  std::vector<uint8_t> ros_to_bus(const {ros_ty} &msg) const {{
    auto bus = ros2_bridge_mappers::{pkg}::{stem}_to_bus(msg);
    return encode_pb(bus);
  }}

  {ros_ty} bus_to_ros(BytesView payload) const {{
    {bus_ty} bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::{pkg}::{stem}_to_ros(bus);
  }}
}};
#else
struct {struct} : TopicMapper {{}};
#endif

}}  // namespace robot_bus
'''
    return body


def write_python_init(generated: list[tuple[str, str, str]]) -> None:
    """generated: (pkg, stem, struct) plus keep manual exports."""
    lines = [
        '"""Built-in and duck-typed ROS 2 ↔ robot-bus mappers."""',
        "",
        "from robot_bus.ros2_bridge.mappers.fibonacci import FibonacciActionMapper",
        "from robot_bus.ros2_bridge.mappers.image import SensorMsgsImageMapper",
        "from robot_bus.ros2_bridge.mappers.set_bool import SetBoolServiceMapper",
        "from robot_bus.ros2_bridge.mappers.string import StdMsgsStringMapper",
        "from robot_bus.ros2_bridge.mappers.trigger import TriggerServiceMapper",
        "",
    ]
    all_names = list(KEEP_PY_MANUAL)
    by_pkg: dict[str, list[tuple[str, str]]] = defaultdict(list)
    for pkg, stem, struct in generated:
        by_pkg[pkg].append((stem, struct))
        lines.append(
            f"from robot_bus.ros2_bridge.mappers.{pkg}.{stem} import {struct}"
        )
        all_names.append(struct)
    lines.append("")
    lines.append("__all__ = [")
    for name in all_names:
        lines.append(f'    "{name}",')
    lines.append("]")
    lines.append("")
    PY_MAPPERS.joinpath("__init__.py").write_text("\n".join(lines))
    for pkg, items in by_pkg.items():
        pkg_init = ["from __future__ import annotations", ""]
        exports = []
        for stem, struct in items:
            pkg_init.append(f"from .{stem} import {struct}")
            exports.append(struct)
        pkg_init.append("")
        pkg_init.append("__all__ = [")
        for name in exports:
            pkg_init.append(f'    "{name}",')
        pkg_init.append("]")
        pkg_init.append("")
        (PY_MAPPERS / pkg / "__init__.py").write_text("\n".join(pkg_init))


def emit_python_convert() -> str:
    return '''"""Shared field conversions for generated topic mappers."""

from __future__ import annotations

import array


def i8_seq_to_bytes(data) -> bytes:
    """Bulk `int8[]` → proto `bytes` (no per-cell Python ints)."""
    if isinstance(data, (bytes, bytearray, memoryview)):
        return bytes(data)
    if isinstance(data, array.array) and data.typecode in ("b", "B"):
        return data.tobytes()
    return array.array("b", data).tobytes()


def bytes_to_i8_seq(data: bytes):
    """Bulk proto `bytes` → ROS `int8[]` as `array.array('b')`."""
    return array.array("b", data)


def time_to_timestamp(t):
    from google.protobuf.timestamp_pb2 import Timestamp

    out = Timestamp()
    out.seconds = int(t.sec)
    out.nanos = int(t.nanosec)
    return out


def timestamp_to_time(ts):
    from builtin_interfaces.msg import Time

    out = Time()
    out.sec = int(ts.seconds)
    out.nanosec = int(ts.nanos)
    return out


def duration_to_proto(d):
    from google.protobuf.duration_pb2 import Duration

    out = Duration()
    out.seconds = int(d.sec)
    out.nanos = int(d.nanosec)
    return out


def proto_to_duration(d):
    from builtin_interfaces.msg import Duration

    out = Duration()
    out.sec = int(d.seconds)
    out.nanosec = int(d.nanos)
    return out
'''


def main() -> None:
    enum_names = collect_enum_names()
    prost_names = load_prost_names()
    all_msgs = load_all_msgs(enum_names)
    mappers = existing_mappers()
    existing_by_type = {ros: (path, struct) for path, struct, ros in mappers}

    missing = []
    for path, struct, ros_type in mappers:
        if ros_type not in all_msgs:
            missing.append((path, ros_type))
    if missing:
        raise SystemExit(
            "proto missing for:\n"
            + "\n".join(f"  {p}: {t}" for p, t in missing)
        )

    written = 0
    skipped = 0
    for path, struct, ros_type in mappers:
        if path in GOLD:
            skipped += 1
            continue
        msg = all_msgs[ros_type]
        path.write_text(emit_mapper(path, struct, msg, existing_by_type, prost_names))
        written += 1

    shim_msgs = [all_msgs[ros] for _, _, ros in mappers if ros in all_msgs]
    SHIM_OUT.write_text(emit_shim(shim_msgs))

    PY_MAPPERS.mkdir(parents=True, exist_ok=True)
    (PY_MAPPERS / "_convert.py").write_text(emit_python_convert())
    CPP_MAPPERS.mkdir(parents=True, exist_ok=True)
    CPP_CONVERT.write_text(emit_cpp_convert_hpp())

    py_generated: list[tuple[str, str, str]] = []
    py_written = 0
    cpp_written = 0
    for path, struct, ros_type in mappers:
        msg = all_msgs[ros_type]
        pkg, stem = py_mod_path(ros_type)
        dest = PY_MAPPERS / pkg / f"{stem}.py"
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(emit_python(struct, msg, existing_by_type))
        if ros_type not in PY_GOLD_TYPES:
            py_generated.append((pkg, stem, struct))
        py_written += 1
        dest = CPP_MAPPERS / pkg / f"{stem}.hpp"
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(emit_cpp(struct, msg, existing_by_type))
        cpp_written += 1

    write_python_init(py_generated)
    print(
        f"wrote {written} rust mappers (skipped {skipped} gold), "
        f"{py_written} python, {cpp_written} cpp, {SHIM_OUT}"
    )


if __name__ == "__main__":
    main()
