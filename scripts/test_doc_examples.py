#!/usr/bin/env python3
"""Run the actual marked README/API Python blocks against an isolated broker.

Requires a built native Python SDK (just python-dev). No hardcoded ports,
external brokers, altered snippet code, or mock API. Child timeouts include shutdown.
"""

import os
from pathlib import Path
import re
import socket
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
DOCUMENTS = ("README.md", "README-zh.md", "docs/en/python-api.md", "docs/zh/python-api.md")
EXPECTED = {
    "topic": "acceleration.z: 9.8",
    "service": "service: True, set:True",
    "action": "result: [0, 1, 1, 2, 3]",
}
BLOCK = re.compile(r"<!-- runnable: (\w+) -->\s*```python\n(.*?)\n```", re.S)


def run_blocks():
    count = 0
    with tempfile.TemporaryDirectory(prefix="robot-bus-docs-") as directory:
        for document in DOCUMENTS:
            blocks = BLOCK.findall((ROOT / document).read_text())
            kinds = [kind for kind, _ in blocks]
            if sorted(kinds) != sorted(EXPECTED):
                raise AssertionError(f"{document}: expected exactly one topic, service and action block; got {kinds}")
            for kind, source in blocks:
                snippet = Path(directory) / "demo.py"
                snippet.write_text(source + "\n")
                # Fresh broker per example: departed service/action workers
                # remain discoverable until their heartbeat expires.
                with start_broker() as broker:
                    env = dict(os.environ, ROBOT_BUS_API_URL=f"http://{broker.api_listen}", PYTHONUNBUFFERED="1")
                    result = subprocess.run(
                        [sys.executable, str(snippet)], cwd=directory, env=env,
                        capture_output=True, text=True, timeout=40,
                    )
                if result.returncode or EXPECTED[kind] not in result.stdout:
                    raise AssertionError(
                        f"{document} ({kind}) failed ({result.returncode})\n"
                        f"{result.stdout}\n{result.stderr}"
                    )
                print(f"OK: {document} ({kind})", flush=True)
                count += 1
    print(f"OK: {count} actual documentation examples", flush=True)


def start_broker():
    import robot_bus

    # Data sockets use :0. The console's discovery snapshot currently retains
    # API port 0 when the broker itself chooses that port. Select a concrete
    # loopback API port first, as documented, and retry if another process wins
    # the close/bind race. Never reuse or stop an existing broker.
    binds = {name: "tcp://127.0.0.1:0" for name in (
        "message_xsub_bind", "message_xpub_bind", "service_frontend_bind",
        "service_backend_bind", "action_frontend_bind", "action_backend_bind",
    )}
    for attempt in range(5):
        with socket.socket() as reservation:
            reservation.bind(("127.0.0.1", 0))
            port = reservation.getsockname()[1]
        try:
            return robot_bus.RobotBusBroker.start(
                **binds, api_listen=f"127.0.0.1:{port}", advertise_host="127.0.0.1",
                tcp_only=True, no_tank=True,
            )
        except RuntimeError as error:
            if "bind API listen" not in str(error) or attempt == 4:
                raise


def main():
    import robot_bus

    version = re.search(r'^version = "([^"]+)"', (ROOT / "Cargo.toml").read_text(), re.M)[1]
    if robot_bus.__version__ != version:
        raise RuntimeError(f"Installed SDK {robot_bus.__version__} != checkout {version}; run just python-dev")
    run_blocks()


if __name__ == "__main__":
    main()
