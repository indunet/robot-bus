"""ROS RPC wait regressions, runnable without ROS or the native extension."""

from concurrent.futures import Future
from contextlib import ExitStack
from types import ModuleType, SimpleNamespace
import threading
import unittest
from unittest.mock import Mock, patch

from robot_bus.ros2_bridge import Direction, TopicQos
from robot_bus.ros2_bridge.builder import bridge as runtime


class RosFuture(Future):
    """Match rclpy's argument-free, non-waiting result() API."""

    def done(self):
        # rclpy tracks CANCELLED separately from FINISHED.
        return super().done() and not self.cancelled()

    def result(self):
        if not self.done():
            raise AssertionError("result() called before completion")
        return super().result(timeout=0)


def ready(value):
    future = RosFuture()
    future.set_result(value)
    return future


class RpcWaitTests(unittest.TestCase):
    def setUp(self):
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        for name in ("_ros_qos", "_ros_service_qos"):
            self.stack.enter_context(patch.object(runtime, name, side_effect=lambda q: q))
        self.bridge = runtime.Ros2Bridge()
        self.bridge._bus = Mock()
        self.bridge._ros_node = Mock()
        self.addCleanup(self.bridge.close)
        self.mapper = SimpleNamespace(
            type_name=lambda: "test/Rpc",
            ros_srv_type=lambda: object,
            bus_req_to_ros=lambda payload: payload,
            ros_resp_to_bus=lambda response: response,
            error_response=lambda message: message.encode(),
            ros_action_type=lambda: SimpleNamespace(Result=lambda: b"default-result"),
            bus_goal_to_ros=lambda payload: payload,
            ros_result_to_bus=lambda result: result,
            ros_feedback_to_bus=lambda feedback: feedback,
        )

    def route(self, timeout):
        return dict(
            mapper=self.mapper,
            direction=Direction.BusToRos2,
            ros_service="/service",
            bus_service="/service",
            ros_action="/action",
            bus_action="/action",
            ros_qos=TopicQos.default(),
            bus_qos=TopicQos.bus(),
            timeout=timeout,
        )

    def service_callback(self, future, timeout=0.5):
        client = self.bridge._ros_node.create_client.return_value
        client.wait_for_service.return_value = True
        client.call_async.return_value = future
        self.bridge._wire_service(self.route(timeout))
        return self.bridge._bus.create_service.call_args.args[1]

    def action_callback(self, goal_future, timeout=0.5):
        action = ModuleType("rclpy.action")
        action.ActionClient = Mock()
        action.ActionServer = Mock()
        action.CancelResponse = Mock()
        action.GoalResponse = Mock()
        self.stack.enter_context(patch.dict("sys.modules", {"rclpy.action": action}))
        client = action.ActionClient.return_value
        client.wait_for_server.return_value = True
        client.send_goal_async.return_value = goal_future
        self.bridge._wire_action(self.route(timeout))
        return self.bridge._bus.create_action_server.call_args.args[1], client

    def complete_later(self, future, value):
        timer = threading.Timer(0.02, future.set_result, args=(value,))
        timer.start()
        self.addCleanup(timer.join)

    def test_wait_ready_with_zero_timeout(self):
        self.assertEqual(runtime._wait_ros_future(ready(b"ready"), 0), b"ready")

    def test_wait_pending_future_times_out(self):
        with self.assertRaises(TimeoutError):
            runtime._wait_ros_future(RosFuture(), 0.01)

    def test_wait_propagates_future_exception(self):
        future = RosFuture()
        future.set_exception(ValueError("ROS response failed"))
        with self.assertRaisesRegex(ValueError, "ROS response failed"):
            runtime._wait_ros_future(future, 0.5)

    def test_wait_cancelled_future(self):
        future = RosFuture()
        future.cancel()
        with self.assertRaisesRegex(RuntimeError, "ROS future cancelled"):
            runtime._wait_ros_future(future, 0.5)

    def test_service_already_ready(self):
        callback = self.service_callback(ready(b"response"))
        self.assertEqual(callback(b"request"), b"response")

    def test_service_waits_for_response(self):
        future = RosFuture()
        callback = self.service_callback(future)
        self.complete_later(future, b"response")
        self.assertEqual(callback(b"request"), b"response")

    def test_service_timeout(self):
        future = RosFuture()
        callback = self.service_callback(future, timeout=0.01)
        self.assertTrue(callback(b"request").startswith(b"RPC_TIMEOUT\0"))
        stats = self.bridge._console_routes[-1]["health"].rpc_snapshot()
        self.assertEqual((stats["calls"], stats["failures"], stats["timeouts"]), (1, 1, 1))
        self.assertFalse(future.done())

    def test_action_already_ready(self):
        handle = Mock()
        handle.get_result_async.return_value = ready(SimpleNamespace(status=4, result=b"result"))
        callback, client = self.action_callback(ready(handle))
        ctx = Mock()
        ctx.cancel_requested.return_value = False
        self.assertEqual(callback(b"goal", ctx), b"result")
        client.send_goal_async.assert_called_once()

    def test_action_waits_for_goal_acceptance_and_result(self):
        goal_future = RosFuture()
        result_future = RosFuture()
        handle = Mock()

        def get_result():
            self.complete_later(result_future, SimpleNamespace(status=4, result=b"result"))
            return result_future

        handle.get_result_async.side_effect = get_result
        callback, client = self.action_callback(goal_future)
        self.complete_later(goal_future, handle)
        ctx = Mock()
        ctx.cancel_requested.return_value = False
        self.assertEqual(callback(b"goal", ctx), b"result")
        feedback = client.send_goal_async.call_args.kwargs["feedback_callback"]
        feedback(SimpleNamespace(feedback=b"feedback"))
        ctx.publish_feedback.assert_called_once_with(b"feedback")

    def test_action_goal_acceptance_timeout(self):
        callback, _ = self.action_callback(RosFuture(), timeout=0.01)
        self.assertTrue(callback(b"goal", Mock()).startswith(b"RPC_TIMEOUT\0"))

    def test_action_result_timeout(self):
        handle = Mock()
        handle.get_result_async.return_value = RosFuture()
        callback, _ = self.action_callback(ready(handle), timeout=0.01)
        ctx = Mock()
        ctx.cancel_requested.return_value = False
        self.assertTrue(callback(b"goal", ctx).startswith(b"RPC_TIMEOUT\0"))
        handle.cancel_goal_async.assert_called_once()

    def test_action_forwards_cancel_while_waiting(self):
        result_future = RosFuture()
        handle = Mock()
        handle.get_result_async.return_value = result_future
        handle.cancel_goal_async.side_effect = lambda: result_future.set_result(
            SimpleNamespace(status=5, result=b"cancelled-result")
        )
        callback, _ = self.action_callback(ready(handle))
        ctx = Mock()
        ctx.cancel_requested.return_value = True
        self.assertTrue(callback(b"goal", ctx).startswith(b"CANCELLED\0"))
        handle.cancel_goal_async.assert_called_once()

    def test_action_rejected_without_requesting_result(self):
        handle = Mock(accepted=False)
        callback, _ = self.action_callback(ready(handle))
        self.assertTrue(callback(b"goal", Mock()).startswith(b"ACTION_REJECTED\0"))
        handle.get_result_async.assert_not_called()
        stats = self.bridge._console_routes[-1]["health"].rpc_snapshot()
        self.assertEqual((stats["calls"], stats["rejected"], stats["failures"]), (1, 1, 1))

    def test_action_terminal_statuses_are_not_success_payloads(self):
        for status, prefix in [(5, b"CANCELLED\0"), (6, b"ACTION_ABORTED\0"), (0, b"RPC_FAILED\0")]:
            with self.subTest(status=status):
                handle = Mock(accepted=True)
                handle.get_result_async.return_value = ready(SimpleNamespace(status=status, result=b"payload"))
                callback, _ = self.action_callback(ready(handle))
                self.assertTrue(callback(b"goal", Mock()).startswith(prefix))
                health = self.bridge._console_routes[-1]["health"]
                self.assertEqual(health.tx, 0)
                self.assertEqual(health.calls, 1)

    def test_bus_cancelled_error_is_not_a_generic_failure(self):
        self.assertEqual(runtime._failure_status(RuntimeError("cancelled 'navigate'")), "cancelled")
        self.assertEqual(runtime._failure_status(RuntimeError("wait_result: cancelled 'ROS action'")), "cancelled")
        self.assertEqual(runtime._failure_status(RuntimeError("action aborted: stopped")), "aborted")
        self.assertEqual(runtime._failure_status(runtime._RpcFailure("rejected", "not accepted")), "rejected")

    def test_service_exception_is_reported_and_counted(self):
        future = RosFuture()
        future.set_exception(ValueError("broken response"))
        callback = self.service_callback(future)
        self.assertEqual(callback(b"request"), b"RPC_FAILED\0broken response")
        health = self.bridge._console_routes[-1]["health"]
        self.assertEqual((health.failures, health.timeouts, health.last_error), (1, 0, "broken response"))

    def test_snapshot_reports_rpc_outcomes(self):
        callback = self.service_callback(ready(b"response"))
        callback(b"request")
        self.bridge._bridges_pub = Mock()
        self.bridge._publish_observe()
        from robot_bus.robot_bus_interfaces.msg.v1 import BridgeSnapshot
        snap = BridgeSnapshot()
        snap.ParseFromString(self.bridge._bridges_pub.publish.call_args.args[0])
        self.assertEqual((snap.routes[0].calls, snap.routes[0].tx), (1, 1))
        self.assertEqual(snap.routes[0].last_status, "succeeded")

    def test_stream_stall_recovery_and_second_stall(self):
        from robot_bus.ros2_bridge.builder.drop_stats import RouteHealth
        health = RouteHealth()
        with patch("robot_bus.ros2_bridge.builder.drop_stats.unix_ms", return_value=1000):
            health.record_rx()
        with patch("robot_bus.ros2_bridge.builder.drop_stats.unix_ms", return_value=16000):
            self.assertTrue(health.take_idle_event(True, True))
            self.assertFalse(health.take_idle_event(True, True))
            health.record_rx()
            self.assertFalse(health.is_idle(True, True))
        with patch("robot_bus.ros2_bridge.builder.drop_stats.unix_ms", return_value=31000):
            self.assertTrue(health.take_idle_event(True, True))
            self.assertFalse(health.is_idle(False, True))

    def test_latched_topic_is_quiet_after_first_sample(self):
        from robot_bus.ros2_bridge.builder.drop_stats import RouteHealth
        health = RouteHealth()
        health.watch_stale = False
        self.assertTrue(health.is_idle(True, True))
        health.last_rx_ms = 1
        self.assertFalse(health.is_idle(True, True))


if __name__ == "__main__":
    unittest.main()
