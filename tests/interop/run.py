#!/usr/bin/env python3
"""Cross-language interop matrix (6 scenarios, diversified language pairs).

  1. Rust pub        → Python sub      (Imu)
  2. Python pub      → Java sub        (Imu)
  3. TypeScript pub  → Python sub      (Imu)
  4. C++ service     → Python client   (SetBool)
  5. Java service    → Rust client     (SetBool)
  6. Python action   → C++ client      (Fibonacci)

Requires: `just python-dev` plus built peers (see `just test-interop`).
Missing peers fail the run. Set ROBOT_BUS_INTEROP_ALLOW_SKIP=1 only for
local partial runs.
"""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
TOPIC = "/interop/imu"
SERVICE = "/interop/set_bool"
ACTION = "/interop/fibonacci"
EXPECT_Z = 0.42


def _ephemeral_binds() -> dict[str, object]:
    # Bind :0 so the OS assigns ports at broker start (avoids free_port TOCTOU).
    return {
        "message_xsub_bind": "tcp://127.0.0.1:0",
        "message_xpub_bind": "tcp://127.0.0.1:0",
        "service_frontend_bind": "tcp://127.0.0.1:0",
        "service_backend_bind": "tcp://127.0.0.1:0",
        "action_frontend_bind": "tcp://127.0.0.1:0",
        "action_backend_bind": "tcp://127.0.0.1:0",
        "api_listen": "127.0.0.1:0",
        "tcp_only": True,
        "no_console": True,
    }


def _peer_env(broker) -> dict[str, str]:
    env = os.environ.copy()
    env.update(
        {
            "ROBOT_BUS_MESSAGE_XSUB": broker.message_xsub_bind,
            "ROBOT_BUS_MESSAGE_XPUB": broker.message_xpub_bind,
            "ROBOT_BUS_SERVICE_FRONTEND": broker.service_frontend_bind,
            "ROBOT_BUS_SERVICE_BACKEND": broker.service_backend_bind,
            "ROBOT_BUS_ACTION_FRONTEND": broker.action_frontend_bind,
            "ROBOT_BUS_ACTION_BACKEND": broker.action_backend_bind,
        }
    )
    native = REPO / "bindings/cpp/native/target/release"
    cpp_build = REPO / "bindings/cpp/build"
    lib_dirs: list[str] = []
    if native.is_dir():
        env.setdefault("ROBOT_BUS_NATIVE_DIR", str(native))
        lib_dirs.append(str(native))
    if cpp_build.is_dir():
        lib_dirs.append(str(cpp_build))
    lib_dirs.append("/usr/local/lib")
    extra = os.pathsep.join(lib_dirs)
    for key in ("LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH"):
        prev = env.get(key, "")
        env[key] = extra if not prev else extra + os.pathsep + prev
    return env


def _node_kwargs(broker) -> dict[str, str]:
    return {
        "message_xsub": broker.message_xsub_bind,
        "message_xpub": broker.message_xpub_bind,
        "service_frontend": broker.service_frontend_bind,
        "service_backend": broker.service_backend_bind,
        "action_frontend": broker.action_frontend_bind,
        "action_backend": broker.action_backend_bind,
    }


def _rust_bin() -> Path | None:
    for p in (
        REPO / "target/debug/robot_bus_interop",
        REPO / "target/release/robot_bus_interop",
    ):
        if p.is_file():
            return p
    return None


def _cpp_bin() -> Path | None:
    p = REPO / "bindings/cpp/build/interop_peer"
    return p if p.is_file() else None


def _java_ready() -> bool:
    classes = REPO / "bindings/java/target/classes"
    test_classes = REPO / "bindings/java/target/test-classes"
    peer = (
        test_classes
        / "org/indunet/robot/bus/interop/InteropPeer.class"
    )
    return classes.is_dir() and peer.is_file()


def _java_classpath() -> str:
    java = REPO / "bindings/java"
    cp_file = java / "target/interop-classpath.txt"
    parts = [
        str(java / "target/classes"),
        str(java / "target/test-classes"),
        str(java / "generated"),
    ]
    if cp_file.is_file():
        parts.append(cp_file.read_text().strip())
    else:
        # Fallback: ask Maven (slow path).
        subprocess.run(
            [
                "mvn",
                "-q",
                "-f",
                str(java / "pom.xml"),
                "dependency:build-classpath",
                f"-Dmdep.outputFile={cp_file}",
            ],
            check=True,
            cwd=str(java),
        )
        parts.append(cp_file.read_text().strip())
    return os.pathsep.join(parts)


def _ts_ready() -> bool:
    dist_ok = (REPO / "bindings/typescript/dist/index.node.js").is_file()
    native_ok = any((REPO / "bindings/typescript").glob("robot-bus*.node"))
    return dist_ok and native_ok


def _spawn(cmd: list[str], env: dict[str, str], *, cwd: Path | None = None) -> subprocess.Popen:
    return subprocess.Popen(
        cmd,
        cwd=str(cwd or REPO),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )


def _wait_ready(proc: subprocess.Popen, label: str, timeout: float = 60.0) -> None:
    assert proc.stdout is not None
    deadline = time.time() + timeout
    buf: list[str] = []
    while time.time() < deadline:
        if proc.poll() is not None:
            rest = proc.stdout.read() or ""
            out = "".join(buf) + rest
            if "READY" in out and proc.returncode == 0:
                return
            raise RuntimeError(
                f"{label} exited before READY (code={proc.returncode}):\n{out}"
            )
        line = proc.stdout.readline()
        if not line:
            time.sleep(0.05)
            continue
        buf.append(line)
        if "READY" in line:
            return
    raise TimeoutError(f"{label} did not print READY within {timeout}s:\n{''.join(buf)}")


def _drain(proc: subprocess.Popen) -> str:
    if proc.stdout is None:
        return ""
    return proc.stdout.read() or ""


def _stop(proc: subprocess.Popen | None) -> None:
    if proc is None or proc.poll() is not None:
        return
    proc.send_signal(signal.SIGTERM)
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=5)


def _spawn_rust(role: str, env: dict[str, str]) -> subprocess.Popen:
    bin_ = _rust_bin()
    if not bin_:
        raise FileNotFoundError("robot_bus_interop missing (cargo build --bin robot_bus_interop)")
    env = dict(env)
    env["ROBOT_BUS_INTEROP_ROLE"] = role
    return _spawn([str(bin_)], env)


def _spawn_cpp(role: str, env: dict[str, str]) -> subprocess.Popen:
    bin_ = _cpp_bin()
    if not bin_:
        raise FileNotFoundError("interop_peer missing (build bindings/cpp)")
    env = dict(env)
    env["ROBOT_BUS_INTEROP_ROLE"] = role
    return _spawn([str(bin_)], env)


def _spawn_java(role: str, env: dict[str, str]) -> subprocess.Popen:
    if not _java_ready():
        raise FileNotFoundError("Java InteropPeer not compiled (mvn test-compile)")
    env = dict(env)
    env["ROBOT_BUS_INTEROP_ROLE"] = role
    cmd = [
        "java",
        f"-Drobot.bus.native.dir={env.get('ROBOT_BUS_NATIVE_DIR', '')}",
        "-cp",
        _java_classpath(),
        "org.indunet.robot.bus.interop.InteropPeer",
    ]
    return _spawn(cmd, env, cwd=REPO / "bindings/java")


def _spawn_ts_pub(env: dict[str, str]) -> subprocess.Popen:
    if not _ts_ready():
        raise FileNotFoundError("TypeScript native binding missing (just ts-dev)")
    env = dict(env)
    env["ROBOT_BUS_INTEROP_ROLE"] = "pub"
    tsx = REPO / "bindings/typescript/node_modules/.bin/tsx"
    script = REPO / "tests/interop/ts_pub.mjs"
    if tsx.is_file():
        cmd = [str(tsx), str(script)]
    else:
        cmd = ["node", "--import", "tsx", str(script)]
    return _spawn(cmd, env, cwd=REPO / "bindings/typescript")


# --- scenarios -----------------------------------------------------------------


def scenario_rust_pub_python_sub(robot_bus, broker) -> None:
    from robot_bus.sensor_msgs.msg.v1 import Imu

    got: list[Imu] = []
    node = robot_bus.Node("py_sub", **_node_kwargs(broker))
    node.create_subscription(TOPIC, lambda imu: got.append(imu), msg_type=Imu)
    node.start()
    time.sleep(0.3)
    proc = _spawn_rust("pub", _peer_env(broker))
    try:
        _wait_ready(proc, "rust pub")
        deadline = time.time() + 5.0
        while not got and time.time() < deadline:
            time.sleep(0.05)
        assert got, "python sub did not receive Imu"
        assert abs(got[0].angular_velocity.z - EXPECT_Z) < 1e-9
        assert proc.wait(timeout=10) == 0
    finally:
        _stop(proc)
        node.shutdown()
        node.stop()
        node.wait()


def scenario_python_pub_java_sub(robot_bus, broker) -> None:
    from robot_bus.geometry_msgs.msg.v1 import Vector3
    from robot_bus.sensor_msgs.msg.v1 import Imu

    proc = _spawn_java("sub", _peer_env(broker))
    try:
        _wait_ready(proc, "java sub")
        time.sleep(0.2)
        node = robot_bus.Node("py_pub", **_node_kwargs(broker))
        pub = node.create_publisher(TOPIC, Imu)
        for _ in range(5):
            pub.publish(Imu(angular_velocity=Vector3(x=0.0, y=0.0, z=EXPECT_Z)))
            time.sleep(0.05)
        assert proc.wait(timeout=10) == 0, _drain(proc)
        node.shutdown()
        node.stop()
        node.wait()
    finally:
        _stop(proc)


def scenario_ts_pub_python_sub(robot_bus, broker) -> None:
    from robot_bus.sensor_msgs.msg.v1 import Imu

    got: list[Imu] = []
    node = robot_bus.Node("py_sub_ts", **_node_kwargs(broker))
    node.create_subscription(TOPIC, lambda imu: got.append(imu), msg_type=Imu)
    node.start()
    time.sleep(0.3)
    proc = _spawn_ts_pub(_peer_env(broker))
    try:
        _wait_ready(proc, "ts pub")
        deadline = time.time() + 5.0
        while not got and time.time() < deadline:
            time.sleep(0.05)
        assert got, "python sub did not receive Imu from TS"
        assert abs(got[0].angular_velocity.z - EXPECT_Z) < 1e-9
        assert proc.wait(timeout=10) == 0, _drain(proc)
    finally:
        _stop(proc)
        node.shutdown()
        node.stop()
        node.wait()


def scenario_cpp_svc_python_client(robot_bus, broker) -> None:
    from robot_bus.std_srvs.srv.v1 import SetBoolRequest, SetBoolResponse

    proc = _spawn_cpp("svc-server", _peer_env(broker))
    try:
        _wait_ready(proc, "cpp svc-server")
        time.sleep(0.2)
        node = robot_bus.Node("py_svc_client", **_node_kwargs(broker))
        client = node.create_client(
            SERVICE,
            request_type=SetBoolRequest,
            response_type=SetBoolResponse,
        )
        resp = client.call(SetBoolRequest(data=True), timeout=5.0)
        assert resp.success
        assert resp.message == "set:true"
    finally:
        _stop(proc)


def scenario_java_svc_rust_client(robot_bus, broker) -> None:
    env = _peer_env(broker)
    server = _spawn_java("svc-server", env)
    try:
        _wait_ready(server, "java svc-server")
        client = _spawn_rust("svc-client", env)
        try:
            _wait_ready(client, "rust svc-client")
            assert client.wait(timeout=10) == 0, _drain(client)
        finally:
            _stop(client)
    finally:
        _stop(server)


def scenario_python_act_cpp_client(robot_bus, broker) -> None:
    from robot_bus.example_interfaces.action.v1 import (
        FibonacciFeedback,
        FibonacciGoal,
        FibonacciResult,
    )

    server = robot_bus.Node("py_act_server", **_node_kwargs(broker))

    def on_fib(goal: FibonacciGoal):
        order = max(int(goal.order), 0)
        seq: list[int] = []
        for i in range(order):
            if i < 2:
                seq.append(i)
            else:
                seq.append(seq[i - 1] + seq[i - 2])
        feedback = seq[:-1] if len(seq) > 1 else list(seq)
        return [
            ("FEEDBACK", FibonacciFeedback(sequence=feedback)),
            ("RESULT", FibonacciResult(sequence=seq)),
        ]

    server.create_action_server(
        ACTION,
        on_fib,
        goal_type=FibonacciGoal,
        feedback_type=FibonacciFeedback,
        result_type=FibonacciResult,
    )
    server.start()
    time.sleep(0.3)
    proc = _spawn_cpp("act-client", _peer_env(broker))
    try:
        _wait_ready(proc, "cpp act-client")
        assert proc.wait(timeout=15) == 0, _drain(proc)
    finally:
        _stop(proc)
        server.shutdown()
        server.stop()
        server.wait()


def _allow_skip() -> bool:
    return os.environ.get("ROBOT_BUS_INTEROP_ALLOW_SKIP", "").lower() in (
        "1",
        "true",
        "yes",
    )


def _missing_peers() -> list[str]:
    missing: list[str] = []
    if _rust_bin() is None:
        missing.append("robot_bus_interop (cargo build --bin robot_bus_interop)")
    if not _java_ready():
        missing.append("Java InteropPeer (mvn test-compile in bindings/java)")
    if not _ts_ready():
        missing.append("TypeScript native binding (just ts-dev)")
    if _cpp_bin() is None:
        missing.append("interop_peer (cmake --build bindings/cpp --target interop_peer)")
    return missing


def main() -> int:
    try:
        import robot_bus
    except ImportError as err:
        print(f"FAIL: native robot_bus not installed ({err})", file=sys.stderr)
        print("hint: just python-dev", file=sys.stderr)
        return 1

    if robot_bus.RobotBusBroker is None or not hasattr(robot_bus, "RobotBusBroker"):
        print("FAIL: RobotBusBroker missing (just python-dev)", file=sys.stderr)
        return 1

    tests: list[tuple[str, str, object]] = [
        ("1 rust→python pub-sub", "rust", scenario_rust_pub_python_sub),
        ("2 python→java pub-sub", "java", scenario_python_pub_java_sub),
        ("3 typescript→python pub-sub", "ts", scenario_ts_pub_python_sub),
        ("4 cpp→python service", "cpp", scenario_cpp_svc_python_client),
        ("5 java→rust service", "java+rust", scenario_java_svc_rust_client),
        ("6 python→cpp action", "cpp", scenario_python_act_cpp_client),
    ]

    missing = _missing_peers()
    if missing and not _allow_skip():
        print("FAIL: required interop peers missing:", file=sys.stderr)
        for item in missing:
            print(f"  - {item}", file=sys.stderr)
        print("hint: just test-interop  (or set ROBOT_BUS_INTEROP_ALLOW_SKIP=1)", file=sys.stderr)
        return 1

    def available(need: str) -> bool:
        if need == "rust":
            return _rust_bin() is not None
        if need == "java":
            return _java_ready()
        if need == "java+rust":
            return _java_ready() and _rust_bin() is not None
        if need == "ts":
            return _ts_ready()
        if need == "cpp":
            return _cpp_bin() is not None
        return False

    failed = 0
    skipped = 0
    passed = 0
    for name, need, fn in tests:
        print(f"== {name} ==")
        if not available(need):
            print(f"skip: missing peer for {need}")
            skipped += 1
            continue
        try:
            with robot_bus.RobotBusBroker.start(**_ephemeral_binds()) as broker:
                fn(robot_bus, broker)
            print(f"ok: {name}")
            passed += 1
        except Exception as err:  # noqa: BLE001
            failed += 1
            print(f"FAIL: {name}: {err}", file=sys.stderr)

    print(
        f"interop summary: {passed} passed, {failed} failed, {skipped} skipped "
        f"(of {len(tests)})"
    )
    if skipped and not _allow_skip():
        return 1
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
