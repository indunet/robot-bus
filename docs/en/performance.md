English | [中文](../zh/performance.md)

# Robot Bus and ROS 2 performance comparison

[Pre-encoded CDR Topic](#matched-pre-encoded-cdr-topic-workload) · [Sensor payloads](#camera-and-lidar-sizes) · [Native Topic, Service, and Action APIs](#native-api-performance) · [Archive and reproduction](#archive-and-reproduction)

## 2026-11-20 same-host retest

Robot Bus 2.4.0 and ROS 2 Humble each ran three rounds on the same host. The pre-encoded CDR Topic runs used the same application data, encoding, and fixed send rates. The native API runs used each system's own message path and executor, and the tables also include later Robot Bus retests on the host.

The AMD EPYC 9554 host had 256 logical CPUs, 755 GiB of memory, and Linux 5.15.0-203-generic. Docker 29.1.3 ran `ros:humble-ros-base` (Ubuntu 22.04.5, x86_64) pinned to logical CPUs 32–39 on eight physical cores, with an 8 GiB memory limit and 512 MiB of shared memory. Robot Bus used Rust 1.100.0 and ZeroMQ 4.3.4. ROS 2 used GCC 11.4, `rclcpp` 16.0.21, `rmw_fastrtps_cpp` 6.2.10, and Fast DDS 2.6.12. Both used release builds based on `b9b414e35f0274bdcef19843562ca8ca770d5685` with the archived benchmark revisions applied.

## Matched pre-encoded CDR Topic workload

The earlier Robot Bus Topic runs sent raw bytes, while ROS 2 used typed `UInt8MultiArray`. In this workload both paths send the same **80 B CDR layout**, with a 16 B prefix for encapsulation and array metadata and 64 B of application data. Before timing, ROS 2 generates and validates the template with the installed RMW. The timed ROS 2 path uses `GenericPublisher` and `GenericSubscription` with serialized buffers. Both publishers reuse a buffer and stamp only the timestamp, trial ID and sequence. Both subscribers validate the prefix and read those fields directly. Timing excludes application message encoding and decoding and includes middleware copying, framing, queues, forwarding and callbacks. [ROS 2 serialized publication](https://github.com/ros2/rclcpp/blob/16.0.21/rclcpp/src/rclcpp/generic_publisher.cpp#L21-L29) · [ROS 2 serialized subscription](https://github.com/ros2/rclcpp/blob/16.0.21/rclcpp/src/rclcpp/generic_subscription.cpp#L42-L48)

Runs were sequential in one Humble container with CPUs 32–39, 8 GiB of memory and 512 MiB of shared memory. Each path used one publisher, one subscriber and one application receive executor thread. Robot Bus ran TCP, IPC and inproc. ROS 2 ran isolated SHM and loopback UDPv4. Both used best effort and a configured depth of 2048 at the ZeroMQ HWM or DDS KeepLast layer, respectively. Three rounds at each target of 1,000, 10,000 and 50,000 messages/s used a 3 s send window and 100 ms drain. Separate one-at-a-time latency runs used 500 samples. At load we sampled delivered messages whose sequence number was divisible by 16, including arrivals during the drain.

Each cell shows the median [minimum–maximum] across rounds. Receive rates use the send window, and delivery is measured after draining. Latency uses nearest-rank percentiles over delivered samples.

| Target/s | Route | Sent/s | Received in window/s | Delivery % | One-at-a-time p50 µs | Loaded p50 µs | Loaded p99 µs |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | Robot Bus TCP | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 145 [141–151] | 121 [121–127] | 192 [146–194] |
| 1,000 | Robot Bus IPC | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 117 [107–122] | 108 [107–109] | 131 [128–143] |
| 1,000 | Robot Bus inproc | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 55 [50–56] | 48 [48–48] | 57 [57–61] |
| 1,000 | ROS 2 SHM | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 32 [31–33] | 33 [30–34] | 37 [36–40] |
| 1,000 | ROS 2 UDPv4 | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 37 [30–43] | 35 [29–37] | 48 [43–49] |
| 10,000 | Robot Bus TCP | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 145 [132–145] | 94 [94–94] | 124 [122–152] |
| 10,000 | Robot Bus IPC | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 108 [100–124] | 93 [93–93] | 111 [107–147] |
| 10,000 | Robot Bus inproc | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 50 [49–52] | 39 [39–39] | 46 [45–50] |
| 10,000 | ROS 2 SHM | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 33 [28–33] | 25 [25–25] | 29 [28–29] |
| 10,000 | ROS 2 UDPv4 | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 36 [35–36] | 29 [29–29] | 34 [33–36] |
| 50,000 | Robot Bus TCP | 50,000 [50,000–50,000] | 49,999 [49,999–49,999] | 100.00 [100.00–100.00] | 125 [116–144] | 70 [69–75] | 179 [161–188] |
| 50,000 | Robot Bus IPC | 50,000 [50,000–50,000] | 49,999 [49,999–49,999] | 100.00 [100.00–100.00] | 123 [108–124] | 52 [48–52] | 117 [115–119] |
| 50,000 | Robot Bus inproc | 50,000 [50,000–50,000] | 50,000 [50,000–50,000] | 100.00 [100.00–100.00] | 50 [49–55] | 21 [21–21] | 39 [32–39] |
| 50,000 | ROS 2 SHM | 50,000 [50,000–50,000] | 50,000 [50,000–50,000] | 100.00 [100.00–100.00] | 32 [23–33] | 18 [18–18] | 23 [23–23] |
| 50,000 | ROS 2 UDPv4 | 50,000 [50,000–50,000] | 50,000 [50,000–50,000] | 100.00 [100.00–100.00] | 37 [36–39] | 21 [21–21] | 27 [27–34] |

All 45 trials across the three rates delivered every message after draining. At 50,000 messages/s, all five paths received close to the target during the send window. The median loaded p99 was 179 µs for Robot Bus TCP and 23 µs for ROS 2 SHM, about one eighth of the TCP figure at that rate.

Per-run logs, environment, run order and samples are archived locally in `bench-results/20261120-190137-matched-cdr-topic/measurements/`. `execution.tsv` includes UDP datagram counts and `raw.jsonl` includes raw counts and latency samples.

Build both release binaries in the same Linux container and run the [benchmark script](../../benches/topic_cdr_perf/run.sh) and [summary validator](../../benches/topic_cdr_perf/summarize.py).

```bash
source /opt/ros/humble/setup.bash
cargo build --release -p robot-bus-perf --bin robot_bus_perf
bash benches/ros2_perf/run.sh --local --build-only
source benches/ros2_perf/_ws/install/setup.bash
bash benches/topic_cdr_perf/run.sh \
  target/release/robot_bus_perf \
  benches/ros2_perf/_ws/install/ros2_perf/lib/ros2_perf/ros2_perf_bench \
  bench-results/replay-cdr-topic
python3 benches/topic_cdr_perf/summarize.py bench-results/replay-cdr-topic
```

### Throughput under higher load

The follow-up used the same release binary and DDS profile hashes, host, Docker image and resource limits. It ran all five paths at **75,000, 100,000, 200,000 and 400,000 messages/s**, three rounds per path and target. Each trial sent for 3 s and drained for 100 ms. The fixed targets were tested independently to capture repeat-run variation.

A round passed if both rates during the send window reached 90% of the target and loss after the drain was at most 1%. Rates and loss are the median [minimum–maximum] across three rounds. The last column counts qualifying rounds individually.

| Target/s | Route | Sent/s | Received in window/s | Loss after drain (%) | Passed |
| ---: | --- | ---: | ---: | ---: | ---: |
| 75,000 | Robot Bus TCP | 75,002 [75,002–75,002] | 75,000 [75,000–75,001] | 0.00 [0.00–0.00] | 3/3 |
| 75,000 | Robot Bus IPC | 75,002 [75,002–75,002] | 75,001 [75,001–75,001] | 0.00 [0.00–0.00] | 3/3 |
| 75,000 | Robot Bus inproc | 75,002 [75,002–75,002] | 75,002 [75,002–75,002] | 0.00 [0.00–0.00] | 3/3 |
| 75,000 | ROS 2 SHM | 75,002 [75,002–75,002] | 75,002 [75,002–75,002] | 0.00 [0.00–0.00] | 3/3 |
| 75,000 | ROS 2 UDPv4 | 75,002 [75,002–75,002] | 75,002 [74,968–75,002] | 0.00 [0.00–0.04] | 3/3 |
| 100,000 | Robot Bus TCP | 100,000 [100,000–100,000] | 99,998 [99,997–99,998] | 0.00 [0.00–0.00] | 3/3 |
| 100,000 | Robot Bus IPC | 100,000 [100,000–100,000] | 99,999 [99,998–99,999] | 0.00 [0.00–0.00] | 3/3 |
| 100,000 | Robot Bus inproc | 100,000 [100,000–100,000] | 100,000 [99,999–100,000] | 0.00 [0.00–0.00] | 3/3 |
| 100,000 | ROS 2 SHM | 100,000 [100,000–100,000] | 98,924 [98,585–99,102] | 1.08 [0.90–1.42] | 1/3 |
| 100,000 | ROS 2 UDPv4 | 100,000 [100,000–100,000] | 97,597 [97,259–100,000] | 2.40 [0.00–2.74] | 1/3 |
| 200,000 | Robot Bus TCP | 200,000 [200,000–200,000] | 199,995 [199,992–199,995] | 0.00 [0.00–0.00] | 3/3 |
| 200,000 | Robot Bus IPC | 200,000 [200,000–200,000] | 199,997 [199,996–199,997] | 0.00 [0.00–0.00] | 3/3 |
| 200,000 | Robot Bus inproc | 200,000 [200,000–200,000] | 199,999 [199,999–199,999] | 0.00 [0.00–0.00] | 3/3 |
| 200,000 | ROS 2 SHM | 200,000 [200,000–200,000] | 112,187 [112,101–112,505] | 43.57 [43.41–43.61] | 0/3 |
| 200,000 | ROS 2 UDPv4 | 188,229 [187,794–191,364] | 136,793 [133,618–141,282] | 26.79 [25.81–28.65] | 0/3 |
| 400,000 | Robot Bus TCP | 400,000 [400,000–400,000] | 356,621 [352,171–380,218] | 5.80 [0.00–6.88] | 1/3 |
| 400,000 | Robot Bus IPC | 400,000 [400,000–400,000] | 396,263 [395,880–396,791] | 0.93 [0.80–1.03] | 2/3 |
| 400,000 | Robot Bus inproc | 400,000 [400,000–400,000] | 396,900 [396,710–396,941] | 0.77 [0.76–0.82] | 3/3 |
| 400,000 | ROS 2 SHM | 400,000 [399,999–400,000] | 69,939 [69,282–71,907] | 82.33 [81.84–82.50] | 0/3 |
| 400,000 | ROS 2 UDPv4 | 189,019 [188,053–189,191] | 125,392 [122,859–136,482] | 33.30 [27.06–34.70] | 0/3 |

The highest tested target that passed all three rounds was 200,000 messages/s for Robot Bus TCP, 200,000 messages/s for Robot Bus IPC, 400,000 messages/s for Robot Bus inproc, 75,000 messages/s for ROS 2 SHM, 75,000 messages/s for ROS 2 UDP. Measurements and per-round logs are archived locally in `bench-results/20261120-191302-matched-cdr-throughput/measurements/`.

Set `TOPIC_CDR_PERF_RATES='75000 100000 200000 400000'` before invoking the runner to reproduce the higher-load tests. The figures describe the matched pre-encoded Topic paths on this host over three 3 s windows.

### Camera and lidar sizes

The comparison above uses 64 B of application data. This run keeps the same pre-encoded CDR layout and changes the application payload to 256 KiB, 640×480 RGB (921,600 B), and 1920×1080 RGB (6,220,800 B). The first is about one mid-size lidar sweep or a compressed image. The other two are uncompressed camera frames.

The fixed rates were 10 and 30 messages/s, three rounds each, with a 3 s send window and 100 ms drain. Both sides used queue depth 8. A depth of 2048, as used for the 64 B runs, would hold more than 12 GiB of 1080p frames on one side, and sensor streams usually keep only the latest frames. The ROS 2 SHM profile for this run allows a 12 MiB sample and a 128 MiB segment. The 64 KiB cap in the 64 B profile cannot carry these frames. UDP stayed on loopback, with the kernel's default 208 KiB socket buffers. The container limits were unchanged: CPUs 32–39, 8 GiB of memory, and 512 MiB of `/dev/shm`.

A round passed when both send-window rates reached 90% of the target and loss after draining was at most 1%. One-at-a-time latency sends the next message only after the previous one arrives. If a message is not received within 2 s, that path stops and its rate trial does not run.

| Payload | Target/s | Route | Sent/s | Received in window/s | Loss after drain (%) | One-at-a-time p50 (µs) | Loaded p50 (µs) | Loaded p99 (µs) | Passed |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 256 KiB | 10 | Robot Bus TCP | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 766 [735–838] | 790 [740–856] | 1,148 [1,052–1,187] | 3/3 |
| 256 KiB | 10 | Robot Bus IPC | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 710 [709–750] | 720 [718–754] | 1,057 [1,042–1,116] | 3/3 |
| 256 KiB | 10 | Robot Bus inproc | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 466 [444–469] | 474 [452–479] | 634 [627–638] | 3/3 |
| 256 KiB | 10 | ROS 2 SHM | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 189 [185–213] | 192 [183–213] | 364 [338–378] | 3/3 |
| 256 KiB | 10 | ROS 2 UDPv4 | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 249 [246–252] | 273 [246–276] | 468 [362–483] | 3/3 |
| 256 KiB | 30 | Robot Bus TCP | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 774 [755–847] | 1,083 [1,046–1,136] | 1,216 [1,107–1,241] | 3/3 |
| 256 KiB | 30 | Robot Bus IPC | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 703 [699–834] | 1,021 [985–1,048] | 1,061 [1,034–1,067] | 3/3 |
| 256 KiB | 30 | Robot Bus inproc | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 461 [458–470] | 587 [568–600] | 615 [606–626] | 3/3 |
| 256 KiB | 30 | ROS 2 SHM | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 184 [158–186] | 339 [283–378] | 399 [310–410] | 3/3 |
| 256 KiB | 30 | ROS 2 UDPv4 | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 188 [157–190] | 366 [331–384] | 405 [380–420] | 3/3 |
| 640×480 RGB | 10 | Robot Bus TCP | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 2,502 [2,450–2,535] | 2,555 [2,520–2,631] | 2,829 [2,767–2,911] | 3/3 |
| 640×480 RGB | 10 | Robot Bus IPC | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 2,770 [2,297–2,801] | 2,744 [2,288–2,804] | 2,755 [2,652–2,805] | 3/3 |
| 640×480 RGB | 10 | Robot Bus inproc | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 1,229 [1,227–1,266] | 1,245 [1,233–1,286] | 1,943 [1,923–1,945] | 3/3 |
| 640×480 RGB | 10 | ROS 2 SHM | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 522 [481–554] | 502 [477–535] | 867 [852–887] | 3/3 |
| 640×480 RGB | 10 | ROS 2 UDPv4 | 10 [10–10] | 9 [5–10] | 13.33 [3.33–46.67] | — | 670 [555–675] | 705 [555–851] | 0/3 |
| 640×480 RGB | 30 | Robot Bus TCP | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 2,542 [2,532–2,546] | 2,841 [2,836–2,893] | 2,942 [2,924–3,226] | 3/3 |
| 640×480 RGB | 30 | Robot Bus IPC | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 2,461 [2,418–3,410] | 2,774 [2,768–2,797] | 2,840 [2,824–3,055] | 3/3 |
| 640×480 RGB | 30 | Robot Bus inproc | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 1,277 [1,269–1,281] | 1,877 [1,860–1,887] | 1,913 [1,895–1,931] | 3/3 |
| 640×480 RGB | 30 | ROS 2 SHM | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 504 [423–542] | 706 [596–764] | 787 [658–819] | 3/3 |
| 640×480 RGB | 30 | ROS 2 UDPv4 | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 524 [473–574] | 821 [757–830] | 854 [815–900] | 3/3 |
| 1920×1080 RGB | 10 | Robot Bus TCP | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 23,255 [19,222–30,236] | 17,568 [16,741–18,546] | 23,577 [19,217–30,498] | 3/3 |
| 1920×1080 RGB | 10 | Robot Bus IPC | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 30,168 [23,369–30,179] | 27,784 [23,578–28,676] | 30,448 [27,289–31,225] | 3/3 |
| 1920×1080 RGB | 10 | Robot Bus inproc | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 5,501 [5,498–5,657] | 5,524 [5,505–5,830] | 12,124 [11,772–12,549] | 3/3 |
| 1920×1080 RGB | 10 | ROS 2 SHM | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 2,729 [2,719–2,736] | 6,833 [6,789–7,003] | 14,315 [14,160–14,663] | 3/3 |
| 1920×1080 RGB | 10 | ROS 2 UDPv4 | 10 [10–10] | 9 [0–9] | 10.00 [6.67–100.00] | — | 3,929 [3,421–4,436] | 3,929 [3,421–4,436] | 0/3 |
| 1920×1080 RGB | 30 | Robot Bus TCP | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 23,185 [23,016–23,535] | 18,436 [17,492–19,379] | 23,510 [23,270–23,532] | 3/3 |
| 1920×1080 RGB | 30 | Robot Bus IPC | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 25,237 [23,549–30,266] | 27,805 [27,489–28,127] | 29,979 [28,336–30,271] | 3/3 |
| 1920×1080 RGB | 30 | Robot Bus inproc | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 5,548 [5,361–5,689] | 12,005 [11,372–12,487] | 12,063 [11,541–13,112] | 3/3 |
| 1920×1080 RGB | 30 | ROS 2 SHM | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 2,715 [2,713–2,718] | 14,185 [14,019–14,237] | 14,467 [14,319–14,560] | 3/3 |
| 1920×1080 RGB | 30 | ROS 2 UDPv4 | 30 [30–30] | 29 [12–30] | 3.33 [0.00–60.00] | — | 3,475 [3,329–4,261] | 4,299 [3,488–4,690] | 1/3 |

1080p at 30 messages/s is about 187 MB/s of application data. Robot Bus TCP, IPC, and inproc, and ROS 2 SHM, delivered every frame in all three rounds. One-at-a-time median p50 on that row was 2.7 ms for SHM, 5.5 ms for inproc, 23.2 ms for TCP, and 25.2 ms for IPC. A Robot Bus message passes through the broker, so a large frame is copied an extra time. SHM shares the frame between the publisher and subscriber.

UDP delivered both 256 KiB rates and 640×480 at 30 messages/s in all three rounds. On the first run, the larger frames did not arrive within 2 s during one-at-a-time sampling, so the rate trial was skipped. Those cells were rerun at the same fixed rate with one-at-a-time sampling turned off. The rerun lost a median 13.33% at 640×480 and 10 messages/s, 10.00% at 1080p and 10 messages/s (one round delivered nothing), and 3.33% at 1080p and 30 messages/s, where only one round passed. A dash in the one-at-a-time column means that cell has no complete serial sample. The kernel UDP socket buffer is 208 KiB, which is smaller than these frames, so fragments are dropped. The rerun logs are in `bench-results/20261120-2132-sensor-payload/udp-rate/`.

Logs are in `bench-results/20261120-2132-sensor-payload/`. To reproduce, build both Release binaries in the same container and run [run_sensor.sh](../../benches/topic_cdr_perf/run_sensor.sh). The defaults are the sizes and rates above, with queue depth 8. The 64 B runner and its Fast DDS profiles are unchanged.

## Native API performance

Topic runs used a 64 B application payload, one publisher, and one subscriber. Robot Bus sent raw bytes through a broker. The native ZeroMQ HWM and the WS Subscribe server queue depth were both 2048. ROS 2 used typed `UInt8MultiArray`, a two-thread `MultiThreadedExecutor`, and best-effort `KeepLast(2048)` QoS, with ROS message encoding and decoding inside the timed path. WS Publish sent from WebSocket to a native subscriber, WS Subscribe from a native publisher to WebSocket, and federation from broker A to broker B.

Each Topic throughput trial sent for 1 second and drained for 100 ms. The search covered 100–750,000 Hz in 100 Hz increments. A trial passed with at most 1% loss after draining and both send-window rates at or above 90% of the target. The selected rates were below the search ceiling. Separate one-at-a-time measurements collected 500 one-way latency samples. Service and Action each used 8 warm-up calls and 32 serial measured calls. Each Action included one feedback message and the final result. A single slow RPC can materially affect p99.

Each cell shows the median [minimum–maximum] over three rounds. Topic receive rates count unique in-trial arrivals during the send window. Delivery divides unique in-trial arrivals after draining by sends. RPC rates divide successful completions by measured elapsed time. The Robot Bus WebSocket rows and TCP, IPC, and inproc Action rows come from CPU-pinned host retests. The remaining rows come from separate runs in the same container. The ROS 2 rows use the [corrected isolated transports](#ros-2-transport-audit).

### Topic publish and subscribe

| System | Path | Receives/s | Delivery (%) | p50 (µs) | p95 (µs) | p99 (µs) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Robot Bus | TCP | 268,340 [242,848–269,716] | 100.0 [100.0–100.0] | 146 [144–148] | 158 [156–158] | 189 [174–194] |
| Robot Bus | IPC | 374,933 [374,928–374,934] | 100.0 [100.0–100.0] | 109 [106–110] | 115 [112–116] | 149 [126–154] |
| Robot Bus | inproc | 374,950 [374,950–374,951] | 100.0 [100.0–100.0] | 49 [47–51] | 53 [53–58] | 92 [67–93] |
| Robot Bus | WebSocket Publish | 6,076 [6,073–6,078] | 99.6 [99.6–99.7] | 214 [208–217] | 239 [235–244] | 337 [313–362] |
| Robot Bus | WebSocket Subscribe | 132,441 [130,154–136,225] | 99.9 [99.6–99.9] | 203 [203–205] | 230 [230–253] | 284 [283–297] |
| Robot Bus | Two-broker federation | 592,504 [580,645–598,393] | 100.0 [100.0–100.0] | 239 [231–248] | 276 [266–316] | 339 [334–356] |
| ROS 2 | SHM | 60,020 [59,947–61,699] | 99.5 [99.0–100.0] | 35 [24–35] | 39 [37–39] | 78 [47–91] |
| ROS 2 | UDPv4 | 62,236 [62,162–71,601] | 100.0 [100.0–100.0] | 39 [38–39] | 44 [44–69] | 85 [82–91] |

### Service call

| System | Path | Completions/s | Completion (%) | p50 (µs) | p95 (µs) | p99 (µs) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Robot Bus | TCP | 4,682 [4,420–4,683] | 100.0 [100.0–100.0] | 213 [213–225] | 227 [218–231] | 233 [225–242] |
| Robot Bus | IPC | 5,422 [5,047–5,764] | 100.0 [100.0–100.0] | 183 [172–196] | 192 [177–206] | 208 [199–233] |
| Robot Bus | inproc | 12,751 [12,224–13,461] | 100.0 [100.0–100.0] | 80 [68–81] | 89 [83–94] | 109 [91–161] |
| Robot Bus | WebSocket | 6,540 [5,606–6,598] | 100.0 [100.0–100.0] | 153 [151–178] | 165 [158–196] | 170 [160–203] |
| Robot Bus | Two-broker federation | 3,122 [2,924–3,820] | 100.0 [100.0–100.0] | 306 [262–339] | 363 [276–371] | 421 [284–432] |
| ROS 2 | SHM | 28,784 [28,316–31,069] | 100.0 [100.0–100.0] | 31 [30–33] | 43 [41–46] | 50 [44–51] |
| ROS 2 | UDPv4 | 21,130 [17,469–22,764] | 100.0 [100.0–100.0] | 44 [43–57] | 56 [46–58] | 59 [56–60] |

### Action goal to result

| System | Path | Completions/s | Completion (%) | p50 (µs) | p95 (µs) | p99 (µs) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Robot Bus | TCP | 2,092 [1,363–2,426] | 100.0 [100.0–100.0] | 542 [381–726] | 572 [554–774] | 606 [573–799] |
| Robot Bus | IPC | 3,015 [2,219–3,385] | 100.0 [100.0–100.0] | 296 [288–448] | 462 [320–501] | 481 [356–528] |
| Robot Bus | inproc | 7,283 [7,236–7,314] | 100.0 [100.0–100.0] | 126 [121–131] | 179 [171–215] | 255 [185–296] |
| Robot Bus | WebSocket | 3,087 [2,359–3,225] | 100.0 [100.0–100.0] | 318 [304–421] | 376 [354–477] | 390 [357–479] |
| Robot Bus | Two-broker federation | 3,009 [2,833–3,244] | 100.0 [100.0–100.0] | 331 [303–351] | 348 [340–370] | 377 [375–398] |
| ROS 2 | SHM | 9,624 [9,535–9,937] | 100.0 [100.0–100.0] | 100 [98–101] | 121 [115–127] | 139 [134–174] |
| ROS 2 | UDPv4 | 8,075 [7,971–8,161] | 100.0 [100.0–100.0] | 122 [122–125] | 134 [129–138] | 144 [139–145] |

## ROS 2 transport audit

The original benchmark placed the publisher and subscriber in one process. Fast DDS 2.6 enables intra-process delivery by default, which can bypass the transport. In two 20,000 msg/s checks with the original binary and container, the original UDP profile emitted only 21 UDP datagrams per run. With intra-process delivery disabled, it emitted more than 80,000. With that same bypass disabled, the original SHM profile also emitted about 20,000 UDP datagrams. Humble's `ROS_LOCALHOST_ONLY=1` appends both UDP and SHM to the XML transport configuration. The original SHM and UDP rows therefore describe the same-process configurations used at the time. [Fast DDS intra-process delivery](https://fast-dds.docs.eprosima.com/en/2.6.x/fastdds/transport/intraprocess.html) · [Humble RMW source](https://github.com/ros2/rmw_fastrtps/blob/6.2.10/rmw_fastrtps_shared_cpp/src/participant.cpp#L145-L162)

The corrected profiles disable intra-process delivery. The runner sets `ROS_LOCALHOST_ONLY=0`, and the UDP profile restricts its interface to loopback. At a fixed load of about 20,000 messages, both profiles delivered all messages. SHM emitted zero UDP datagrams and UDPv4 emitted 20,138. See the [SHM XML](../../benches/ros2_perf/config/fastdds_shm.xml) and [UDPv4 XML](../../benches/ros2_perf/config/fastdds_udp.xml). Fast DDS SHM transport, Data-sharing, and loaned zero-copy delivery have distinct configuration requirements. [Fast DDS Data-sharing](https://fast-dds.docs.eprosima.com/en/2.6.x/fastdds/transport/datasharing.html)

At a fixed 80,000 msg/s under the original configuration, changing the executor from two threads to one increased send-window receives from about 51,000 to 80,000. This shows that the original results were sensitive to executor configuration. The throughput search reports the highest passing rate it encountered. Repeat rates independently to characterize variation near that point.

## Robot Bus WebSocket and Action retests

A historical WS Subscribe trial reported 100.1% delivery (`sent=151447`, `recv=151595`). The old counter could count messages still queued from the previous trial. In a controlled regression, one send produced three observed arrivals, including one unique in-trial receive, one cross-trial arrival, and one duplicate. With trial IDs and sequence numbers, all 77 measured WS throughput trials satisfied `recv_send <= recv_final <= sent`.

TCP, IPC, and inproc Action were remeasured on the default SDK path. Queueing a reply now wakes the executor, eliminating the wait for the 250 ms poll timeout. That retest ran on the host pinned to CPUs 32–39, with the same warm-up and sample counts. The logs are in `bench-results/20261120-2115-executor-wake/`. WebSocket Action took about 44 ms and federation Action about 0.34 ms in that run.

The 2026-11-21 WebSocket retest ran three rounds each before (`d24a506`) and after the change, back to back on the same CPU-pinned host outside the container's 8 GiB memory limit. The WebSocket server and Rust client both enabled `TCP_NODELAY`, and WS Subscribe switched from polling its inbox every 5 ms to waiting for arrivals. The table shows the before and after figures from that host retest.

| Scenario | Before: receives or completions/s | Before: p50 (µs) | Before: p99 (µs) | After: receives or completions/s | After: p50 (µs) | After: p99 (µs) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| ws message Publish | 6,083 | 217 | 336 | 6,076 | 214 | 337 |
| ws message Subscribe | 233,501 | 4,556 | 5,085 | 132,441 | 203 | 284 |
| ws service Call | 23 | 43,992 | 48,030 | 6,540 | 153 | 170 |
| ws action SendGoal | 23 | 44,010 | 48,000 | 3,087 | 318 | 390 |
| tcp message pub/sub | 208,507 | 154 | 217 | 206,315 | 155 | 220 |
| tcp service call | 5,698 | 176 | 186 | 6,751 | 147 | 158 |
| ipc service call | 8,807 | 117 | 124 | 7,557 | 134 | 154 |
| federation message pub/sub | 442,157 | 248 | 357 | 442,665 | 256 | 352 |

With only `TCP_NODELAY` enabled, WS Subscribe topped out at about 128,000 messages/s, near the 132,441 messages/s recorded after both changes. The pre-change figure was 233,501 messages/s. After the inbox change, the one-way WS Subscribe p50 fell from 4,556 µs to 203 µs. The three rounds of logs, raw data, baseline binary, `TCP_NODELAY` control build, and summary script are in `bench-results/20261121-0911-ws-nodelay-inbox/`.

## Native topic QoS performance

Release builds from the workspace before and after the change ran alternately on the same AMD EPYC 9554 host, pinned to CPUs 48–55. The benchmark used a socket HWM of 2,048 and a single-threaded Node receiver. The 64 B and 256 KiB application payloads used a 16 B CDR header. Fixed-rate figures are medians of five runs. Single-message p99 comes from a separate probe that waits for each sample before sending the next.

| Transport | 64 B received/s at 100,000/s | 64 B single-message p99 (µs) | 256 KiB received/s at 500/s | 256 KiB single-message p99 (µs) |
| --- | ---: | ---: | ---: | ---: |
| TCP | 99,988 → 99,980 | 186 → 234 | 500 → 500 | 937 → 507 |
| IPC | 99,992 → 99,992 | 149 → 173 | 500 → 500 | 755 → 470 |
| inproc | 99,998 → 99,997 | 93 → 96 | 500 → 500 | 499 → 165 |

All fixed-rate runs delivered every sample after draining. Under the 100,000/s TCP load, the sampled 64 B p99 was 32.7 ms before and 32.1 ms after. Three stepped searches used a 1% loss limit and required sending and receiving at least 90% of the offered rate during the window. Median sustained 64 B receive rates were 200k → 200k/s over TCP, 300k → 425k/s over IPC, and 300k → 525k/s over inproc. The 256 KiB rates went from about 5,700/s to near the 6,500/s search ceiling on all three transports.

For stateful topics, both endpoints enabled `transient_local`. The publisher wrote the sample before the subscriber registered. Replay latency runs from registration on an already-connected Node to its first callback. All five update trials at 5,000/s for 64 B and 500/s for 256 KiB delivered every sample after draining.

| Transport | Median first replay (µs) | Single-update p99, 64 B / 256 KiB (µs) |
| --- | ---: | ---: |
| TCP | 1,008 | 242 / 613 |
| IPC | 355 | 220 / 541 |
| inproc | 167 | 139 / 304 |

The slow-consumer test blocked the first callback, then published samples numbered 1 through 10. At depth 2, both single- and multi-threaded executors delivered only 9 and 10 after the callback unblocked. State tests covered late join, TCP reconnect after a broker restart, publisher shutdown, and replay over broker federation.

Set `ROBOT_BUS_PERF_ONLY=cdr-topic` and `ROBOT_BUS_PERF_PAYLOAD_BYTES=64` or `262144` before running `cargo run --release -p robot-bus-perf`. Set `ROBOT_BUS_PERF_GOODPUT_RATE_LO` and `ROBOT_BUS_PERF_GOODPUT_RATE_HI` to the same value for fixed-rate trials. Add `ROBOT_BUS_PERF_TOPIC_QOS=transient-local` to measure state updates and replay. See the [benchmark guide](../../benches/robot_bus_perf/README.md).

### Single-message latency attribution

Three repeated runs of the original before/after binaries produced median TCP single-message p99s of 181 → 194 µs for 64 B and 925 → 504 µs for 256 KiB. The single-message probe waits for the callback after each send. The 500/s paced window ran separately. Its sampled 256 KiB TCP p99 in the original five runs was 859 → 556 µs.

Temporary builds measured from receipt of both ZeroMQ frames to entry into the user callback. For 64 B TCP, the median for this interval went from **0.34 µs** to **25.1 µs**. The receive thread now enqueues the sample and wakes the executor to run the callback. The median for this interval was 23–25 µs across the three transports. End-to-end p99 also reflects transport and scheduling variability.

The 256 KiB difference comes mainly from allocation and copying. The former path converted the received `Vec<u8>` into `Arc<[u8]>`, allocating and copying the entire sample again. Replacing only that conversion with `Arc::new(vec)` in the temporary inline-dispatch build reduced TCP p99 from **871 µs to 524 µs**. Allocation plus copy in the previous path took about **240 µs** at the median in this setup.

A separate `strace` run counted **2,211 `brk` calls** in the original and **7** in the new version. The original receive thread repeatedly grew and shrank its heap. Setting glibc `mmap_threshold` and `trim_threshold` to 1 MiB for the unmodified original binary reduced its median TCP p99 over three runs from **940 µs to 554 µs**. The controls attribute most of the gap in this environment to repeated allocation and heap resizing. The new subscription queues share the received buffer. The allocator settings were used only for diagnosis. [glibc memory allocation tunables](https://sourceware.org/glibc/manual/latest/html_node/Memory-Allocation-Tunables.html)

## Archive and reproduction

The initial local archive is `bench-results/20261120-134156-same-host/`. Its `run.sh` is the executed command sequence, `execution.tsv` records run order and exit codes, `raw.jsonl` contains per-trial counts, windows and latency samples, and `round-*.log` contains the full output. It also contains the environment, DDS profiles, source patches, Cargo lockfile, binary hashes, `SHA256SUMS` and an earlier Bus smoke timeout. The transport audit and corrected ROS 2 samples live in `transport/` and `isolated/` under `bench-results/20261120-183629-ros2-executor-probe/`, with XML, environment details, UDP counts and full logs.

See [contributing](../../CONTRIBUTING.md) for source-build prerequisites. `just perf` and `just perf-ros2` write generated Markdown reports under the local `bench-results/` directory. To capture only Robot Bus logs, set `ROBOT_BUS_PERF_SKIP_REPORT=1`. When invoking `ros2_perf_bench` directly, use `ROS2_PERF_REPORT` to set its report path. See the [Robot Bus bench](../../benches/robot_bus_perf/README.md) and [ROS 2 bench](../../benches/ros2_perf/README.md) for scenario commands. The bidirectional tests through `Ros2Bridge` and their status are covered in the [bridge performance report](ros2-bridge-perf-report.md).

Record the following for each measured run.

1. Date, commit and source changes, CPU model and core count, memory, OS/kernel, container resource limits, release-build toolchain, ROS distribution, RMW and DDS versions.
2. Payload type and size, publisher/subscriber counts, process layout, transport, HWM/QoS, trial and drain windows, and all environment overrides.
3. Raw counts, timing windows, latency samples, failures and skips for each round, plus the median and range across rounds.
