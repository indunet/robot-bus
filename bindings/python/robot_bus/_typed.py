"""Pure-Python typed wrappers over the raw-bytes Node API.

Rust uses generics (``create_publisher::<Imu>``); Python cannot map that through
PyO3, so this layer binds a protobuf message class at create time and
SerializeToString / ParseFromString around the native raw API.
"""

from __future__ import annotations

import logging
from typing import Any, Callable, Optional, Type, TypeVar

from google.protobuf.message import Message

_log = logging.getLogger("robot_bus")

MsgT = TypeVar("MsgT", bound=Message)


def _require_message_type(msg_type: Any, *, what: str) -> Type[Message]:
    if not isinstance(msg_type, type) or not issubclass(msg_type, Message):
        raise TypeError(
            f"{what} must be a google.protobuf.message.Message subclass, "
            f"got {msg_type!r}"
        )
    return msg_type


def _encode(msg: Message) -> bytes:
    if not isinstance(msg, Message):
        raise TypeError(
            f"expected a protobuf Message, got {type(msg).__name__}"
        )
    return msg.SerializeToString()


def _decode(msg_type: Type[MsgT], payload: bytes) -> Optional[MsgT]:
    try:
        msg = msg_type()
        msg.ParseFromString(payload)
        return msg  # type: ignore[return-value]
    except Exception as err:  # noqa: BLE001 — match Rust: skip bad payloads
        _log.warning("typed decode failed for %s: %s", msg_type.__name__, err)
        return None


class TypedTopicPublisher:
    """Publisher that accepts protobuf Message instances."""

    __slots__ = ("_inner", "_msg_type")

    def __init__(self, inner: Any, msg_type: Type[Message]) -> None:
        self._inner = inner
        self._msg_type = msg_type

    @property
    def topic(self) -> str:
        return self._inner.topic

    @property
    def msg_type(self) -> Type[Message]:
        return self._msg_type

    def publish(self, msg: Message) -> None:
        if not isinstance(msg, self._msg_type):
            raise TypeError(
                f"publisher for {self._msg_type.__name__} got "
                f"{type(msg).__name__}"
            )
        self._inner.publish(_encode(msg))

    def __repr__(self) -> str:
        return (
            f"TypedTopicPublisher(topic={self.topic!r}, "
            f"msg_type={self._msg_type.__name__})"
        )


class TypedServiceClient:
    """Service client that encodes requests and decodes responses."""

    __slots__ = ("_inner", "_request_type", "_response_type")

    def __init__(
        self,
        inner: Any,
        request_type: Type[Message],
        response_type: Type[Message],
    ) -> None:
        self._inner = inner
        self._request_type = request_type
        self._response_type = response_type

    @property
    def service_name(self) -> str:
        return self._inner.service_name

    @property
    def request_type(self) -> Type[Message]:
        return self._request_type

    @property
    def response_type(self) -> Type[Message]:
        return self._response_type

    def service_is_ready(self) -> bool:
        return bool(self._inner.service_is_ready())

    def wait_for_service(self, timeout: Optional[float] = None) -> bool:
        return bool(self._inner.wait_for_service(timeout))

    def call(
        self, request: Message, timeout: Optional[float] = None
    ) -> Message:
        if not isinstance(request, self._request_type):
            raise TypeError(
                f"client for {self._request_type.__name__} got "
                f"{type(request).__name__}"
            )
        raw = self._inner.call(_encode(request), timeout)
        reply = _decode(self._response_type, raw)
        if reply is None:
            raise ValueError(
                f"service {self.service_name!r} response decode failed"
            )
        return reply

    def __repr__(self) -> str:
        return (
            f"TypedServiceClient(service_name={self.service_name!r}, "
            f"request={self._request_type.__name__}, "
            f"response={self._response_type.__name__})"
        )


class TypedActionGoalHandle:
    """Live action goal handle that decodes the protobuf result."""

    __slots__ = ("_inner", "_result_type")

    def __init__(self, inner: Any, result_type: Type[Message]) -> None:
        self._inner = inner
        self._result_type = result_type

    @property
    def goal_id(self) -> str:
        return self._inner.goal_id

    @property
    def action_name(self) -> str:
        return self._inner.action_name

    def result(self, timeout: Optional[float] = None) -> Message:
        raw = self._inner.result(timeout)
        result = _decode(self._result_type, raw)
        if result is None:
            raise ValueError(
                f"action {self.action_name!r} result decode failed"
            )
        return result

    def cancel(self) -> None:
        self._inner.cancel()

    def __repr__(self) -> str:
        return (
            f"TypedActionGoalHandle(action_name={self.action_name!r}, "
            f"goal_id={self.goal_id!r})"
        )


class TypedActionClient:
    """Action client that encodes goals and returns a live typed goal handle."""

    __slots__ = ("_inner", "_goal_type", "_feedback_type", "_result_type")

    def __init__(
        self,
        inner: Any,
        goal_type: Type[Message],
        feedback_type: Type[Message],
        result_type: Type[Message],
    ) -> None:
        self._inner = inner
        self._goal_type = goal_type
        self._feedback_type = feedback_type
        self._result_type = result_type

    @property
    def action_name(self) -> str:
        return self._inner.action_name

    def action_server_is_ready(self) -> bool:
        return bool(self._inner.action_server_is_ready())

    def wait_for_action_server(self, timeout: Optional[float] = None) -> bool:
        return bool(self._inner.wait_for_action_server(timeout))

    @property
    def goal_type(self) -> Type[Message]:
        return self._goal_type

    @property
    def feedback_type(self) -> Type[Message]:
        return self._feedback_type

    @property
    def result_type(self) -> Type[Message]:
        return self._result_type

    def _decode_body(self, kind: str, body: bytes) -> Optional[Message]:
        if kind == "FEEDBACK":
            return _decode(self._feedback_type, body)
        if kind == "RESULT":
            return _decode(self._result_type, body)
        if kind == "GOAL":
            return _decode(self._goal_type, body)
        return body  # type: ignore[return-value]

    def send_goal(
        self,
        goal: Message,
        goal_id: Optional[str] = None,
        timeout: Optional[float] = None,
        feedback_callback: Optional[Callable[[Message], Any]] = None,
    ) -> TypedActionGoalHandle:
        if not isinstance(goal, self._goal_type):
            raise TypeError(
                f"action client for {self._goal_type.__name__} got "
                f"{type(goal).__name__}"
            )
        raw_callback = None
        if feedback_callback is not None:
            def raw_callback(payload: bytes) -> None:
                feedback = _decode(self._feedback_type, payload)
                if feedback is not None:
                    feedback_callback(feedback)

        handle = self._inner.send_goal(
            _encode(goal), goal_id, timeout, raw_callback
        )
        return TypedActionGoalHandle(handle, self._result_type)

    def send_goal_and_wait(
        self,
        goal: Message,
        goal_id: Optional[str] = None,
        timeout: Optional[float] = None,
    ) -> list:
        if not isinstance(goal, self._goal_type):
            raise TypeError(
                f"action client for {self._goal_type.__name__} got "
                f"{type(goal).__name__}"
            )
        messages = self._inner.send_goal_and_wait(
            _encode(goal), goal_id, timeout
        )
        out = []
        for msg in messages:
            kind = msg["kind"]
            decoded = self._decode_body(kind, msg["body"])
            if decoded is None and kind in ("FEEDBACK", "RESULT", "GOAL"):
                continue
            out.append(
                {
                    "kind": kind,
                    "body": decoded if decoded is not None else msg["body"],
                    "goal_id": msg["goal_id"],
                    "action_name": msg["action_name"],
                }
            )
        return out

    def __repr__(self) -> str:
        return (
            f"TypedActionClient(action_name={self.action_name!r}, "
            f"goal={self._goal_type.__name__})"
        )


def _pair_types(
    request_type: Any,
    response_type: Any,
    *,
    what: str,
) -> Optional[tuple[Type[Message], Type[Message]]]:
    if request_type is None and response_type is None:
        return None
    if request_type is None or response_type is None:
        raise TypeError(
            f"{what} requires both request_type and response_type, or neither"
        )
    return (
        _require_message_type(request_type, what="request_type"),
        _require_message_type(response_type, what="response_type"),
    )


def _action_types(
    goal_type: Any,
    feedback_type: Any,
    result_type: Any,
    *,
    what: str,
) -> Optional[tuple[Type[Message], Type[Message], Type[Message]]]:
    types = (goal_type, feedback_type, result_type)
    if all(t is None for t in types):
        return None
    if any(t is None for t in types):
        raise TypeError(
            f"{what} requires goal_type, feedback_type, and result_type "
            "together, or none of them"
        )
    return (
        _require_message_type(goal_type, what="goal_type"),
        _require_message_type(feedback_type, what="feedback_type"),
        _require_message_type(result_type, what="result_type"),
    )


def install_typed_node_api(Node: Any) -> None:
    """Patch ``Node`` create_* methods to accept optional protobuf types."""

    if getattr(Node, "_robot_bus_typed_api", False):
        return

    _raw_create_publisher = Node.create_publisher
    _raw_create_subscription = Node.create_subscription
    _raw_create_service = Node.create_service
    _raw_create_client = Node.create_client
    _raw_create_action_server = Node.create_action_server
    _raw_create_action_client = Node.create_action_client

    def create_publisher(self, topic: str, msg_type: Any = None, qos_depth: Any = None):
        # Only forward qos_depth when set so callers/fakes without the kwarg still work.
        if qos_depth is None:
            raw = _raw_create_publisher(self, topic)
        else:
            raw = _raw_create_publisher(self, topic, qos_depth=qos_depth)
        if msg_type is None:
            return raw
        cls = _require_message_type(msg_type, what="msg_type")
        return TypedTopicPublisher(raw, cls)

    def create_subscription(
        self,
        topic: str,
        callback: Callable[..., Any],
        callback_group: Any = None,
        msg_type: Any = None,
        qos_depth: Any = None,
    ):
        def _raw_sub(cb: Callable[..., Any]):
            if qos_depth is None:
                return _raw_create_subscription(self, topic, cb, callback_group)
            return _raw_create_subscription(
                self, topic, cb, callback_group, qos_depth=qos_depth
            )

        if msg_type is None:
            return _raw_sub(callback)
        cls = _require_message_type(msg_type, what="msg_type")

        def _wrapped(topic_name: str, payload: bytes) -> None:
            msg = _decode(cls, payload)
            if msg is None:
                return
            callback(topic_name, msg)

        return _raw_sub(_wrapped)

    def create_service(
        self,
        service_name: str,
        handler: Callable[..., Any],
        callback_group: Any = None,
        *,
        request_type: Any = None,
        response_type: Any = None,
    ):
        pair = _pair_types(
            request_type, response_type, what="create_service"
        )
        if pair is None:
            return _raw_create_service(
                self, service_name, handler, callback_group
            )
        req_t, resp_t = pair

        def _wrapped(body: bytes) -> bytes:
            req = _decode(req_t, body)
            if req is None:
                return b""
            resp = handler(req)
            if not isinstance(resp, resp_t):
                raise TypeError(
                    f"service handler must return {resp_t.__name__}, "
                    f"got {type(resp).__name__}"
                )
            return _encode(resp)

        return _raw_create_service(
            self, service_name, _wrapped, callback_group
        )

    def create_client(
        self,
        service_name: str,
        *,
        request_type: Any = None,
        response_type: Any = None,
    ):
        pair = _pair_types(
            request_type, response_type, what="create_client"
        )
        raw = _raw_create_client(self, service_name)
        if pair is None:
            return raw
        req_t, resp_t = pair
        return TypedServiceClient(raw, req_t, resp_t)

    def create_action_server(
        self,
        action_name: str,
        handler: Callable[..., Any],
        callback_group: Any = None,
        *,
        goal_type: Any = None,
        feedback_type: Any = None,
        result_type: Any = None,
    ):
        types = _action_types(
            goal_type,
            feedback_type,
            result_type,
            what="create_action_server",
        )
        if types is None:
            return _raw_create_action_server(
                self, action_name, handler, callback_group
            )
        goal_t, fb_t, res_t = types

        def _wrapped(payload: bytes):
            goal = _decode(goal_t, payload)
            if goal is None:
                return [("RESULT", b"")]
            replies = handler(goal)
            out = []
            for phase, body in replies:
                phase_u = phase.upper() if isinstance(phase, str) else phase
                if phase_u == "FEEDBACK":
                    if not isinstance(body, fb_t):
                        raise TypeError(
                            f"FEEDBACK must be {fb_t.__name__}, "
                            f"got {type(body).__name__}"
                        )
                elif phase_u == "RESULT":
                    if not isinstance(body, res_t):
                        raise TypeError(
                            f"RESULT must be {res_t.__name__}, "
                            f"got {type(body).__name__}"
                        )
                else:
                    if not isinstance(body, Message):
                        raise TypeError(
                            f"action reply body must be a Message, "
                            f"got {type(body).__name__}"
                        )
                out.append((phase, _encode(body)))
            return out

        return _raw_create_action_server(
            self, action_name, _wrapped, callback_group
        )

    def create_action_client(
        self,
        action_name: str,
        *,
        goal_type: Any = None,
        feedback_type: Any = None,
        result_type: Any = None,
    ):
        types = _action_types(
            goal_type,
            feedback_type,
            result_type,
            what="create_action_client",
        )
        raw = _raw_create_action_client(self, action_name)
        if types is None:
            return raw
        goal_t, fb_t, res_t = types
        return TypedActionClient(raw, goal_t, fb_t, res_t)

    Node.create_publisher = create_publisher
    Node.create_subscription = create_subscription
    Node.create_service = create_service
    Node.create_client = create_client
    Node.create_action_server = create_action_server
    Node.create_action_client = create_action_client
    Node._robot_bus_typed_api = True
