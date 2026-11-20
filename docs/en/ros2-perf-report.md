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
