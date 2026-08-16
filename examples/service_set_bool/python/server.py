"""Service server for /examples/set_bool (std_srvs SetBool)."""

from __future__ import annotations

import robot_bus
from robot_bus.std_srvs.srv.v1 import SetBoolRequest, SetBoolResponse


def on_set_bool(req: SetBoolRequest) -> SetBoolResponse:
    return SetBoolResponse(success=True, message=f"set:{req.data}")


def main() -> None:
    node = robot_bus.Node("examples_set_bool_server")
    node.create_service(
        "/examples/set_bool",
        on_set_bool,
        request_type=SetBoolRequest,
        response_type=SetBoolResponse,
    )
    print("serving /examples/set_bool (Ctrl+C to stop)")
    try:
        node.spin()
    except KeyboardInterrupt:
        node.shutdown()


if __name__ == "__main__":
    main()
