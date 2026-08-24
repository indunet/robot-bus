"""Builtin: `std_srvs/srv/SetBool`."""

from __future__ import annotations


class SetBoolServiceMapper:
    def type_name(self) -> str:
        return "std_srvs/srv/SetBool"

    def ros_srv_type(self):
        from std_srvs.srv import SetBool

        return SetBool

    def ros_req_to_bus(self, req) -> bytes:
        from robot_bus.std_srvs.srv.v1 import SetBoolRequest

        return SetBoolRequest(data=bool(req.data)).SerializeToString()

    def bus_req_to_ros(self, payload: bytes):
        from robot_bus.std_srvs.srv.v1 import SetBoolRequest
        from std_srvs.srv import SetBool

        bus = SetBoolRequest()
        bus.ParseFromString(payload)
        out = SetBool.Request()
        out.data = bus.data
        return out

    def ros_resp_to_bus(self, resp) -> bytes:
        from robot_bus.std_srvs.srv.v1 import SetBoolResponse

        return SetBoolResponse(success=bool(resp.success), message=str(resp.message)).SerializeToString()

    def bus_resp_to_ros(self, payload: bytes):
        from robot_bus.std_srvs.srv.v1 import SetBoolResponse
        from std_srvs.srv import SetBool

        bus = SetBoolResponse()
        bus.ParseFromString(payload)
        out = SetBool.Response()
        out.success = bus.success
        out.message = bus.message
        return out

    def error_response(self, message: str):
        from std_srvs.srv import SetBool

        out = SetBool.Response()
        out.success = False
        out.message = message
        return out
