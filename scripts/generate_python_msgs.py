#!/usr/bin/env python3
"""Generate Python protobuf modules under python/robot_bus/<pkg>/{msg|srv}/v1/.

Requires ``protoc`` on PATH (override with env ``PROTOC``). Regenerates
committed sources under ``python/robot_bus/``; run after changing ``proto/``.

The generated ``*_pb2.py`` files are rewritten so imports use the
``robot_bus.<pkg>…`` namespace (no top-level ``sensor_msgs`` install).

Usage (from repo root)::

  python3 scripts/generate_python_msgs.py

Pin ``protoc`` to ``EXPECTED_PROTOC_VERSION`` (same as CI). Match the
installed ``protobuf`` pip package to that generator (``>=7.35,<8`` for
protoc 35.x). Override the check with ``ROBOT_BUS_SKIP_PROTOC_VERSION_CHECK=1``
only for local experiments.
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
OUT_ROOT = ROOT / "python" / "robot_bus"
PROTOC = os.environ.get("PROTOC", "protoc")

# Keep in sync with .github/workflows/ci.yml and publish-pypi.yml.
EXPECTED_PROTOC_VERSION = "35.1"

# Top-level packages that own message/service protos (not robot_bus.grpc).
MSG_PACKAGES = (
    "builtin_interfaces",
    "control_msgs",
    "diagnostic_msgs",
    "foxglove_msgs",
    "geometry_msgs",
    "nav2_msgs",
    "nav_msgs",
    "sensor_msgs",
    "shape_msgs",
    "std_msgs",
    "std_srvs",
    "tf2_msgs",
    "trajectory_msgs",
    "unique_identifier_msgs",
    "visualization_msgs",
)

IMPORT_RE = re.compile(
    r"^((?:from|import)\s+)("
    + "|".join(re.escape(p) for p in MSG_PACKAGES)
    + r")(\b)",
    re.MULTILINE,
)

# protoc embeds the Python module path in BuildTopDescriptorsAndMessages.
MODULE_STR_RE = re.compile(
    r"(['\"])("
    + "|".join(re.escape(p) for p in MSG_PACKAGES)
    + r")(\.[^'\"]*_pb2)(['\"])"
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
        # gRPC stubs are generated separately; keep robot_bus/action for prost/pb2.
        if rel.startswith("robot_bus/grpc/"):
            continue
        protos.append(path)
    return sorted(protos)


def run_protoc(protos: list[Path], out_dir: Path) -> None:
    cmd = [
        PROTOC,
        f"--proto_path={PROTO_ROOT}",
        f"--python_out={out_dir}",
        f"--pyi_out={out_dir}",
        *[str(p) for p in protos],
    ]
    subprocess.run(cmd, check=True)


def rewrite_imports(text: str) -> str:
    text = IMPORT_RE.sub(r"\1robot_bus.\2\3", text)
    text = MODULE_STR_RE.sub(r"\1robot_bus.\2\3\4", text)
    return text


def clear_generated_packages() -> None:
    for pkg in MSG_PACKAGES:
        target = OUT_ROOT / pkg
        if target.exists():
            shutil.rmtree(target)
    # Built-in robot_bus.action lives at python/robot_bus/action/ (not a nested pkg).
    action_dst = OUT_ROOT / "action"
    if action_dst.exists():
        shutil.rmtree(action_dst)


def copy_and_rewrite(tmp: Path) -> None:
    for pkg in MSG_PACKAGES:
        src = tmp / pkg
        if not src.exists():
            continue
        dst = OUT_ROOT / pkg
        shutil.copytree(src, dst)
        for py_file in dst.rglob("*.py*"):
            if py_file.suffix not in {".py", ".pyi"}:
                continue
            text = py_file.read_text(encoding="utf-8")
            rewritten = rewrite_imports(text)
            if rewritten != text:
                py_file.write_text(rewritten, encoding="utf-8")

    # robot_bus.action.v1 → python/robot_bus/action/v1 (import robot_bus.action.v1).
    action_src = tmp / "robot_bus" / "action"
    if action_src.exists():
        action_dst = OUT_ROOT / "action"
        shutil.copytree(action_src, action_dst)


def ensure_package_inits(pkg_dir: Path) -> None:
    """Create missing __init__.py along the package tree."""
    for directory in [pkg_dir, *pkg_dir.rglob("*")]:
        if not directory.is_dir():
            continue
        init = directory / "__init__.py"
        if not init.exists():
            init.write_text("", encoding="utf-8")


def write_leaf_reexports(pkg_dir: Path) -> None:
    """For each msg/v1, srv/v1, or action/v1 dir, re-export classes from *_pb2."""
    for leaf in (
        list(pkg_dir.rglob("msg"))
        + list(pkg_dir.rglob("srv"))
        + list(pkg_dir.rglob("action"))
    ):
        v1 = leaf / "v1"
        if not v1.is_dir():
            continue
        pb2_files = sorted(v1.glob("*_pb2.py"))
        if not pb2_files:
            continue
        lines = [
            "# Generated by scripts/generate_python_msgs.py — do not edit by hand.",
            '"""Re-export protobuf message classes for this package version."""',
            "",
        ]
        for pb2 in pb2_files:
            mod = pb2.stem  # imu_pb2
            lines.append(f"from .{mod} import *  # noqa: F403")
        lines.append("")
        lines.append("__all__ = [name for name in globals() if not name.startswith('_')]")
        lines.append("")
        (v1 / "__init__.py").write_text("\n".join(lines), encoding="utf-8")


def write_action_leaf_reexports() -> None:
    """Re-export robot_bus.action.v1 classes (layout differs from ROS pkgs)."""
    v1 = OUT_ROOT / "action" / "v1"
    if not v1.is_dir():
        return
    pb2_files = sorted(v1.glob("*_pb2.py"))
    if not pb2_files:
        return
    lines = [
        "# Generated by scripts/generate_python_msgs.py — do not edit by hand.",
        '"""Re-export protobuf message classes for robot_bus.action.v1."""',
        "",
    ]
    for pb2 in pb2_files:
        lines.append(f"from .{pb2.stem} import *  # noqa: F403")
    lines.append("")
    lines.append("__all__ = [name for name in globals() if not name.startswith('_')]")
    lines.append("")
    (v1 / "__init__.py").write_text("\n".join(lines), encoding="utf-8")


def write_pkg_root_init(pkg: str) -> None:
    path = OUT_ROOT / pkg / "__init__.py"
    path.write_text(
        f'# Generated package root for robot_bus.{pkg}\n'
        f'"""ROS-style protobuf types: robot_bus.{pkg}.{{msg|srv|action}}.v1"""\n',
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

    OUT_ROOT.mkdir(parents=True, exist_ok=True)
    clear_generated_packages()

    with tempfile.TemporaryDirectory(prefix="robot-bus-py-msgs-") as tmp_name:
        tmp = Path(tmp_name)
        print(f"{PROTOC} ({protoc_version(PROTOC)}) → {len(protos)} files …")
        run_protoc(protos, tmp)
        copy_and_rewrite(tmp)

    for pkg in MSG_PACKAGES:
        pkg_dir = OUT_ROOT / pkg
        if not pkg_dir.exists():
            continue
        ensure_package_inits(pkg_dir)
        write_leaf_reexports(pkg_dir)
        write_pkg_root_init(pkg)

    action_dir = OUT_ROOT / "action"
    if action_dir.exists():
        ensure_package_inits(action_dir)
        write_action_leaf_reexports()

    print(f"wrote Python msgs under {OUT_ROOT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
