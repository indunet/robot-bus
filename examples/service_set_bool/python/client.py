"""Call /examples/set_bool once."""

from __future__ import annotations

import time

import robot_bus
from robot_bus.std_srvs.srv.v1 import SetBoolRequest, SetBoolResponse


def main() -> None:
    node = robot_bus.Node("examples_set_bool_client")
    client = node.create_client(
        "/examples/set_bool",
        request_type=SetBoolRequest,
        response_type=SetBoolResponse,
    )
    client.wait_for_service(timeout=5.0)
    time.sleep(0.2)
    reply = client.call(SetBoolRequest(data=True), timeout=5.0)
    print(f"success={reply.success} message={reply.message}")


if __name__ == "__main__":
    main()
