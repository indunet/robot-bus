#!/usr/bin/env python3
"""Generate C++ protobuf sources under bindings/cpp/generated/robot_bus/.

Requires ``protoc`` on PATH (override with env ``PROTOC``). Outputs are
gitignored; CI and package workflows run this before build/publish so DEB/MSI
embed the stubs. End users who install the SDK do not need protoc.

``protoc --cpp_out`` emits Google-style ``*.pb.h`` / ``*.pb.cc``; we keep those
extensions (hand-written SDK code uses ``.hpp`` / ``.cpp``). Public includes:

  #include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>
  #include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>

The outer ``robot_bus/`` is the SDK include prefix. Built-in protos under
``proto/robot_bus_interfaces/…`` keep that path segment.

Usage (from repo root)::

  python3 scripts/generate_cpp_msgs.py
  # or: just gen-cpp

Pin ``protoc`` to ``EXPECTED_PROTOC_VERSION`` (same as CI).
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
OUT_ROOT = ROOT / "bindings" / "cpp" / "generated" / "robot_bus"
GENERATED_ROOT = ROOT / "bindings" / "cpp" / "generated"
PROTOC = os.environ.get("PROTOC", "protoc")

EXPECTED_PROTOC_VERSION = "35.1"

MSG_PACKAGES = (
    "ackermann_msgs",
    "action_msgs",
    "apriltag_msgs",
    "builtin_interfaces",
    "control_msgs",
    "diagnostic_msgs",
    "example_interfaces",
    "foxglove_msgs",
    "geometry_msgs",
    "lifecycle_msgs",
    "map_msgs",
    "nav2_msgs",
    "nav_msgs",
    "sensor_msgs",
    "shape_msgs",
    "std_msgs",
    "std_srvs",
    "stereo_msgs",
    "tf2_msgs",
    "trajectory_msgs",
    "unique_identifier_msgs",
    "vision_msgs",
    "visualization_msgs",
)

PROTOC_VERSION_RE = re.compile(r"libprotoc\s+(\d+\.\d+(?:\.\d+)?)")

INCLUDE_RE = re.compile(
    r'(#include\s+)([<"])('
    + "|".join(re.escape(p) for p in MSG_PACKAGES)
    + r"|robot_bus_interfaces"
    + r')(/[^">]+)([>"])'
)


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
        if rel.startswith("robot_bus_interfaces/grpc/"):
            continue
        protos.append(path)
    return sorted(protos)


def run_protoc(protos: list[Path], out_dir: Path) -> None:
    cmd = [
        PROTOC,
        f"--proto_path={PROTO_ROOT}",
        f"--cpp_out={out_dir}",
        *[str(p) for p in protos],
    ]
    subprocess.run(cmd, check=True)


def rewrite_includes(text: str) -> str:
    def repl(m: re.Match[str]) -> str:
        directive, _quote_open, pkg, rest, _quote_close = m.groups()
        # ROS pkgs: sensor_msgs/... → robot_bus/sensor_msgs/...
        # Built-in: robot_bus_interfaces/action/... → robot_bus/robot_bus_interfaces/action/...
        if pkg == "robot_bus_interfaces":
            return f"{directive}<robot_bus/robot_bus_interfaces{rest}>"
        return f"{directive}<robot_bus/{pkg}{rest}>"

    return INCLUDE_RE.sub(repl, text)


def clear_output() -> None:
    if GENERATED_ROOT.exists():
        shutil.rmtree(GENERATED_ROOT)
    OUT_ROOT.mkdir(parents=True, exist_ok=True)


def copy_and_rewrite(tmp: Path) -> None:
    for pkg in MSG_PACKAGES:
        src = tmp / pkg
        if not src.exists():
            continue
        dst = OUT_ROOT / pkg
        shutil.copytree(src, dst)

    # proto/robot_bus_interfaces/... → generated/robot_bus/robot_bus_interfaces/...
    iface_src = tmp / "robot_bus_interfaces"
    if iface_src.exists():
        for child in iface_src.iterdir():
            if child.name == "grpc":
                continue
            dst = OUT_ROOT / "robot_bus_interfaces" / child.name
            if dst.exists():
                shutil.rmtree(dst)
            shutil.copytree(child, dst)

    for path in OUT_ROOT.rglob("*"):
        if not path.name.endswith((".pb.h", ".pb.cc")):
            continue
        text = path.read_text(encoding="utf-8")
        rewritten = rewrite_includes(text)
        if rewritten != text:
            path.write_text(rewritten, encoding="utf-8")


def write_readme() -> None:
    (GENERATED_ROOT / "README.md").write_text(
        "# Generated C++ protobuf stubs\n\n"
        "Produced by `scripts/generate_cpp_msgs.py` (`just gen-cpp`).\n"
        "Do not edit by hand; re-run after changing `proto/`.\n\n"
        "Extensions stay as protoc defaults (`*.pb.h` / `*.pb.cc`). "
        "Public includes use the `robot_bus/` prefix (no `generated/` segment):\n\n"
        "```cpp\n"
        "#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>\n"
        "// → bindings/cpp/generated/robot_bus/sensor_msgs/msg/v1/imu.pb.h\n"
        "\n"
        "#include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>\n"
        "// → proto/example_interfaces/action/v1/fibonacci.proto\n"
        "```\n",
        encoding="utf-8",
    )


def main() -> int:
    if not shutil.which(PROTOC) and not Path(PROTOC).is_file():
        print(f"error: protoc not found ({PROTOC!r})", file=sys.stderr)
        return 1

    ensure_protoc_version(PROTOC)

    protos = collect_protos()
    if not protos:
        print("error: no proto files found", file=sys.stderr)
        return 1

    clear_output()

    with tempfile.TemporaryDirectory(prefix="robot-bus-cpp-msgs-") as tmp_name:
        tmp = Path(tmp_name)
        print(f"{PROTOC} ({protoc_version(PROTOC)}) -> {len(protos)} files ...")
        run_protoc(protos, tmp)
        copy_and_rewrite(tmp)

    write_readme()
    print(f"wrote C++ msgs under {OUT_ROOT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
