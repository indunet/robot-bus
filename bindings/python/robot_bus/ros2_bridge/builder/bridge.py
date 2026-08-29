"""Ros2Bridge runtime (rclpy wiring)."""

from __future__ import annotations

import threading
import time
from typing import Any, Optional

import robot_bus

from .config import (
    ACTION_CALL_TIMEOUT,
    CONSOLE_DETECT_TIMEOUT,
    SERVICE_CALL_TIMEOUT,
    ActionWireContext,
    Direction,
    Ros2BridgeBuilder,
    ServiceWireContext,
    TopicWireContext,
    should_enable_ros_subscription,
    _ros_qos,
    _ros_service_qos,
    _topic_supports_lazy,
    TOPIC_DEMAND,
    TOPICS_SNAPSHOT,
)

class Ros2Bridge:
    def __init__(self) -> None:
        self._bus: Any = None
        self._ros_node: Any = None
        self._executor: Any = None
        self._spin_thread: Optional[threading.Thread] = None
        self._halt = threading.Event()
        self._keep_alive: list[Any] = []
        self._lazy_routes: dict[str, dict[str, Any]] = {}
        self._eager_bus_topics: set[str] = set()
        self._subscriber_counts: dict[str, int] = {}
        self._console_live: Optional[bool] = None
        self._first_spin_at: Optional[float] = None
        self._callback_group: Any = None

    @staticmethod
    def new(name: str) -> Ros2BridgeBuilder:
        return Ros2BridgeBuilder(name)

    @classmethod
    def _from_builder(cls, builder: Ros2BridgeBuilder) -> "Ros2Bridge":
        try:
            import rclpy
            from rclpy.callback_groups import ReentrantCallbackGroup
            from rclpy.executors import MultiThreadedExecutor
        except ImportError as err:
            raise RuntimeError(
                "ROS 2 not available: source Humble/Jazzy and install rclpy"
            ) from err

        if not rclpy.ok():
            rclpy.init()

        self = cls()
        self._bus = builder._bus_factory(f"{builder._name}_bus")
        self._ros_node = rclpy.create_node(builder._name)
        self._callback_group = ReentrantCallbackGroup()
        self._executor = MultiThreadedExecutor()
        self._executor.add_node(self._ros_node)

        for route in builder._routes:
            self._wire_topic(route)
        for svc in builder._services:
            self._wire_service(svc)
        for act in builder._actions:
            self._wire_action(act)

        if self._lazy_routes:
            self._subscribe_demand()

        self._spin_thread = threading.Thread(
            target=self._ros_spin, name="ros2_bridge_spin", daemon=True
        )
        self._spin_thread.start()
        return self

    def _ros_spin(self) -> None:
        try:
            self._executor.spin()
        except Exception:  # noqa: BLE001
            pass

    def spin(self) -> None:
        while True:
            self.spin_once(None)

    def spin_once(self, timeout: Optional[float] = 0.01) -> None:
        if self._first_spin_at is None:
            self._first_spin_at = time.monotonic()
        try:
            self._bus.spin_once(timeout)
        except Exception as err:  # noqa: BLE001
            if "nothing registered" not in str(err):
                raise
        self._apply_lazy()

    def has_ros_subscription(self, bus_topic: str) -> bool:
        if bus_topic in self._lazy_routes:
            return self._lazy_routes[bus_topic]["sub"] is not None
        return bus_topic in self._eager_bus_topics

    def close(self) -> None:
        self._halt.set()
        if self._executor is not None:
            try:
                self._executor.shutdown()
            except Exception:  # noqa: BLE001
                pass
        if self._spin_thread is not None:
            self._spin_thread.join(timeout=2.0)
            self._spin_thread = None
        if self._executor is not None and self._ros_node is not None:
            try:
                self._executor.remove_node(self._ros_node)
            except Exception:  # noqa: BLE001
                pass
        if self._ros_node is not None:
            try:
                self._ros_node.destroy_node()
            except Exception:  # noqa: BLE001
                pass
            self._ros_node = None

    def __del__(self) -> None:  # pragma: no cover
        try:
            self.close()
        except Exception:
            pass

    def _subscribe_demand(self) -> None:
        def on_demand(payload: bytes) -> None:
            try:
                from robot_bus.robot_bus_interfaces.msg.v1 import TopicDemand
            except ImportError:
                return
            msg = TopicDemand()
            msg.ParseFromString(payload)
            self._console_live = True
            self._subscriber_counts[msg.topic] = int(msg.subscribers)

        def on_topics(payload: bytes) -> None:
            try:
                from robot_bus.robot_bus_interfaces.msg.v1 import TopicStatsList
            except ImportError:
                return
            msg = TopicStatsList()
            msg.ParseFromString(payload)
            self._console_live = True
            for row in msg.topics:
                self._subscriber_counts[row.name] = int(row.subscribers)

        self._keep_alive.append(self._bus.create_subscription(TOPIC_DEMAND, on_demand))
        self._keep_alive.append(self._bus.create_subscription(TOPICS_SNAPSHOT, on_topics))

    def _apply_lazy(self) -> None:
        if not self._lazy_routes:
            return
        if self._console_live is None and self._first_spin_at is not None:
            if time.monotonic() - self._first_spin_at >= CONSOLE_DETECT_TIMEOUT:
                self._console_live = False
        for bus_topic, route in self._lazy_routes.items():
            n = self._subscriber_counts.get(bus_topic, 0)
            want = should_enable_ros_subscription(True, self._console_live, n)
            sub = route["sub"]
            if want and sub is None:
                route["sub"] = route["create"]()
            elif not want and sub is not None:
                try:
                    self._ros_node.destroy_subscription(sub)
                except Exception:  # noqa: BLE001
                    pass
                route["sub"] = None

    def _wire_topic(self, route: dict[str, Any]) -> None:
        mapper = route["mapper"]
        ros_topic = route["ros_topic"]
        bus_topic = route["bus_topic"]
        direction = route["direction"]
        lazy = route["lazy"]

        if callable(getattr(mapper, "attach", None)) and not _topic_supports_lazy(mapper):
            ctx = TopicWireContext(
                self._ros_node,
                self._bus,
                ros_topic,
                bus_topic,
                direction,
                self._keep_alive,
                qos=_ros_qos(route["ros_qos"]),
                bus_qos_depth=route["bus_qos"].depth,
            )
            mapper.attach(ctx)
            if direction == Direction.Ros2ToBus:
                self._eager_bus_topics.add(bus_topic)
            return

        msg_type = mapper.ros_msg_type()
        ros_qos = _ros_qos(route["ros_qos"])
        bus_depth = route["bus_qos"].depth
        if direction == Direction.BusToRos2:
            ros_pub = self._ros_node.create_publisher(msg_type, ros_topic, ros_qos)

            def on_bus(payload: bytes, m=mapper, pub=ros_pub) -> None:
                try:
                    pub.publish(m.bus_to_ros(payload))
                except Exception:
                    pass

            sub_kw: dict[str, Any] = {"qos_depth": bus_depth}
            self._keep_alive.append(self._bus.create_subscription(bus_topic, on_bus, **sub_kw))
            self._keep_alive.append(ros_pub)
            return

        pub_kw: dict[str, Any] = {"qos_depth": bus_depth}
        bus_pub = self._bus.create_publisher(bus_topic, **pub_kw)
        lock = threading.Lock()

        def create_sub(
            m=mapper, t=msg_type, rt=ros_topic, pub=bus_pub, mtx=lock, rq=ros_qos
        ) -> Any:
            def on_ros(msg, pub=pub, mtx=mtx, m=m) -> None:
                try:
                    payload = m.ros_to_bus(msg)
                    with mtx:
                        pub.publish(payload)
                except Exception:
                    pass

            return self._ros_node.create_subscription(t, rt, on_ros, rq)

        if lazy:
            self._lazy_routes[bus_topic] = {"create": create_sub, "sub": None}
            self._keep_alive.append(bus_pub)
            return

        self._keep_alive.append(create_sub())
        self._keep_alive.append(bus_pub)
        self._eager_bus_topics.add(bus_topic)

    def _wire_service(self, route: dict[str, Any]) -> None:
        mapper = route["mapper"]
        if callable(getattr(mapper, "attach", None)) and not callable(
            getattr(mapper, "ros_srv_type", None)
        ):
            ctx = ServiceWireContext(
                self._ros_node,
                self._bus,
                route["ros_service"],
                route["bus_service"],
                route["direction"],
                route["timeout"],
                self._callback_group,
                self._keep_alive,
                _ros_service_qos(route["ros_qos"]),
                bus_qos_depth=route["bus_qos"].depth,
            )
            mapper.attach(ctx)
            return

        srv_type = mapper.ros_srv_type()
        timeout = route["timeout"]
        ros_qos = _ros_service_qos(route["ros_qos"])
        if route["direction"] == Direction.Ros2ToBus:
            bus_client = self._bus.create_client(
                route["bus_service"], qos_depth=route["bus_qos"].depth
            )
            lock = threading.Lock()

            def on_ros(request, response, m=mapper, client=bus_client, mtx=lock) -> Any:
                try:
                    req_bytes = m.ros_req_to_bus(request)
                    with mtx:
                        resp_bytes = client.call(req_bytes, timeout)
                    out = m.bus_resp_to_ros(resp_bytes)
                    _copy_msg(out, response)
                except Exception as err:  # noqa: BLE001
                    err_fn = getattr(m, "error_response", None)
                    if callable(err_fn):
                        _copy_msg(err_fn(f"bus call failed: {err}"), response)
                return response

            srv = self._ros_node.create_service(
                srv_type,
                route["ros_service"],
                on_ros,
                qos_profile=ros_qos,
                callback_group=self._callback_group,
            )
            self._keep_alive.extend((bus_client, srv, lock))
            return

        ros_client = self._ros_node.create_client(
            srv_type,
            route["ros_service"],
            qos_profile=ros_qos,
            callback_group=self._callback_group,
        )

        def on_bus(payload: bytes, m=mapper, client=ros_client) -> bytes:
            if not client.wait_for_service(timeout_sec=timeout):
                err_fn = getattr(m, "error_response", None)
                if callable(err_fn):
                    return m.ros_resp_to_bus(err_fn("timed out waiting for ROS service"))
                return b""
            req = m.bus_req_to_ros(payload)
            future = client.call_async(req)
            try:
                resp = future.result(timeout=timeout)
            except Exception:
                err_fn = getattr(m, "error_response", None)
                if callable(err_fn):
                    return m.ros_resp_to_bus(err_fn("timed out waiting for ROS response"))
                return b""
            return m.ros_resp_to_bus(resp)

        self._keep_alive.append(
            self._bus.create_service(
                route["bus_service"], on_bus, qos_depth=route["bus_qos"].depth
            )
        )
        self._keep_alive.append(ros_client)

    def _wire_action(self, route: dict[str, Any]) -> None:
        mapper = route["mapper"]
        if callable(getattr(mapper, "attach", None)) and not callable(
            getattr(mapper, "ros_action_type", None)
        ):
            ctx = ActionWireContext(
                self._ros_node,
                self._bus,
                route["ros_action"],
                route["bus_action"],
                route["direction"],
                route["timeout"],
                self._callback_group,
                self._keep_alive,
                route["ros_qos"],
                bus_qos_depth=route["bus_qos"].depth,
            )
            mapper.attach(ctx)
            return

        from rclpy.action import ActionClient, ActionServer, CancelResponse, GoalResponse

        act_type = mapper.ros_action_type()
        timeout = route["timeout"]
        srv_qos = _ros_service_qos(route["ros_qos"])
        fb_qos = _ros_qos(route["ros_qos"])
        if route["direction"] == Direction.Ros2ToBus:
            bus_client = self._bus.create_action_client(
                route["bus_action"], qos_depth=route["bus_qos"].depth
            )
            lock = threading.Lock()

            def execute_cb(goal_handle, m=mapper, client=bus_client, mtx=lock):
                goal = goal_handle.request
                try:
                    goal_bytes = m.ros_goal_to_bus(goal)

                    def on_fb(body: bytes, gh=goal_handle, mm=m) -> None:
                        try:
                            fb = mm.bus_feedback_to_ros(body)
                            gh.publish_feedback(fb)
                        except Exception:
                            pass

                    with mtx:
                        handle = client.send_goal(
                            goal_bytes, timeout=timeout, feedback_callback=on_fb
                        )
                    result_bytes = handle.result(timeout=timeout)
                    result = m.bus_result_to_ros(result_bytes)
                    goal_handle.succeed()
                    return result
                except Exception:
                    goal_handle.abort()
                    return act_type.Result()

            server = ActionServer(
                self._ros_node,
                act_type,
                route["ros_action"],
                execute_callback=execute_cb,
                goal_callback=lambda _g: GoalResponse.ACCEPT,
                cancel_callback=lambda _g: CancelResponse.ACCEPT,
                callback_group=self._callback_group,
                goal_service_qos_profile=srv_qos,
                result_service_qos_profile=srv_qos,
                cancel_service_qos_profile=srv_qos,
                feedback_pub_qos_profile=fb_qos,
            )
            self._keep_alive.extend((bus_client, server, lock))
            return

        ros_client = ActionClient(
            self._ros_node,
            act_type,
            route["ros_action"],
            callback_group=self._callback_group,
            goal_service_qos_profile=srv_qos,
            result_service_qos_profile=srv_qos,
            cancel_service_qos_profile=srv_qos,
            feedback_sub_qos_profile=fb_qos,
        )

        def on_bus(payload: bytes, m=mapper, client=ros_client) -> list:
            goal = m.bus_goal_to_ros(payload)
            if not client.wait_for_server(timeout_sec=timeout):
                return [("RESULT", m.ros_result_to_bus(act_type.Result()))]
            feedbacks: list[bytes] = []
            fb_lock = threading.Lock()

            def on_fb(fb_msg, mm=m) -> None:
                try:
                    raw = mm.ros_feedback_to_bus(fb_msg.feedback)
                    with fb_lock:
                        feedbacks.append(raw)
                except Exception:
                    pass

            send_future = client.send_goal_async(goal, feedback_callback=on_fb)
            try:
                goal_handle = send_future.result(timeout=timeout)
            except Exception:
                return [("RESULT", m.ros_result_to_bus(act_type.Result()))]
            if goal_handle is None:
                return [("RESULT", m.ros_result_to_bus(act_type.Result()))]
            result_future = goal_handle.get_result_async()
            try:
                wrapped = result_future.result(timeout=timeout)
            except Exception:
                return [("RESULT", m.ros_result_to_bus(act_type.Result()))]
            phases: list[tuple[str, bytes]] = []
            with fb_lock:
                for fb in feedbacks:
                    phases.append(("FEEDBACK", fb))
            result = wrapped.result if wrapped is not None else act_type.Result()
            phases.append(("RESULT", m.ros_result_to_bus(result)))
            return phases

        self._keep_alive.append(
            self._bus.create_action_server(
                route["bus_action"], on_bus, qos_depth=route["bus_qos"].depth
            )
        )
        self._keep_alive.append(ros_client)


def _copy_msg(src: Any, dest: Any) -> None:
    for slot in getattr(dest, "__slots__", ()):
        name = slot.lstrip("_")
        if hasattr(src, name):
            try:
                setattr(dest, name, getattr(src, name))
            except Exception:
                pass
    if hasattr(src, "get_fields_and_field_types"):
        for name in src.get_fields_and_field_types():
            if hasattr(dest, name):
                try:
                    setattr(dest, name, getattr(src, name))
                except Exception:
                    pass
