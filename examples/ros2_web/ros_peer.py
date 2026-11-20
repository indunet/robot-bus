"""Real DDS peer for the browser demo; deliberately independent of robot_bus."""

import json
import signal
import threading
import time

import rclpy
from example_interfaces.action import Fibonacci
from rclpy.action import ActionServer, CancelResponse, GoalResponse
from rclpy.callback_groups import ReentrantCallbackGroup
from rclpy.executors import MultiThreadedExecutor
from rclpy.node import Node
from std_msgs.msg import String
from std_srvs.srv import Trigger

PREFIX = "/ros2_web"


class DemoRobot(Node):
    def __init__(self):
        super().__init__("ros2_web_robot")
        self.lock = threading.Lock()
        self.state = dict(sequence=0, last_command="", resets=0,
                          slow_started=0, slow_completed=0, goals_started=0,
                          goals_succeeded=0, goals_rejected=0,
                          goals_cancelled=0, goals_aborted=0)
        self.group = ReentrantCallbackGroup()
        self.status = self.create_publisher(String, PREFIX + "/status", 10)
        self.create_subscription(String, PREFIX + "/command", self.command, 10)
        self.create_timer(0.1, self.publish_status)
        self.create_service(Trigger, PREFIX + "/reset", self.reset,
                            callback_group=self.group)
        self.create_service(Trigger, PREFIX + "/slow", self.slow,
                            callback_group=self.group)
        self.action = ActionServer(
            self, Fibonacci, PREFIX + "/fibonacci", self.execute,
            callback_group=self.group,
            goal_callback=self.on_goal,
            cancel_callback=lambda _goal: CancelResponse.ACCEPT,
        )

    def command(self, msg):
        with self.lock:
            self.state["last_command"] = msg.data

    def publish_status(self):
        with self.lock:
            self.state["sequence"] += 1
            payload = json.dumps(self.state)
        self.status.publish(String(data=payload))

    def reset(self, _request, response):
        with self.lock:
            self.state["resets"] += 1
            response.message = f"reset:{self.state['resets']}"
        response.success = True
        return response

    def slow(self, _request, response):
        with self.lock:
            self.state["slow_started"] += 1
        time.sleep(2)  # Bridge deadline is 0.4 s; the ROS operation still completes.
        with self.lock:
            self.state["slow_completed"] += 1
        response.success = True
        response.message = "late response"
        return response

    def on_goal(self, goal):
        if 1 <= goal.order <= 30:
            return GoalResponse.ACCEPT
        with self.lock:
            self.state["goals_rejected"] += 1
        return GoalResponse.REJECT

    def execute(self, goal):
        with self.lock:
            self.state["goals_started"] += 1
        sequence = []
        for i in range(goal.request.order):
            if goal.is_cancel_requested or not rclpy.ok():
                goal.canceled()
                with self.lock:
                    self.state["goals_cancelled"] += 1
                return Fibonacci.Result(sequence=sequence)
            if goal.request.order == 13:
                goal.abort()
                with self.lock:
                    self.state["goals_aborted"] += 1
                return Fibonacci.Result(sequence=sequence)
            sequence.append(i if i < 2 else sequence[-1] + sequence[-2])
            goal.publish_feedback(Fibonacci.Feedback(sequence=sequence))
            time.sleep(0.15)
        goal.succeed()
        with self.lock:
            self.state["goals_succeeded"] += 1
        return Fibonacci.Result(sequence=sequence)


def main():
    rclpy.init()
    node = DemoRobot()
    executor = MultiThreadedExecutor(num_threads=4)
    executor.add_node(node)
    stopped = threading.Event()
    # Leave the executor alive while in-flight callbacks finish during shutdown.
    signal.signal(signal.SIGTERM, lambda *_: stopped.set())
    signal.signal(signal.SIGINT, lambda *_: stopped.set())
    try:
        print("ROS peer ready", flush=True)
        while not stopped.is_set():
            executor.spin_once(timeout_sec=0.1)
    finally:
        executor.shutdown(timeout_sec=8)
        node.action.destroy()
        node.destroy_node()
        rclpy.shutdown()


if __name__ == "__main__":
    main()
