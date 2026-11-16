"""Serve the browser demo, an embedded broker, Python bridge and real ROS peer."""

import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import logging
from pathlib import Path
import signal
import subprocess
import sys
import threading

import robot_bus
from robot_bus.ros2_bridge import (
    FibonacciActionMapper, Ros2Bridge, StdMsgsStringMapper, TopicQos,
    TriggerServiceMapper,
)

HERE = Path(__file__).resolve().parent
PREFIX = "/ros2_web"


def build_bridge(api_url):
    ros, bus = TopicQos.default(), TopicQos.bus()
    builder = Ros2Bridge.new("ros2_web_bridge").bus_discover(api_url, timeout=10)
    builder = (builder.from_ros(PREFIX + "/status", ros)
               .to_bus(PREFIX + "/status", bus).mapper(StdMsgsStringMapper()).add())
    builder = (builder.from_bus(PREFIX + "/command", bus)
               .to_ros(PREFIX + "/command", ros).mapper(StdMsgsStringMapper()).add())
    for name, timeout in (("reset", 3), ("slow", 0.4)):
        builder = (builder.service().from_bus(PREFIX + "/" + name, bus)
                   .to_ros(PREFIX + "/" + name, ros)
                   .mapper(TriggerServiceMapper()).timeout(timeout).add())
    for name, timeout in (("fibonacci", 10), ("deadline", 0.4)):
        builder = (builder.action().from_bus(PREFIX + "/" + name, bus)
                   .to_ros(PREFIX + "/fibonacci", ros)
                   .mapper(FibonacciActionMapper()).timeout(timeout).add())
    return builder.build()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--broker-port", type=int, default=15560)
    parser.add_argument("--web-port", type=int, default=8080)
    args = parser.parse_args()
    if not (HERE / "dist/index.html").is_file():
        parser.error("browser assets missing: run npm ci && npm run build in examples/ros2_web")
    if not robot_bus.ros2_available():
        parser.error("real ROS is required: source /opt/ros/humble/setup.bash")
    logging.basicConfig(level=logging.INFO)
    stopped = threading.Event()
    signal.signal(signal.SIGTERM, lambda *_: stopped.set())
    signal.signal(signal.SIGINT, lambda *_: stopped.set())
    peer = subprocess.Popen([sys.executable, "-u", str(HERE / "ros_peer.py")])
    bridge = None
    httpd = None
    try:
        with robot_bus.RobotBusBroker.start(
            api_listen=f"{args.host}:{args.broker_port}", tcp_only=True,
            no_tank=True, no_docs=True, no_discovery=True,
        ):
            try:
                bridge = build_bridge(f"http://127.0.0.1:{args.broker_port}")
                httpd = ThreadingHTTPServer(
                    (args.host, args.web_port),
                    partial(SimpleHTTPRequestHandler, directory=str(HERE / "dist")),
                )
                web_thread = threading.Thread(target=httpd.serve_forever, daemon=True)
                web_thread.start()
                print(f"Browser: http://127.0.0.1:{args.web_port}/?brokerPort={args.broker_port}",
                      flush=True)
                while not stopped.is_set():
                    if peer.poll() is not None:
                        raise RuntimeError(f"ROS peer exited with code {peer.returncode}")
                    bridge.spin_once(0.01)
            finally:
                if httpd is not None:
                    httpd.shutdown()
                    httpd.server_close()
                if bridge is not None:
                    bridge.close()
    finally:
        peer.terminate()
        try:
            peer.wait(timeout=10)
        except subprocess.TimeoutExpired:
            peer.kill()
            peer.wait()


if __name__ == "__main__":
    main()
