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
        "crates/robot-bus-ros2/Cargo.toml": f'version = "={version}"',
    }
    for rel, needle in must_contain.items():
        body = (ROOT / rel).read_text()
        if needle not in body:
            errors.append(f"{rel} does not contain {needle}")

    for rel in (
        "crates/robot-bus/Cargo.toml",
        "crates/robot-bus-msgs/Cargo.toml",
        "crates/robot-bus-ros2/Cargo.toml",
    ):
        if "version.workspace = true" not in (ROOT / rel).read_text():
            errors.append(f"{rel} does not use version.workspace")

    versions = {}
    for raw in (ROOT / "scripts" / "protobuf-versions.env").read_text().splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        versions[key.strip()] = value.strip()
    python_floor = versions["PYTHON_PROTOBUF_FLOOR"]
    java_floor = versions["JAVA_PROTOBUF_FLOOR"]
    pyproject = (ROOT / "bindings/python/pyproject.toml").read_text()
    if f"protobuf>={python_floor},<9" not in pyproject:
        errors.append(
            f"bindings/python/pyproject.toml does not depend on protobuf>={python_floor},<9"
        )
    pom = (ROOT / "bindings/java/pom.xml").read_text()
    if f"<protobuf.version>{java_floor}</protobuf.version>" not in pom:
        errors.append(f"bindings/java/pom.xml protobuf.version is not {java_floor}")
    gradle = (ROOT / "bindings/android/build.gradle.kts").read_text()
    if f"protobuf-javalite:{java_floor}" not in gradle:
        errors.append(
            f"bindings/android/build.gradle.kts does not use protobuf-javalite:{java_floor}"
        )

    if errors:
        print("version check failed:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        sys.exit(1)
    print(f"versions match {version}")


if __name__ == "__main__":
    main()
