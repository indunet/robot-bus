"""Ros2Bridge runtime (rclpy wiring)."""

from __future__ import annotations

import logging
import threading
import time
import uuid
from typing import Any, Callable, Optional

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
    BRIDGES,
    EVENTS,
    IDLE_GRACE_S,
    SNAPSHOT_INTERVAL_S,
    mapper_type_name,
    direction_label,
)
from .drop_stats import DropStats, RouteHealth, forward_bus_to_ros, forward_ros_to_bus, unix_ms


def _wait_ros_future(
    future: Any, timeout: float, on_wait: Optional[Callable[[], None]] = None
) -> Any:
    """Wait while the background ROS executor progresses an rclpy Future."""
    deadline = time.monotonic() + timeout
    completed = threading.Event()
    # rclpy result() does not wait and has no timeout argument.
    future.add_done_callback(lambda _future: completed.set())
    while not future.done():
        if future.cancelled():
            raise RuntimeError("ROS future cancelled")
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("timed out waiting for ROS future")
        if on_wait is not None:
            on_wait()
        completed.wait(min(0.05, remaining) if on_wait is not None else remaining)
    return future.result()


class _Deadline:
    """One monotonic budget, including discovery, locking and response waits."""
    def __init__(self, timeout: float):
        self.end = time.monotonic() + timeout

    def remaining(self) -> float:
        remaining = self.end - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("ROS bridge RPC deadline exceeded")
        return remaining


class _RpcFailure(Exception):
    def __init__(self, status: str, message: str):
        super().__init__(message)
        self.status = status


def _failure_status(err: Exception) -> str:
    if isinstance(err, _RpcFailure):
        return err.status
    message = str(err).lower()
    if isinstance(err, TimeoutError) or "timed out" in message or "timeout" in message:
        return "timeout"
    if "cancelled" in message or "canceled" in message:
        return "cancelled"
    if "action rejected" in message:
        return "rejected"
    if "action aborted" in message:
        return "aborted"
    return "failed"


def _rpc_error_body(status: str, message: str) -> bytes:
    prefix = {"timeout": "RPC_TIMEOUT", "cancelled": "CANCELLED",
              "rejected": "ACTION_REJECTED", "aborted": "ACTION_ABORTED"}.get(status, "RPC_FAILED")
    return prefix.encode() + b"\0" + message.encode("utf-8", errors="replace")


class Ros2Bridge:
    def __init__(self) -> None:
        self._bus: Any = None
        self._rpc_bus: Any = None
        self._bus_executor: Any = None
        self._executor_error: Optional[Exception] = None
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
        self._drop_stats = DropStats()
        self._console_routes: list[dict[str, Any]] = []
        self._bridge_id = ""
        self._bridge_name = ""
        self._bridges_pub: Any = None
        self._events_pub: Any = None
        self._last_snapshot_at: Optional[float] = None
        self._event_seq = 0

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
        self._rpc_bus = builder._bus_factory(f"{builder._name}_rpc")
        self._bus_executor = robot_bus.MultiThreadedExecutor(num_threads=4)
        self._bus_executor.add_node(self._rpc_bus)
        self._ros_node = rclpy.create_node(builder._name)
        self._callback_group = ReentrantCallbackGroup()
        self._executor = MultiThreadedExecutor()
        self._executor.add_node(self._ros_node)
        self._bridge_id = uuid.uuid4().hex
        self._bridge_name = builder._name

        for route in builder._routes:
            self._wire_topic(route)
        for svc in builder._services:
            self._wire_service(svc)
        for act in builder._actions:
            self._wire_action(act)

        self._log_route_table()
        self._bridges_pub = self._bus.create_publisher(BRIDGES)
        self._events_pub = self._bus.create_publisher(EVENTS)
        self._keep_alive.extend((self._bridges_pub, self._events_pub))

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
        except Exception as err:
            if not self._halt.is_set():
                self._executor_error = err
                logging.getLogger("robot_bus.ros2_bridge").exception("ROS executor failed")
        else:
            if not self._halt.is_set():
                self._executor_error = RuntimeError("ROS executor stopped unexpectedly")

    def _check_executor(self) -> None:
        if self._executor_error is not None:
            raise RuntimeError(f"ROS executor failed: {self._executor_error}") from self._executor_error

    def spin(self) -> None:
        while True:
            self.spin_once(None)

    def spin_once(self, timeout: Optional[float] = 0.01) -> None:
        self._check_executor()
        if self._first_spin_at is None:
            self._first_spin_at = time.monotonic()
        # Bound idle polling so executor failures and RPC replies remain observable.
        poll = min(timeout, 0.01) if timeout is not None and timeout >= 0 else 0.01
        if self._rpc_bus is not None:
            self._rpc_bus.spin_once(0.0)
        try:
            self._bus.spin_once(poll)
        except Exception as err:  # noqa: BLE001
            if "nothing registered" not in str(err):
                raise
        self._check_executor()
        self._apply_lazy()
        self._publish_observe()

    def has_ros_subscription(self, bus_topic: str) -> bool:
        if bus_topic in self._lazy_routes:
            return self._lazy_routes[bus_topic]["sub"] is not None
        return bus_topic in self._eager_bus_topics

    def drop_stats(self) -> dict[str, int]:
        return self._drop_stats.snapshot()

    def _log_route_table(self) -> None:
        log = logging.getLogger("robot_bus.ros2_bridge")
        lines = [f"ros2_bridge '{self._bridge_name}' routes:"]
        for row in self._console_routes:
            ty = row["type_name"] or "-"
            lazy = "  lazy" if row["lazy"] else ""
            source, target = (row["ros_name"], row["bus_name"]) if row["direction"] == "ros→bus" else (row["bus_name"], row["ros_name"])
            lines.append(
                f"  {row['kind']:<7} {row['direction']:<8} {source} → "
                f"{target}  {ty}  ros={row['ros_qos']}  bus={row['bus_qos']}{lazy}"
            )
        log.info("\n".join(lines))

    def _route_enabled(self, row: dict[str, Any]) -> bool:
        if not row["lazy"]:
            return True
        return self.has_ros_subscription(row["bus_name"])

    def _grace_elapsed(self) -> bool:
        if self._first_spin_at is None:
            return False
        return time.monotonic() - self._first_spin_at >= IDLE_GRACE_S

    def _publish_observe(self) -> None:
        now = time.monotonic()
        if self._last_snapshot_at is not None and now - self._last_snapshot_at < SNAPSHOT_INTERVAL_S:
            return
        self._last_snapshot_at = now
        if self._bridges_pub is None:
            return
        try:
            from robot_bus.robot_bus_interfaces.msg.v1 import (
                BridgeSnapshot,
                ConsoleEvent,
            )
        except ImportError:
            return
        grace = self._grace_elapsed()
        snap = BridgeSnapshot()
        snap.bridge_id = self._bridge_id
        snap.bridge_name = self._bridge_name
        for row in self._console_routes:
            enabled = self._route_enabled(row)
            health: RouteHealth = row["health"]
            idle = bool(row["watch_idle"] and health.is_idle(enabled, grace))
            proto = snap.routes.add()
            proto.kind = row["kind"]
            proto.direction = row["direction"]
            proto.ros_name = row["ros_name"]
            proto.bus_name = row["bus_name"]
            proto.type_name = row["type_name"]
            proto.ros_qos = row["ros_qos"]
            proto.bus_qos = row["bus_qos"]
            proto.lazy = row["lazy"]
            proto.enabled = enabled
            proto.rx = health.rx
            proto.tx = health.tx
            proto.convert_fail = health.convert_fail
            proto.decode_fail = health.decode_fail
            proto.publish_fail = health.publish_fail
            proto.last_rx_ms = health.last_rx_ms
            proto.idle = idle
            for key, value in health.rpc_snapshot().items():
                setattr(proto, key, value)
        try:
            self._bridges_pub.publish(snap.SerializeToString())
        except Exception:  # noqa: BLE001
            pass
        log = logging.getLogger("robot_bus.ros2_bridge")
        for row in self._console_routes:
            if not row["watch_idle"]:
                continue
            health = row["health"]
            enabled = self._route_enabled(row)
            if not health.take_idle_event(enabled, grace):
                continue
            msg = (
                f"no traffic on {row['direction']} {row['ros_name']} for "
                f"{int(IDLE_GRACE_S)}s; check source traffic, connection, direction and ROS QoS"
            )
            log.warning("ros2_bridge/%s: %s", self._bridge_name, msg)
            if self._events_pub is None:
                continue
            self._event_seq += 1
            ev = ConsoleEvent()
            ev.id = f"bridge-idle-{self._event_seq}"
            ev.ts = unix_ms()
            ev.level = "WARN"
            ev.source = f"ros2_bridge/{self._bridge_name}"
            ev.message = msg
            try:
                self._events_pub.publish(ev.SerializeToString())
            except Exception:  # noqa: BLE001
                pass

    def close(self) -> None:
        self._halt.set()
        if self._bus_executor is not None:
            self._bus_executor.shutdown()
            self._rpc_bus = None
            self._bus_executor = None
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
        health = RouteHealth()
        health.watch_stale = not route["ros_qos"].is_transient_local
        self._console_routes.append(
            {
                "kind": "topic",
                "direction": direction_label(direction),
                "ros_name": ros_topic,
                "bus_name": bus_topic,
                "type_name": mapper_type_name(mapper),
                "ros_qos": route["ros_qos"].console_label(),
                "bus_qos": route["bus_qos"].console_label(),
                "lazy": lazy,
                "watch_idle": True,
                "health": health,
            }
        )

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
                drop_stats=self._drop_stats,
                health=health,
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

            def on_bus(payload: bytes, m=mapper, pub=ros_pub, h=health) -> None:
                forward_bus_to_ros(
                    self._drop_stats, ros_topic, m.bus_to_ros, pub.publish, payload, h
                )

            sub_kw: dict[str, Any] = {"qos_depth": bus_depth}
            self._keep_alive.append(self._bus.create_subscription(bus_topic, on_bus, **sub_kw))
            self._keep_alive.append(ros_pub)
            return

        pub_kw: dict[str, Any] = {"qos_depth": bus_depth}
        bus_pub = self._bus.create_publisher(bus_topic, **pub_kw)
        lock = threading.Lock()

        def create_sub(
            m=mapper, t=msg_type, rt=ros_topic, pub=bus_pub, mtx=lock, rq=ros_qos, h=health
        ) -> Any:
            def on_ros(msg, pub=pub, mtx=mtx, m=m, h=h) -> None:
                def publish(payload: bytes) -> None:
                    with mtx:
                        pub.publish(payload)

                forward_ros_to_bus(self._drop_stats, rt, m.ros_to_bus, publish, msg, h)

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
        health = RouteHealth()
        self._console_routes.append(
            {
                "kind": "service",
                "direction": direction_label(route["direction"]),
                "ros_name": route["ros_service"],
                "bus_name": route["bus_service"],
                "type_name": mapper_type_name(mapper),
                "ros_qos": route["ros_qos"].console_label(),
                "bus_qos": route["bus_qos"].console_label(),
                "lazy": False,
                "watch_idle": False,
                "health": health,
            }
        )
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
                health=health,
            )
            mapper.attach(ctx)
            return

        srv_type = mapper.ros_srv_type()
        timeout = route["timeout"]
        ros_qos = _ros_service_qos(route["ros_qos"])
        if route["direction"] == Direction.Ros2ToBus:
            bus_client = (self._rpc_bus or self._bus).create_client(
                route["bus_service"], qos_depth=route["bus_qos"].depth
            )
            lock = threading.Lock()

            def on_ros(request, response, m=mapper, client=bus_client, mtx=lock) -> Any:
                health.rpc_start()
                deadline = _Deadline(timeout)
                try:
                    req_bytes = m.ros_req_to_bus(request)
                    if not mtx.acquire(timeout=deadline.remaining()):
                        raise TimeoutError("timed out waiting for bridge service lock")
                    try:
                        resp_bytes = client.call(req_bytes, deadline.remaining())
                    finally:
                        mtx.release()
                    out = m.bus_resp_to_ros(resp_bytes)
                    _copy_msg(out, response)
                    health.rpc_finish()
                except Exception as err:  # noqa: BLE001
                    health.rpc_finish(_failure_status(err), str(err))
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
            health.rpc_start()
            deadline = _Deadline(timeout)
            future = None
            try:
                if not client.wait_for_service(timeout_sec=deadline.remaining()):
                    raise TimeoutError("timed out waiting for ROS service")
                req = m.bus_req_to_ros(payload)
                future = client.call_async(req)
                resp = _wait_ros_future(future, deadline.remaining())
                out = m.ros_resp_to_bus(resp)
                health.rpc_finish()
                return out
            except Exception as err:
                if future is not None and not future.done():
                    client.remove_pending_request(future)
                status = _failure_status(err)
                health.rpc_finish(status, str(err))
                return _rpc_error_body(status, str(err))

        self._keep_alive.append(
            (self._rpc_bus or self._bus).create_service(
                route["bus_service"], on_bus, qos_depth=route["bus_qos"].depth
            )
        )
        self._keep_alive.append(ros_client)

    def _wire_action(self, route: dict[str, Any]) -> None:
        mapper = route["mapper"]
        health = RouteHealth()
        self._console_routes.append(
            {
                "kind": "action",
                "direction": direction_label(route["direction"]),
                "ros_name": route["ros_action"],
                "bus_name": route["bus_action"],
                "type_name": mapper_type_name(mapper),
                "ros_qos": route["ros_qos"].console_label(),
                "bus_qos": route["bus_qos"].console_label(),
                "lazy": False,
                "watch_idle": False,
                "health": health,
            }
        )
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
                health=health,
            )
            mapper.attach(ctx)
            return

        from rclpy.action import ActionClient, ActionServer, CancelResponse, GoalResponse

        act_type = mapper.ros_action_type()
        timeout = route["timeout"]
        srv_qos = _ros_service_qos(route["ros_qos"])
        fb_qos = _ros_qos(route["ros_qos"])
        if route["direction"] == Direction.Ros2ToBus:
            bus_client = (self._rpc_bus or self._bus).create_action_client(
                route["bus_action"], qos_depth=route["bus_qos"].depth
            )
            lock = threading.Lock()
            live_goals: dict[int, Any] = {}
            live_lock = threading.Lock()

            def cancel_cb(goal_handle, goals=live_goals, mtx=live_lock):
                with mtx:
                    handle = goals.get(id(goal_handle))
                if handle is not None:
                    try:
                        handle.cancel()
                    except Exception:
                        pass
                return CancelResponse.ACCEPT

            def execute_cb(
                goal_handle,
                m=mapper,
                client=bus_client,
                mtx=lock,
                goals=live_goals,
                goals_mtx=live_lock,
            ):
                goal = goal_handle.request
                health.rpc_start()
                deadline = _Deadline(timeout)
                handle = None
                try:
                    goal_bytes = m.ros_goal_to_bus(goal)

                    def on_fb(body: bytes, gh=goal_handle, mm=m) -> None:
                        try:
                            fb = mm.bus_feedback_to_ros(body)
                            gh.publish_feedback(fb)
                        except Exception:
                            pass

                    if not mtx.acquire(timeout=deadline.remaining()):
                        raise TimeoutError("timed out waiting for bridge action lock")
                    try:
                        handle = client.send_goal(
                            goal_bytes, timeout=deadline.remaining(), feedback_callback=on_fb
                        )
                    finally:
                        mtx.release()
                    with goals_mtx:
                        goals[id(goal_handle)] = handle
                    if goal_handle.is_cancel_requested:
                        handle.cancel()
                    result_bytes = handle.result(timeout=deadline.remaining())
                    result = m.bus_result_to_ros(result_bytes)
                    if goal_handle.is_cancel_requested:
                        goal_handle.canceled()
                    else:
                        goal_handle.succeed()
                    health.rpc_finish("cancelled" if goal_handle.is_cancel_requested else "succeeded")
                    return result
                except Exception as err:
                    status = _failure_status(err)
                    health.rpc_finish(status, str(err))
                    if goal_handle.is_cancel_requested or status == "cancelled":
                        goal_handle.canceled()
                    else:
                        goal_handle.abort()
                    return act_type.Result()
                finally:
                    with goals_mtx:
                        goals.pop(id(goal_handle), None)

            server = ActionServer(
                self._ros_node,
                act_type,
                route["ros_action"],
                execute_callback=execute_cb,
                goal_callback=lambda _g: GoalResponse.ACCEPT,
                cancel_callback=cancel_cb,
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

        def on_bus(payload: bytes, ctx, m=mapper, client=ros_client) -> bytes:
            health.rpc_start()
            goal_handle = None
            deadline = _Deadline(timeout)
            try:
                goal = m.bus_goal_to_ros(payload)
                if not client.wait_for_server(timeout_sec=deadline.remaining()):
                    raise TimeoutError("timed out waiting for ROS action server")

                def on_fb(fb_msg, mm=m) -> None:
                    try:
                        ctx.publish_feedback(mm.ros_feedback_to_bus(fb_msg.feedback))
                    except Exception as err:
                        health.record_convert_fail()
                        if health.should_log_warn():
                            logging.getLogger("robot_bus.ros2_bridge").warning("action feedback: %s", err)

                send_future = client.send_goal_async(goal, feedback_callback=on_fb)
                goal_handle = _wait_ros_future(send_future, deadline.remaining())
                if goal_handle is None or not goal_handle.accepted:
                    raise _RpcFailure("rejected", "ROS action rejected goal")
                result_future = goal_handle.get_result_async()
                cancel_sent = False

                def forward_cancel() -> None:
                    nonlocal cancel_sent
                    if ctx.cancel_requested() and not cancel_sent:
                        goal_handle.cancel_goal_async()
                        cancel_sent = True

                wrapped = _wait_ros_future(result_future, deadline.remaining(), forward_cancel)
                # action_msgs/GoalStatus: SUCCEEDED=4, CANCELED=5, ABORTED=6.
                status = getattr(wrapped, "status", None)
                if status != 4:
                    outcome = {5: "cancelled", 6: "aborted"}.get(status, "failed")
                    raise _RpcFailure(outcome, f"ROS action ended with status {status}")
                out = m.ros_result_to_bus(wrapped.result)
                health.rpc_finish()
                return out
            except Exception as err:
                status = _failure_status(err)
                if status == "timeout" and goal_handle is not None:
                    try:
                        goal_handle.cancel_goal_async()
                    except Exception:
                        pass
                health.rpc_finish(status, str(err))
                return _rpc_error_body(status, str(err))

        self._keep_alive.append(
            (self._rpc_bus or self._bus).create_action_server(
                route["bus_action"],
                on_bus,
                qos_depth=route["bus_qos"].depth,
                streaming=True,
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
