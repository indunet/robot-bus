"""Unit tests for pure-Python typed Node wrappers (no broker / extension needed)."""

from __future__ import annotations

from google.protobuf import wrappers_pb2

from robot_bus._typed import (
    TypedActionClient,
    TypedActionGoalHandle,
    TypedServiceClient,
    TypedTopicPublisher,
    _decode,
    install_typed_node_api,
)

BoolValue = wrappers_pb2.BoolValue
StringValue = wrappers_pb2.StringValue


class _FakePublisher:
    def __init__(self, topic: str = "/t") -> None:
        self.topic = topic
        self.last: bytes | None = None

    def publish(self, payload: bytes) -> None:
        self.last = bytes(payload)


class _FakeServiceClient:
    service_name = "echo"

    def call(self, body: bytes, timeout=None) -> bytes:
        req = BoolValue()
        req.ParseFromString(body)
        return StringValue(value=f"ok:{req.value}").SerializeToString()


class _FakeActionGoalHandle:
    goal_id = "goal-1"
    action_name = "typed-action"

    def result(self, timeout=None) -> bytes:
        return StringValue(value="done").SerializeToString()

    def cancel(self) -> None:
        self.cancelled = True


class _FakeActionClient:
    action_name = "typed-action"

    def send_goal(
        self, body, goal_id=None, timeout=None, feedback_callback=None
    ):
        goal = BoolValue()
        goal.ParseFromString(body)
        if feedback_callback is not None:
            feedback_callback(
                StringValue(value=f"step:{goal.value}").SerializeToString()
            )
        return _FakeActionGoalHandle()


class _RawNode:
    def create_publisher(self, topic: str, qos_depth=None):
        self.last_pub_qos = qos_depth
        return _FakePublisher(topic)

    def create_subscription(self, topic, callback, callback_group=None, qos_depth=None):
        self._sub_cb = callback
        self.last_sub_qos = qos_depth

    def create_service(self, service_name, handler, callback_group=None):
        self._svc_handler = handler

    def create_client(self, service_name: str):
        return _FakeServiceClient()

    def create_action_server(self, action_name, handler, callback_group=None):
        self._action_handler = handler

    def create_action_client(self, action_name: str):
        return object()


def test_typed_topic_publisher_roundtrip() -> None:
    inner = _FakePublisher("/imu")
    pub = TypedTopicPublisher(inner, BoolValue)
    pub.publish(BoolValue(value=True))
    decoded = _decode(BoolValue, inner.last or b"")
    assert decoded is not None
    assert decoded.value is True


def test_typed_topic_publisher_rejects_wrong_type() -> None:
    pub = TypedTopicPublisher(_FakePublisher(), BoolValue)
    try:
        pub.publish(StringValue(value="x"))  # type: ignore[arg-type]
    except TypeError:
        pass
    else:
        raise AssertionError("expected TypeError")


def test_typed_service_client_roundtrip() -> None:
    client = TypedServiceClient(_FakeServiceClient(), BoolValue, StringValue)
    resp = client.call(BoolValue(value=True), timeout=1.0)
    assert resp.value == "ok:True"


def test_typed_action_goal_handle_roundtrip() -> None:
    client = TypedActionClient(
        _FakeActionClient(), BoolValue, StringValue, StringValue
    )
    feedback: list[str] = []
    goal = client.send_goal(
        BoolValue(value=True),
        feedback_callback=lambda msg: feedback.append(msg.value),
    )
    assert isinstance(goal, TypedActionGoalHandle)
    assert goal.goal_id == "goal-1"
    assert feedback == ["step:True"]
    assert goal.result(timeout=1.0).value == "done"


def test_install_typed_node_api_publisher_and_subscription() -> None:
    install_typed_node_api(_RawNode)
    node = _RawNode()

    raw = node.create_publisher("/raw")
    assert isinstance(raw, _FakePublisher)
    assert node.last_pub_qos is None

    typed = node.create_publisher("/imu", BoolValue, qos_depth=10)
    assert isinstance(typed, TypedTopicPublisher)
    assert node.last_pub_qos == 10
    typed.publish(BoolValue(value=False))
    assert typed._inner.last == BoolValue(value=False).SerializeToString()

    got: list[tuple[str, bool]] = []
    node.create_subscription(
        "/imu",
        lambda topic, msg: got.append((topic, msg.value)),
        msg_type=BoolValue,
        qos_depth=5,
    )
    assert node.last_sub_qos == 5
    node._sub_cb("/imu", BoolValue(value=True).SerializeToString())
    assert got == [("/imu", True)]


def test_install_typed_node_api_service() -> None:
    install_typed_node_api(_RawNode)
    node = _RawNode()

    def handler(req: BoolValue) -> StringValue:
        return StringValue(value=f"echo:{req.value}")

    node.create_service(
        "set",
        handler,
        request_type=BoolValue,
        response_type=StringValue,
    )
    out = node._svc_handler(BoolValue(value=True).SerializeToString())
    resp = StringValue()
    resp.ParseFromString(out)
    assert resp.value == "echo:True"

    client = node.create_client(
        "set", request_type=BoolValue, response_type=StringValue
    )
    assert isinstance(client, TypedServiceClient)
    assert client.call(BoolValue(value=False)).value == "ok:False"


if __name__ == "__main__":
    test_typed_topic_publisher_roundtrip()
    test_typed_topic_publisher_rejects_wrong_type()
    test_typed_service_client_roundtrip()
    test_typed_action_goal_handle_roundtrip()
    test_install_typed_node_api_publisher_and_subscription()
    test_install_typed_node_api_service()
    print("python typed api smoke ok")
