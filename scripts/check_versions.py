#!/usr/bin/env python3
"""Fail when package versions drift from the workspace version."""

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]


def workspace_version() -> str:
    text = (ROOT / "Cargo.toml").read_text()
    match = re.search(r'^version = "([^"]+)"', text, re.M)
    if not match:
        raise SystemExit("Cargo.toml has no workspace version")
    return match.group(1)


def main() -> None:
    version = workspace_version()
    errors = []
    changelog = (ROOT / "CHANGELOG.md").read_text()
    if f"## [{version}]" not in changelog:
        errors.append(f"CHANGELOG.md has no heading ## [{version}]")

    must_contain = {
        "bindings/python/pyproject.toml": f'version = "{version}"',
        "bindings/python/native/Cargo.toml": f'version = "{version}"',
        "bindings/cpp/native/Cargo.toml": f'version = "{version}"',
        "bindings/typescript/native/Cargo.toml": f'version = "{version}"',
        "bindings/typescript/package.json": f'"version": "{version}"',
        "console/package.json": f'"version": "{version}"',
        "bindings/java/pom.xml": f"<version>{version}</version>",
        "bindings/android/gradle.properties": f"VERSION_NAME={version}",
        "bindings/cpp/CMakeLists.txt": f"VERSION {version}",
        "benches/ros2_perf/src/ros2_perf/package.xml": f"<version>{version}</version>",
        "README.md": f'robot-bus = "{version}"',
        "README-zh.md": f'robot-bus = "{version}"',
        "crates/robot-bus/Cargo.toml": f'version = "{version}"',
        "crates/robot-bus-broker/Cargo.toml": f'version = "={version}"',
        "crates/robot-bus-ros2/Cargo.toml": f'version = "={version}"',
    }
    for rel, needle in must_contain.items():
        body = (ROOT / rel).read_text()
        if needle not in body:
            errors.append(f"{rel} does not contain {needle}")

    for rel in (
        "crates/robot-bus/Cargo.toml",
        "crates/robot-bus-broker/Cargo.toml",
        "crates/robot-bus-msgs/Cargo.toml",
        "crates/robot-bus-ros2/Cargo.toml",
    ):
        if "version.workspace = true" not in (ROOT / rel).read_text():
            errors.append(f"{rel} does not use version.workspace")

    if errors:
        print("version check failed:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        sys.exit(1)
    print(f"versions match {version}")


if __name__ == "__main__":
    main()
