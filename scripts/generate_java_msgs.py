#!/usr/bin/env python3
"""Generate Java protobuf stubs under bindings/java/generated/.

Requires ``protoc`` on PATH (override with env ``PROTOC``). Outputs are
gitignored; CI and Maven publish run this before package so the JAR embeds
the stubs. Consumers who depend on ``org.indunet:robot-bus`` do not need
protoc.

Each proto gets::

  option java_package = "org.indunet.robot.bus.<proto.package>";
  option java_multiple_files = true;
  option java_outer_classname = "<Stem>Proto";

``java_outer_classname`` avoids collisions between the file-descriptor class
(derived from ``color_rgba.proto`` → ``ColorRgba``) and messages like
``ColorRGBA`` under protobuf-java 4.35.

Imports mirror other languages::

  import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;
  // ↔ from robot_bus.sensor_msgs.msg.v1 import Imu
  // ↔ #include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>

Usage (from repo root)::

  python3 scripts/generate_java_msgs.py
  # or: just gen-java

Pin ``protoc`` to ``EXPECTED_PROTOC_VERSION`` (same as CI). Match
``com.google.protobuf:protobuf-java`` to that generator (4.35.x for
protoc 35.x). Override the check with
``ROBOT_BUS_SKIP_PROTOC_VERSION_CHECK=1`` only for local experiments.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROTO_ROOT = ROOT / "proto"
# Override with ROBOT_BUS_JAVA_OUT for Android: bindings/android/generated
OUT_ROOT = Path(
    os.environ.get(
        "ROBOT_BUS_JAVA_OUT",
        str(ROOT / "bindings" / "java" / "generated"),
    )
)
PROTOC = os.environ.get("PROTOC", "protoc")

EXPECTED_PROTOC_VERSION = "35.1"

JAVA_BASE = "org.indunet.robot.bus"

PACKAGE_RE = re.compile(r"^package\s+([\w.]+)\s*;", re.MULTILINE)
JAVA_PACKAGE_RE = re.compile(r"^option\s+java_package\s*=\s*[^;]+;\s*\n?", re.MULTILINE)
JAVA_MULTIPLE_RE = re.compile(
    r"^option\s+java_multiple_files\s*=\s*[^;]+;\s*\n?", re.MULTILINE
)
JAVA_OUTER_RE = re.compile(
    r"^option\s+java_outer_classname\s*=\s*[^;]+;\s*\n?", re.MULTILINE
)
PROTOC_VERSION_RE = re.compile(r"libprotoc\s+(\d+\.\d+(?:\.\d+)?)")


def protoc_version(protoc: str) -> str:
    out = subprocess.check_output([protoc, "--version"], text=True).strip()
    match = PROTOC_VERSION_RE.search(out)
    if not match:
        raise RuntimeError(f"could not parse protoc version from {out!r}")
    return match.group(1)


def ensure_protoc_version(protoc: str) -> None:
    if os.environ.get("ROBOT_BUS_SKIP_PROTOC_VERSION_CHECK") == "1":
        return
    version = protoc_version(protoc)
    if version != EXPECTED_PROTOC_VERSION:
        raise SystemExit(
            f"error: protoc {version} != required {EXPECTED_PROTOC_VERSION}\n"
            f"  Install https://github.com/protocolbuffers/protobuf/releases/tag/v{EXPECTED_PROTOC_VERSION}\n"
            f"  or set PROTOC to that binary. Skip with ROBOT_BUS_SKIP_PROTOC_VERSION_CHECK=1."
        )


def collect_protos() -> list[Path]:
    protos: list[Path] = []
    for path in PROTO_ROOT.rglob("*.proto"):
        rel = path.relative_to(PROTO_ROOT).as_posix()
        if rel.startswith("robot_bus_interface/grpc/"):
            continue
        protos.append(path)
    return sorted(protos)


def snake_to_pascal(stem: str) -> str:
    return "".join(part[:1].upper() + part[1:] for part in stem.split("_") if part)


def inject_java_options(text: str, proto_path: Path) -> str:
    match = PACKAGE_RE.search(text)
    if not match:
        raise ValueError(f"{proto_path}: missing package declaration")
    proto_pkg = match.group(1)
    java_pkg = f"{JAVA_BASE}.{proto_pkg}"
    outer = snake_to_pascal(proto_path.stem) + "Proto"
    text = JAVA_PACKAGE_RE.sub("", text)
    text = JAVA_MULTIPLE_RE.sub("", text)
    text = JAVA_OUTER_RE.sub("", text)
    options = (
        f'option java_package = "{java_pkg}";\n'
        f"option java_multiple_files = true;\n"
        f'option java_outer_classname = "{outer}";\n'
    )
    insert_at = match.end()
    if insert_at < len(text) and text[insert_at] == "\n":
        insert_at += 1
    return text[:insert_at] + options + text[insert_at:]


def prepare_proto_tree(tmp: Path) -> list[Path]:
    """Copy protos into tmp with java_package / outer classname options injected."""
    prepared: list[Path] = []
    for src in collect_protos():
        rel = src.relative_to(PROTO_ROOT)
        dst = tmp / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        text = src.read_text(encoding="utf-8")
        dst.write_text(inject_java_options(text, src), encoding="utf-8")
        prepared.append(dst)
    return prepared


def clear_generated() -> None:
    if OUT_ROOT.exists():
        shutil.rmtree(OUT_ROOT)
    OUT_ROOT.mkdir(parents=True, exist_ok=True)


def run_protoc(proto_root: Path, protos: list[Path]) -> None:
    cmd = [
        PROTOC,
        f"--proto_path={proto_root}",
        f"--java_out={OUT_ROOT}",
        *[str(p) for p in protos],
    ]
    subprocess.run(cmd, check=True)


def main() -> int:
    if not shutil.which(PROTOC) and not Path(PROTOC).is_file():
        print(f"error: protoc not found ({PROTOC!r})", file=sys.stderr)
        return 1

    ensure_protoc_version(PROTOC)

    if not collect_protos():
        print("error: no proto files found", file=sys.stderr)
        return 1

    clear_generated()

    with tempfile.TemporaryDirectory(prefix="robot-bus-java-msgs-") as tmp_name:
        tmp = Path(tmp_name)
        prepared = prepare_proto_tree(tmp)
        print(f"{PROTOC} ({protoc_version(PROTOC)}) → {len(prepared)} files …")
        run_protoc(tmp, prepared)

    imu = (
        OUT_ROOT
        / "org"
        / "indunet"
        / "robot"
        / "bus"
        / "sensor_msgs"
        / "msg"
        / "v1"
        / "Imu.java"
    )
    color = (
        OUT_ROOT
        / "org"
        / "indunet"
        / "robot"
        / "bus"
        / "std_msgs"
        / "msg"
        / "v1"
        / "ColorRGBA.java"
    )
    if not imu.is_file():
        print(f"error: expected generated file missing: {imu}", file=sys.stderr)
        return 1
    if not color.is_file():
        print(f"error: expected generated file missing: {color}", file=sys.stderr)
        return 1
    color_text = color.read_text(encoding="utf-8")
    if "class ColorRGBA " not in color_text and "class ColorRGBA\n" not in color_text:
        # Message class line is typically `public final class ColorRGBA extends`
        if "public final class ColorRGBA extends" not in color_text:
            print(
                f"error: {color} does not contain message class ColorRGBA "
                "(outer classname collision?)",
                file=sys.stderr,
            )
            return 1

    print(f"wrote Java msgs under {OUT_ROOT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
