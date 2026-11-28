"""Read ``scripts/protobuf-versions.env`` and check ``protoc`` against it."""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSIONS_FILE = Path(__file__).resolve().parent / "protobuf-versions.env"
PROTOC_VERSION_RE = re.compile(r"libprotoc\s+(\d+\.\d+(?:\.\d+)?)")


def load_versions() -> dict[str, str]:
    values: dict[str, str] = {}
    for raw in VERSIONS_FILE.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip().strip('"').strip("'")
    return values


def require(key: str) -> str:
    try:
        return load_versions()[key]
    except KeyError as exc:
        raise SystemExit(f"{VERSIONS_FILE.name} is missing {key}") from exc


def protoc_version(protoc: str) -> str:
    out = subprocess.check_output([protoc, "--version"], text=True).strip()
    match = PROTOC_VERSION_RE.search(out)
    if not match:
        raise RuntimeError(f"could not parse protoc version from {out!r}")
    return match.group(1)


def ensure_protoc_version(
    protoc: str,
    expected: str,
    *,
    warn_if_protoc_env: bool = False,
) -> None:
    """Exit when ``protoc`` is not ``expected``.

    ``warn_if_protoc_env`` is for the C++ generator: an explicit ``PROTOC``
    (a source build using the protobuf already on the machine) warns and
    continues. The default ``protoc`` on ``PATH`` still has to match.
    """
    if os.environ.get("ROBOT_BUS_SKIP_PROTOC_VERSION_CHECK") == "1":
        return
    version = protoc_version(protoc)
    if version == expected:
        return
    message = (
        f"protoc {version} != required {expected}\n"
        f"  Install https://github.com/protocolbuffers/protobuf/releases/tag/v{expected}\n"
        f"  or set PROTOC to that binary. Skip with ROBOT_BUS_SKIP_PROTOC_VERSION_CHECK=1."
    )
    if warn_if_protoc_env and os.environ.get("PROTOC"):
        print(f"warning: {message}", file=sys.stderr)
        return
    raise SystemExit(f"error: {message}")
