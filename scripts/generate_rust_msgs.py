#!/usr/bin/env python3
"""Generate Rust prost/tonic stubs under src/msgs/generated and src/grpc/generated.

Requires ``protoc`` on PATH (override with env ``PROTOC``). Outputs are
gitignored; CI and package workflows run this before build/publish so artifacts
ship inside the crate / wheel. End users who install from crates.io do not need
protoc.

Usage (from repo root)::

  python3 scripts/generate_rust_msgs.py
  # or: just gen-rust

Pin ``protoc`` to ``EXPECTED_PROTOC_VERSION`` (same as CI).
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROTOC = os.environ.get("PROTOC", "protoc")
EXPECTED_PROTOC_VERSION = "35.1"
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


def main() -> None:
    ensure_protoc_version(PROTOC)
    # prost-build / tonic-prost-build invoke `protoc` from PATH / PROTOC.
    env = os.environ.copy()
    env["PROTOC"] = PROTOC
    cmd = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(ROOT / "tools" / "gen-msgs" / "Cargo.toml"),
        "--release",
    ]
    print("+", " ".join(cmd), flush=True)
    subprocess.run(cmd, check=True, cwd=ROOT, env=env)


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as e:
        sys.exit(e.returncode)
