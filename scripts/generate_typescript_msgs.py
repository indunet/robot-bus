#!/usr/bin/env python3
"""Generate TypeScript protobuf modules under bindings/typescript/generated/.

Requires ``protoc`` on PATH (override with env ``PROTOC``) and
``@protobuf-ts/plugin`` installed under ``bindings/typescript``
(``npm install`` there once). Outputs are gitignored; CI and npm publish
run this before pack so the package embeds stubs. Consumers do not need
protoc.

Usage (from repo root)::

  python3 scripts/generate_typescript_msgs.py
  # or: just gen-typescript

Pin ``protoc`` to ``EXPECTED_PROTOC_VERSION`` (same as CI / Python codegen).
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
TS_ROOT = ROOT / "bindings" / "typescript"
OUT_ROOT = TS_ROOT / "generated"
PROTOC = os.environ.get("PROTOC", "protoc")

EXPECTED_PROTOC_VERSION = "35.1"

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


def find_protoc_gen_ts_js() -> Path:
    """Resolve the Node entry for @protobuf-ts/plugin (the real plugin script)."""
    js = TS_ROOT / "node_modules" / "@protobuf-ts" / "plugin" / "bin" / "protoc-gen-ts"
    if js.is_file():
        return js.resolve()
    raise SystemExit(
        f"error: {js} not found.\n"
        f"  Run: cd bindings/typescript && npm install"
    )


def collect_protos() -> list[Path]:
    return sorted(PROTO_ROOT.rglob("*.proto"))


def clear_generated() -> None:
    if OUT_ROOT.exists():
        shutil.rmtree(OUT_ROOT)
    OUT_ROOT.mkdir(parents=True, exist_ok=True)


def _protoc_cmd(protos: list[Path], *, plugin: Path | None) -> list[str]:
    # client_generic → usable with GrpcWebFetchTransport / RpcTransport.
    ts_opt = ",".join(
        [
            "long_type_string",
            "generate_dependencies",
            "client_generic",
            "server_none",
            "eslint_disable",
            "ts_nocheck",
        ]
    )
    cmd = [
        PROTOC,
        f"--proto_path={PROTO_ROOT}",
        f"--ts_out={ts_opt}:{OUT_ROOT}",
        *[str(p) for p in protos],
    ]
    if plugin is not None:
        cmd.insert(1, f"--plugin=protoc-gen-ts={plugin}")
    return cmd


def _run_protoc_windows(protos: list[Path], js_plugin: Path) -> None:
    """Invoke protoc on Windows without ``--plugin=`` absolute paths.

    protoc 35.1 on Windows uses CreateProcess EXACT_NAME for ``--plugin=name=path``,
    which cannot run npm's Node shebang shim (error: ``%1 is not a valid Win32
    application``). SEARCH_PATH mode runs ``cmd.exe /c protoc-gen-ts``, which
    resolves ``.cmd`` via PATHEXT.

    We do **not** put ``node_modules/.bin`` on PATH: that directory also has an
    extensionless ``protoc-gen-ts`` shim that can be preferred and fail the same
    way. Instead, expose a temp dir containing only a ``.cmd`` that calls
    ``node.exe`` with absolute paths.
    """
    node = shutil.which("node")
    if not node:
        raise SystemExit("error: node not found on PATH (required for protoc-gen-ts)")
    node = str(Path(node).resolve())
    js = str(js_plugin)

    with tempfile.TemporaryDirectory(prefix="protoc-gen-ts-") as tmp:
        wrap_dir = Path(tmp)
        (wrap_dir / "protoc-gen-ts.cmd").write_bytes(
            f'@echo off\r\n"{node}" "{js}" %*\r\n'.encode("ascii")
        )
        env = os.environ.copy()
        env["PATH"] = str(wrap_dir) + os.pathsep + env.get("PATH", "")
        subprocess.run(_protoc_cmd(protos, plugin=None), check=True, env=env)


def run_protoc(protos: list[Path], js_plugin: Path) -> None:
    if sys.platform == "win32":
        _run_protoc_windows(protos, js_plugin)
        return

    # Unix: shebang works with EXACT_NAME; pin the script path.
    subprocess.run(_protoc_cmd(protos, plugin=js_plugin), check=True)


def write_index() -> None:
    """Write marker + README for the generated tree."""
    lines = [
        "// Generated by scripts/generate_typescript_msgs.py — do not edit by hand.",
        "// Public import: robot-bus/<pkg>/msg/v1/<file>.js (maps to generated/)",
        "",
    ]
    for pkg in MSG_PACKAGES:
        pkg_dir = OUT_ROOT / pkg
        if pkg_dir.is_dir():
            lines.append(f"// package {pkg}")
    lines.append("")
    (OUT_ROOT / "README.md").write_text(
        "# Generated TypeScript protobuf / gRPC stubs\n\n"
        "Produced by `scripts/generate_typescript_msgs.py` (`just gen-typescript`).\n"
        "Do not edit by hand; re-run after changing `proto/`.\n\n"
        "Public imports (via `package.json` `exports`) omit this directory name:\n\n"
        "```ts\n"
        'import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js";\n'
        "// → ./generated/sensor_msgs/msg/v1/imu.js\n"
        "```\n",
        encoding="utf-8",
    )
    (OUT_ROOT / ".generated").write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    if not shutil.which(PROTOC) and not Path(PROTOC).is_file():
        print(f"error: protoc not found ({PROTOC!r})", file=sys.stderr)
        return 1

    ensure_protoc_version(PROTOC)
    js_plugin = find_protoc_gen_ts_js()
    protos = collect_protos()
    if not protos:
        print("error: no proto files found", file=sys.stderr)
        return 1

    clear_generated()
    print(f"{PROTOC} ({protoc_version(PROTOC)}) -> {len(protos)} files ...")
    run_protoc(protos, js_plugin)
    write_index()
    print(f"wrote TypeScript stubs under {OUT_ROOT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
