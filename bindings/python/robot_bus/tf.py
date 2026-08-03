"""Typed TF helpers over the native ``TfBuffer`` / ``TfListener`` (protobuf messages)."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Union

if TYPE_CHECKING:
    from robot_bus.geometry_msgs.msg.v1 import TransformStamped
    from robot_bus.tf2_msgs.msg.v1 import TFMessage

try:
    from robot_bus._native import TfBuffer as _NativeTfBuffer
    from robot_bus._native import TfListener as _NativeTfListener
except ImportError:  # pragma: no cover
    _NativeTfBuffer = None  # type: ignore[misc, assignment]
    _NativeTfListener = None  # type: ignore[misc, assignment]


def _tf_message_type():
    from robot_bus.tf2_msgs.msg.v1 import TFMessage

    return TFMessage


def _transform_stamped_type():
    from robot_bus.geometry_msgs.msg.v1 import TransformStamped

    return TransformStamped


class TfBuffer:
    """In-memory TF tree; accepts / returns protobuf message instances."""

    def __init__(self, _inner: Any = None) -> None:
        if _NativeTfBuffer is None:
            raise ImportError("robot_bus._native is not available (build with maturin)")
        self._inner = _inner if _inner is not None else _NativeTfBuffer()

    def clear(self) -> None:
        self._inner.clear()

    def set_transform_msg(self, msg: TFMessage, is_static: bool = False) -> None:
        TFMessage = _tf_message_type()
        if not isinstance(msg, TFMessage):
            raise TypeError(f"expected TFMessage, got {type(msg).__name__}")
        self._inner.set_transform_msg(msg.SerializeToString(), is_static)

    def lookup_transform(self, target: str, source: str) -> TransformStamped:
        TransformStamped = _transform_stamped_type()
        raw = self._inner.lookup_transform(target, source)
        out = TransformStamped()
        out.ParseFromString(raw)
        return out

    def can_transform(self, target: str, source: str) -> bool:
        return bool(self._inner.can_transform(target, source))

    def frames(self) -> list[str]:
        return list(self._inner.frames())


class TfListener:
    """Subscribe ``/tf`` + ``/tf_static`` (or custom topics) into a shared buffer."""

    def __init__(
        self,
        node: Any,
        tf_topic: str = "/tf",
        tf_static_topic: str = "/tf_static",
    ) -> None:
        if _NativeTfListener is None:
            raise ImportError("robot_bus._native is not available (build with maturin)")
        if tf_topic == "/tf" and tf_static_topic == "/tf_static":
            self._inner = _NativeTfListener.with_defaults(node)
        else:
            self._inner = _NativeTfListener(node, tf_topic, tf_static_topic)

    def buffer(self) -> TfBuffer:
        return TfBuffer(_inner=self._inner.buffer())


class TransformBroadcaster:
    """Thin helper over a typed or raw ``TFMessage`` publisher."""

    def __init__(self, publisher: Any) -> None:
        if publisher is None:
            raise TypeError("publisher is required")
        self._publisher = publisher

    def send(
        self,
        *transforms: Union[TFMessage, TransformStamped],
    ) -> None:
        TFMessage = _tf_message_type()
        TransformStamped = _transform_stamped_type()
        if len(transforms) == 1 and isinstance(transforms[0], TFMessage):
            msg = transforms[0]
        else:
            msg = TFMessage()
            for t in transforms:
                if not isinstance(t, TransformStamped):
                    raise TypeError(
                        f"expected TransformStamped, got {type(t).__name__}"
                    )
                msg.transforms.append(t)
        if hasattr(self._publisher, "publish"):
            try:
                self._publisher.publish(msg)
            except TypeError:
                self._publisher.publish(msg.SerializeToString())
        else:
            raise TypeError("publisher must provide publish()")


__all__ = ["TfBuffer", "TfListener", "TransformBroadcaster"]
