English | [中文](../zh/performance.md)

# Performance interpretation and reproduction

## 2026-11-20 same-host retest

Robot Bus and ROS 2 Humble initially ran three rounds each in the same Docker container. All nine process runs exited successfully. The initial archive contains 306 throughput trials, 24 sets of one-way latency samples and 42 sets of RPC samples. A follow-up audit found that the initial ROS configuration did not isolate SHM and UDPv4. ROS 2 was rerun with corrected profiles in the same container.

| Report | Coverage |
| --- | --- |
| [Robot Bus](perf-report.md) | Message, Service and Action over TCP, IPC, inproc, WebSocket and two-broker federation |
| [ROS 2 Humble](ros2-perf-report.md) | Message, Service and Action over Fast DDS SHM and UDPv4 with the corrected profiles |
| [ROS 2 bridge](ros2-bridge-perf-report.md) | Humble and Jazzy build and route checks, with bridge throughput and latency sampling pending |

The AMD EPYC 9554 host had 256 logical CPUs, 755 GiB of memory and Linux 5.15.0-203-generic. Docker 29.1.3 ran `ros:humble-ros-base` (Ubuntu 22.04.5, x86_64) with affinity to logical CPUs 32–39 on eight physical cores, an 8 GiB memory limit and 512 MiB of shared memory. Robot Bus used Rust 1.100.0 and ZeroMQ 4.3.4. ROS 2 used GCC 11.4 and Fast DDS 2.6.12. Both used release builds. The measured source was based on `b9b414e35f0274bdcef19843562ca8ca770d5685` with the archived benchmark revisions applied.

The figures apply to their respective local API paths. Robot Bus routes raw bytes through a broker. ROS 2 uses typed `UInt8MultiArray` messages, DDS and a two-thread executor. ZeroMQ HWM and DDS KeepLast both use a depth of 2048 on different queues. The corrected ROS 2 figures and initial Robot Bus figures come from separate runs. A cross-product comparison still requires aligned application workloads, process and thread layouts, and a common set of fixed rates.

## ROS 2 transport audit

The original benchmark placed the publisher and subscriber in one process. Fast DDS 2.6 enables intra-process delivery by default, which can bypass the transport. In two 20,000 msg/s checks with the original binary and container, the original UDP profile emitted only 21 UDP datagrams per run. With intra-process delivery disabled, it emitted more than 80,000. With that same bypass disabled, the original SHM profile also emitted about 20,000 UDP datagrams. Humble's `ROS_LOCALHOST_ONLY=1` appends both UDP and SHM to the XML transport configuration. The original SHM and UDP rows therefore describe the same-process configurations used at the time. [Fast DDS intra-process delivery](https://fast-dds.docs.eprosima.com/en/2.6.x/fastdds/transport/intraprocess.html) · [Humble RMW source](https://github.com/ros2/rmw_fastrtps/blob/6.2.10/rmw_fastrtps_shared_cpp/src/participant.cpp#L145-L162)

The corrected profiles disable intra-process delivery. The runner sets `ROS_LOCALHOST_ONLY=0`, and the UDP profile restricts its interface to loopback. At a fixed load of about 20,000 messages, both profiles delivered all messages. SHM emitted zero UDP datagrams and UDP emitted about 20,000. The [ROS 2 report](ros2-perf-report.md) uses the corrected runs.

At a fixed 80,000 msg/s under the original configuration, changing the executor from two threads to one increased send-window receives from about 51,000 to 80,000. This shows that the original results were sensitive to executor configuration. The throughput search reports the highest passing rate it encountered. Repeat rates independently to characterize variation near that point.

## Matched pre-encoded CDR Topic workload

The earlier Robot Bus Topic runs sent raw bytes, while ROS 2 used typed `UInt8MultiArray`. In this workload both paths send the same **80 B CDR layout**, with a 16 B prefix for encapsulation and array metadata and 64 B of application data. Before timing, ROS 2 generates and validates the template with the installed RMW. The timed ROS 2 path uses `GenericPublisher` and `GenericSubscription` with serialized buffers. Both publishers reuse a buffer and stamp only the timestamp, trial ID and sequence. Both subscribers validate the prefix and read those fields directly. Timing excludes application message encoding and decoding and includes middleware copying, framing, queues, forwarding and callbacks. [ROS 2 serialized publication](https://github.com/ros2/rclcpp/blob/16.0.21/rclcpp/src/rclcpp/generic_publisher.cpp#L21-L29) · [ROS 2 serialized subscription](https://github.com/ros2/rclcpp/blob/16.0.21/rclcpp/src/rclcpp/generic_subscription.cpp#L42-L48)

Runs were sequential in one Humble container with CPUs 32–39, 8 GiB of memory and 512 MiB of shared memory. Each path used one publisher, one subscriber and one application receive executor thread. Robot Bus ran TCP, IPC and inproc. ROS 2 ran isolated SHM and loopback UDPv4. Both used best effort and a configured depth of 2048 at the ZeroMQ HWM or DDS KeepLast layer, respectively. Three rounds at each target of 1,000, 10,000 and 50,000 messages/s used a 3 s send window and 100 ms drain. Separate one-at-a-time latency runs used 500 samples. At load we sampled delivered messages whose sequence number was divisible by 16, including arrivals during the drain.

Each cell shows the median [minimum–maximum] across rounds. Receive rates use the send window, and delivery is measured after draining. Latency uses nearest-rank percentiles over delivered samples.

| Target/s | Route | Sent/s | Received in window/s | Delivery % | One-at-a-time p50 µs | Loaded p50 µs | Loaded p99 µs |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | tcp | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 145 [141–151] | 121 [121–127] | 192 [146–194] |
| 1,000 | ipc | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 117 [107–122] | 108 [107–109] | 131 [128–143] |
| 1,000 | inproc | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 55 [50–56] | 48 [48–48] | 57 [57–61] |
| 1,000 | shm | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 32 [31–33] | 33 [30–34] | 37 [36–40] |
| 1,000 | udp | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 37 [30–43] | 35 [29–37] | 48 [43–49] |
| 10,000 | tcp | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 145 [132–145] | 94 [94–94] | 124 [122–152] |
| 10,000 | ipc | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 108 [100–124] | 93 [93–93] | 111 [107–147] |
| 10,000 | inproc | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 50 [49–52] | 39 [39–39] | 46 [45–50] |
| 10,000 | shm | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 33 [28–33] | 25 [25–25] | 29 [28–29] |
| 10,000 | udp | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 36 [35–36] | 29 [29–29] | 34 [33–36] |
| 50,000 | tcp | 50,000 [50,000–50,000] | 49,999 [49,999–49,999] | 100.00 [100.00–100.00] | 125 [116–144] | 70 [69–75] | 179 [161–188] |
| 50,000 | ipc | 50,000 [50,000–50,000] | 49,999 [49,999–49,999] | 100.00 [100.00–100.00] | 123 [108–124] | 52 [48–52] | 117 [115–119] |
| 50,000 | inproc | 50,000 [50,000–50,000] | 50,000 [50,000–50,000] | 100.00 [100.00–100.00] | 50 [49–55] | 21 [21–21] | 39 [32–39] |
| 50,000 | shm | 50,000 [50,000–50,000] | 50,000 [50,000–50,000] | 100.00 [100.00–100.00] | 32 [23–33] | 18 [18–18] | 23 [23–23] |
| 50,000 | udp | 50,000 [50,000–50,000] | 50,000 [50,000–50,000] | 100.00 [100.00–100.00] | 37 [36–39] | 21 [21–21] | 27 [27–34] |

All five paths received close to the 50,000 messages/s target at the highest tested rate, and all 45 trials delivered every message after draining. At that target, the median loaded p99 was 179 µs for Robot Bus TCP and 23 µs for ROS 2 SHM. These results cover the fixed rates tested. Maximum throughput and typed application codec costs require separate measurements.

Per-run logs, environment, run order and samples are archived locally in `bench-results/20261120-190137-matched-cdr-topic/measurements/`. `execution.tsv` includes UDP datagram counts and `raw.jsonl` includes raw counts and latency samples.

Build both release binaries in the same Linux container and run the [benchmark script](../../benches/topic_cdr_perf/run.sh) and [summary validator](../../benches/topic_cdr_perf/summarize.py).

```bash
source /opt/ros/humble/setup.bash
cargo build --release --no-default-features --features ws --bin robot_bus_perf
bash benches/ros2_perf/run.sh --local --build-only
source benches/ros2_perf/_ws/install/setup.bash
bash benches/topic_cdr_perf/run.sh \
  target/release/robot_bus_perf \
  benches/ros2_perf/_ws/install/ros2_perf/lib/ros2_perf/ros2_perf_bench \
  bench-results/replay-cdr-topic
python3 benches/topic_cdr_perf/summarize.py bench-results/replay-cdr-topic
```

## Counting and WS validation

The message receive rate divides unique in-trial receives during the send window by the actual send duration. Delivery divides unique in-trial receives after draining by sends. Separate one-at-a-time samples measure message latency. The tables summarize per-round percentiles using the median and minimum–maximum range. RPC rates divide successful completions by measured elapsed time.

The historical WS Subscribe line recorded `sent=151447`, `recv=151595`, and `delivery=100.1%`. The old counter could attribute still-queued arrivals to the next trial. The historical logs lack the identifiers needed to trace the extra 148 events individually. The current benchmark counts unique in-trial receives, cross-trial arrivals and duplicates by trial ID and sequence. All 77 measured WS throughput trials satisfied `recv_send <= recv_final <= sent`. WS Publish and WS Subscribe measure independent directions.

## Archive and reproduction

The initial local archive is `bench-results/20261120-134156-same-host/`. Its `run.sh` is the executed command sequence, `execution.tsv` records run order and exit codes, `raw.jsonl` contains per-trial counts, windows and latency samples, and `round-*.log` contains the full output. It also contains the environment, DDS profiles, source patches, Cargo lockfile, binary hashes, `SHA256SUMS` and an earlier Bus smoke timeout. The transport audit and corrected ROS 2 samples live in `transport/` and `isolated/` under `bench-results/20261120-183629-ros2-executor-probe/`, with XML, environment details, UDP counts and full logs.

See [contributing](../../CONTRIBUTING.md) for source-build prerequisites and the [Robot Bus bench](../../benches/robot_bus_perf/README.md), [ROS 2 bench](../../benches/ros2_perf/README.md) and [bridge bench](../../benches/ros2_bridge_perf/README.md) for commands and dependencies. During sampling, set `ROBOT_BUS_PERF_SKIP_REPORT=1` and use `ROS2_PERF_REPORT` to direct reports to the run archive when invoking `ros2_perf_bench` directly. Save each run's stdout and stderr. The default `just perf`, `just perf-ros2` and `just perf-ros2-bridge` commands write to the corresponding reports.

Record the following for each measured run.

1. Date, commit and source changes, CPU model and core count, memory, OS/kernel, container resource limits, release-build toolchain, ROS distribution, RMW and DDS versions.
2. Payload type and size, publisher/subscriber counts, process layout, transport, HWM/QoS, trial and drain windows, and all environment overrides.
3. Raw counts, timing windows, latency samples, failures and skips for each round, plus the median and range across rounds.
