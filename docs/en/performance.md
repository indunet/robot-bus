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
