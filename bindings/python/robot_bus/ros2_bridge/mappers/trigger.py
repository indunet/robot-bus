"""Builtin: `std_srvs/srv/Trigger`."""

from __future__ import annotations


class TriggerServiceMapper:
    def type_name(self) -> str:
        return "std_srvs/srv/Trigger"

    def ros_srv_type(self):
        from std_srvs.srv import Trigger

        return Trigger

    def ros_req_to_bus(self, _req) -> bytes:
        from robot_bus.std_srvs.srv.v1 import TriggerRequest

        return TriggerRequest().SerializeToString()

    def bus_req_to_ros(self, _payload: bytes):
        from std_srvs.srv import Trigger

        return Trigger.Request()

    def ros_resp_to_bus(self, resp) -> bytes:
        from robot_bus.std_srvs.srv.v1 import TriggerResponse

        return TriggerResponse(success=bool(resp.success), message=str(resp.message)).SerializeToString()

    def bus_resp_to_ros(self, payload: bytes):
        from robot_bus.std_srvs.srv.v1 import TriggerResponse
        from std_srvs.srv import Trigger

        bus = TriggerResponse()
        bus.ParseFromString(payload)
        out = Trigger.Response()
        out.success = bus.success
        out.message = bus.message
        return out

    def error_response(self, message: str):
        from std_srvs.srv import Trigger

        out = Trigger.Response()
        out.success = False
        out.message = message
        return out
