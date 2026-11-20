English | [中文](../zh/ros2-perf-report.md)

# ROS 2 Humble performance report

On 2026-11-20, Fast DDS SHM and UDPv4 each ran three rounds with corrected profiles in the same host, container and resource limits used for Robot Bus. This was a separate ROS 2 run. The release build used ROS 2 Humble, `rmw_fastrtps_cpp` 6.2.10, Fast DDS 2.6.12, `rclcpp` 16.0.21 and GCC 11.4. The archive retains both the original diagnostic data and the new samples. [Performance guide](performance.md)

The Topic results here use typed `UInt8MultiArray` and include ROS message encoding and decoding. See the [matched pre-encoded CDR workload](performance.md#matched-pre-encoded-cdr-topic-workload) for direct serialized publication and subscription.

## Method

- Multiple nodes ran in one process with a two-thread `MultiThreadedExecutor`. Message runs used `UInt8MultiArray` with a 64 B application payload, one publisher, one subscriber, and best-effort `KeepLast(2048)` QoS.
- Both Fast DDS profiles disable intra-process delivery, and the runner sets `ROS_LOCALHOST_ONLY=0` so the XML selects SHM or loopback-only UDPv4. Both modes delivered all 20,000 messages in the fixed-rate transport check. The UDP datagram count was 0 for SHM and 20,138 for UDPv4. [SHM profile](../../benches/ros2_perf/config/fastdds_shm.xml) · [UDPv4 profile](../../benches/ros2_perf/config/fastdds_udp.xml)
- Each throughput trial sent for 1 second and drained for 100 ms. The search ranged from 100 to 750,000 Hz in 100 Hz increments. A trial passed with at most 1% loss after draining and both send-window rates at or above 90% of the target. The selected rate for both transports fell below the search ceiling.
- Separate runs collected 500 one-way message latency samples. Service and Action each used 8 warm-up calls and 32 serial measured calls. Each Action included one feedback message and the final result. With 32 samples, an unusually slow call can materially affect p99.

## Results across three rounds

Each cell shows the median [minimum–maximum] across rounds. The message receive rate counts unique in-trial receives during the send window. Delivery uses unique in-trial receives after draining divided by sends. RPC rates use successful completions and elapsed call time.

| Transport | Scenario | Receives or completions/s | Delivery or completion (%) | p50 (µs) | p95 (µs) | p99 (µs) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| shm | message pub/sub | 60,020 [59,947–61,699] | 99.5 [99.0–100.0] | 35 [24–35] | 39 [37–39] | 78 [47–91] |
| shm | service call | 28,784 [28,316–31,069] | 100.0 [100.0–100.0] | 31 [30–33] | 43 [41–46] | 50 [44–51] |
| shm | action send_goal | 9,624 [9,535–9,937] | 100.0 [100.0–100.0] | 100 [98–101] | 121 [115–127] | 139 [134–174] |
| udp | message pub/sub | 62,236 [62,162–71,601] | 100.0 [100.0–100.0] | 39 [38–39] | 44 [44–69] | 85 [82–91] |
| udp | service call | 21,130 [17,469–22,764] | 100.0 [100.0–100.0] | 44 [43–57] | 56 [46–58] | 59 [56–60] |
| udp | action send_goal | 8,075 [7,971–8,161] | 100.0 [100.0–100.0] | 122 [122–125] | 134 [129–138] | 144 [139–145] |

These figures cover ordinary typed publication, subscription callbacks and RPC. Fast DDS SHM transport, Data-sharing and loaned zero-copy delivery each have distinct configuration requirements. A cross-product comparison still requires matched application paths and workloads. [Fast DDS Data-sharing](https://fast-dds.docs.eprosima.com/en/2.6.x/fastdds/transport/datasharing.html) · [Reproduction and raw data](performance.md)

## Camera and lidar sizes

The same container, CPUs 32–39, 8 GiB of memory and 512 MiB of shared memory were used to remeasure pre-encoded CDR payloads of 256 KiB, 640×480 RGB and 1920×1080 RGB. The rates were 10 and 30 messages/s, the send window was 3 s, and the queue depth was 8. The SHM sample cap was 12 MiB. UDP stayed on loopback with the kernel's 208 KiB socket buffers. The Robot Bus TCP, IPC and inproc rows for the same payloads are in the [Robot Bus report](perf-report.md).

Each cell is the median [minimum–maximum] of three rounds. One-at-a-time p50 sends the next message only after the previous one arrives. Loaded latency is sampled during the send window. A round passed when both window rates reached 90% of the target and loss after draining was at most 1%.

| Payload | Target/s | Transport | Sent/s | Received in window/s | Loss after drain (%) | One-at-a-time p50 (µs) | Loaded p50 (µs) | Loaded p99 (µs) | Passed |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 256 KiB | 10 | shm | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 189 [185–213] | 192 [183–213] | 364 [338–378] | 3/3 |
| 256 KiB | 10 | udp | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 249 [246–252] | 273 [246–276] | 468 [362–483] | 3/3 |
| 256 KiB | 30 | shm | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 184 [158–186] | 339 [283–378] | 399 [310–410] | 3/3 |
| 256 KiB | 30 | udp | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 188 [157–190] | 366 [331–384] | 405 [380–420] | 3/3 |
| 640×480 RGB | 10 | shm | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 522 [481–554] | 502 [477–535] | 867 [852–887] | 3/3 |
| 640×480 RGB | 10 | udp | 10 [10–10] | 9 [5–10] | 13.33 [3.33–46.67] | — | 670 [555–675] | 705 [555–851] | 0/3 |
| 640×480 RGB | 30 | shm | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 504 [423–542] | 706 [596–764] | 787 [658–819] | 3/3 |
| 640×480 RGB | 30 | udp | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 524 [473–574] | 821 [757–830] | 854 [815–900] | 3/3 |
| 1920×1080 RGB | 10 | shm | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 2,729 [2,719–2,736] | 6,833 [6,789–7,003] | 14,315 [14,160–14,663] | 3/3 |
| 1920×1080 RGB | 10 | udp | 10 [10–10] | 9 [0–9] | 10.00 [6.67–100.00] | — | 3,929 [3,421–4,436] | 3,929 [3,421–4,436] | 0/3 |
| 1920×1080 RGB | 30 | shm | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 2,715 [2,713–2,718] | 14,185 [14,019–14,237] | 14,467 [14,319–14,560] | 3/3 |
| 1920×1080 RGB | 30 | udp | 30 [30–30] | 29 [12–30] | 3.33 [0.00–60.00] | — | 3,475 [3,329–4,261] | 4,299 [3,488–4,690] | 1/3 |

SHM delivered every frame in all six rows. UDP did the same at both 256 KiB rates and at 640×480 and 30 messages/s. Median loss was 13.33% at 640×480 and 10 messages/s, 10.00% at 1080p and 10 messages/s, and 3.33% at 1080p and 30 messages/s, where only one round passed. The three one-at-a-time dashes are cells whose first run received no next message within 2 s. Their rate numbers come from a rerun with one-at-a-time sampling disabled. Logs are in `bench-results/20261120-2132-sensor-payload/`, and the UDP rerun is in `udp-rate/` there.
