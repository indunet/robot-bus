#!/usr/bin/env python3
"""Generate typed ROS↔protobuf topic mappers and ros-env-shim message stubs.

Reads proto/*/msg/v1/*.proto plus existing mapper type_name() strings, then
rewrites src/ros2_bridge/mappers/<pkg>/<msg>.rs (except gold samples) and
third_party/ros-env-shim/src/generated_msgs.rs.
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
        tm = re.search(
            r'fn type_name\(&self\) -> &(?:\'static )?str \{\s*"([^"]+)"',
            text,
        )
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
    return f'''//! Typed mapper for `{msg.ros_type}`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn {fn}_to_bus({fn_msg}: {ros_ty}) -> {bus_ty} {{
    {bus_ty} {{
{to_bus_body}
    }}
}}

pub(crate) fn {fn}_to_ros({fn_bus}: {bus_ty}) -> {ros_ty} {{
    {ros_ty} {{
{to_ros_body}
    }}
}}

#[derive(Clone, Copy, Debug, Default)]
pub struct {struct};

impl TypedTopicMapper for {struct} {{
    type Ros = {ros_ty};
    type Bus = {bus_ty};

    fn type_name(&self) -> &'static str {{
        "{msg.ros_type}"
    }}

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
    print(f"wrote {written} mappers (skipped {skipped} gold), {SHIM_OUT}")


if __name__ == "__main__":
    main()
