#!/usr/bin/env python3
"""Generate Rust prost/tonic stubs under crates/robot-bus-msgs/src/generated/<pkg>/{msg|srv|action}/v1/<stem>.rs.

Requires ``protoc`` on PATH (override with env ``PROTOC``). Outputs are
gitignored; CI and package workflows run this before build/publish so artifacts
ship inside the crate / wheel. End users who install from crates.io do not need
protoc.

Usage (from repo root)::

  python3 scripts/generate_rust_msgs.py
  # or: just gen-rust

Pin ``protoc`` to ``CPP_PROTOC`` in ``scripts/protobuf-versions.env``.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
from protobuf_versions import ensure_protoc_version, require  # noqa: E402

PROTOC = os.environ.get("PROTOC", "protoc")
EXPECTED_PROTOC_VERSION = require("CPP_PROTOC")


def main() -> None:
    ensure_protoc_version(PROTOC, EXPECTED_PROTOC_VERSION)
    # prost-build / tonic-prost-build invoke `protoc` from PATH / PROTOC.
    env = os.environ.copy()
    env["PROTOC"] = PROTOC
    cmd = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(ROOT / "scripts" / "gen_rust_msgs" / "Cargo.toml"),
        "--release",
    ]
    print("+", " ".join(cmd), flush=True)
    subprocess.run(cmd, check=True, cwd=ROOT, env=env)


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as e:
        sys.exit(e.returncode)
