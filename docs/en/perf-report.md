English | [中文](../zh/perf-report.md)

# Robot Bus performance report

The 2026-11-20 single-host retest covered TCP, IPC, inproc, WebSocket and two-broker federation, with three rounds per scenario. The release build used Robot Bus 2.4.0, Rust 1.100.0 and ZeroMQ 4.3.4. The measured source was based on commit `b9b414e` with the archived benchmark revisions applied. See the [performance guide](performance.md) for the shared hardware and container configuration.

The Topic results here use Robot Bus's raw 64 B byte path. See the [matched pre-encoded CDR workload](performance.md#matched-pre-encoded-cdr-topic-workload) for the run with ROS 2 using the same application byte layout.

## Method

- Message runs used a 64 B payload, one publisher and one subscriber, and a single-host, in-process setup. Native message HWM and the WS Subscribe server queue depth were both 2048. WS Publish sent from WebSocket to a native subscriber, WS Subscribe sent from a native publisher to WebSocket, and federation sent from broker A to broker B.
- Each throughput trial sent for 1 second and drained for 100 ms. The search ranged from 100 to 750,000 Hz in 100 Hz increments. A trial passed with at most 1% loss after draining and both send-window rates at or above 90% of the target. The selected rate for every path fell below the search ceiling.
- Separate message latency runs sent one message at a time and collected 500 one-way samples. Service and Action each used 8 warm-up calls and 32 serial measured calls. Each Action included one feedback message and the final result. With 32 samples, an unusually slow call can materially affect p99.

## Results across three rounds

Each cell shows the median [minimum–maximum] across rounds. The message receive rate counts unique in-trial receives during the send window. Delivery uses unique in-trial receives after draining divided by sends. RPC rates use successful completions and elapsed call time.

| Transport | Scenario | Receives or completions/s | Delivery or completion (%) | p50 (µs) | p95 (µs) | p99 (µs) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| tcp | message pub/sub | 268,340 [242,848–269,716] | 100.0 [100.0–100.0] | 146 [144–148] | 158 [156–158] | 189 [174–194] |
| tcp | service call | 4,682 [4,420–4,683] | 100.0 [100.0–100.0] | 213 [213–225] | 227 [218–231] | 233 [225–242] |
| tcp | action send_goal | 2,092 [1,363–2,426] | 100.0 [100.0–100.0] | 542 [381–726] | 572 [554–774] | 606 [573–799] |
| ipc | message pub/sub | 374,933 [374,928–374,934] | 100.0 [100.0–100.0] | 109 [106–110] | 115 [112–116] | 149 [126–154] |
| ipc | service call | 5,422 [5,047–5,764] | 100.0 [100.0–100.0] | 183 [172–196] | 192 [177–206] | 208 [199–233] |
| ipc | action send_goal | 3,015 [2,219–3,385] | 100.0 [100.0–100.0] | 296 [288–448] | 462 [320–501] | 481 [356–528] |
| inproc | message pub/sub | 374,950 [374,950–374,951] | 100.0 [100.0–100.0] | 49 [47–51] | 53 [53–58] | 92 [67–93] |
| inproc | service call | 12,751 [12,224–13,461] | 100.0 [100.0–100.0] | 80 [68–81] | 89 [83–94] | 109 [91–161] |
| inproc | action send_goal | 7,283 [7,236–7,314] | 100.0 [100.0–100.0] | 126 [121–131] | 179 [171–215] | 255 [185–296] |
| ws | message Publish | 6,074 [6,074–6,080] | 99.6 [99.6–99.7] | 201 [199–206] | 241 [226–242] | 387 [372–391] |
| ws | message Subscribe | 228,739 [226,563–238,961] | 100.0 [99.3–100.0] | 4,558 [4,534–4,559] | 5,011 [5,010–5,034] | 5,057 [5,052–5,082] |
| ws | service Call | 23 [23–23] | 100.0 [100.0–100.0] | 43,998 [43,992–44,005] | 44,243 [44,208–44,246] | 44,277 [44,224–48,029] |
| ws | action SendGoal | 23 [23–23] | 100.0 [100.0–100.0] | 43,992 [43,991–44,001] | 44,212 [44,199–44,251] | 44,326 [44,217–48,114] |
| federation | message pub/sub | 592,504 [580,645–598,393] | 100.0 [100.0–100.0] | 239 [231–248] | 276 [266–316] | 339 [334–356] |
| federation | service call | 3,122 [2,924–3,820] | 100.0 [100.0–100.0] | 306 [262–339] | 363 [276–371] | 421 [284–432] |
| federation | action send_goal | 3,009 [2,833–3,244] | 100.0 [100.0–100.0] | 331 [303–351] | 348 [340–370] | 377 [375–398] |

## WS counts and Action latency

- A historical WS Subscribe trial reported 100.1% delivery. The old counter reset at trial boundaries while messages from the previous trial could still be queued. The historical logs lack per-message identifiers. The new benchmark attributes messages by trial and sequence. In a controlled WS regression, the current trial counted one send and observed three arrivals consisting of one unique in-trial receive, one cross-trial arrival and one duplicate. In all 77 measured WS throughput trials, unique in-trial receives were at or below sends.
- TCP, IPC and inproc Action were remeasured on the default SDK path. Queueing a reply now wakes the executor instead of waiting out the 250 ms poll timeout. Median p50 across three rounds was 542 µs for TCP, 296 µs for IPC and 126 µs for inproc. The process was pinned to CPUs 32–39, with the same warm-up and sample counts as the other rows, but it ran on the host without the original 8 GiB container memory limit. The three logs are in `bench-results/20261120-2115-executor-wake/`. WebSocket and federation Action do not wait on that poll; the same rerun still measured about 44 ms and 0.34 ms, so those rows keep the original numbers.

See the [performance guide](performance.md) for reproduction commands and the archive location for per-round logs and raw samples.

## Camera and lidar sizes

The same container remeasured pre-encoded CDR payloads of 256 KiB, 640×480 RGB and 1920×1080 RGB at 10 and 30 messages/s. The send window was 3 s and the queue depth was 8. The table below is Robot Bus TCP, IPC and inproc. ROS 2 SHM and UDPv4, including loss on the large UDP frames, are in the [ROS 2 report](ros2-perf-report.md). The combined comparison is in the [performance guide](performance.md#camera-and-lidar-sizes).

| Payload | Target/s | Transport | Sent/s | Received in window/s | Loss after drain (%) | One-at-a-time p50 (µs) | Loaded p50 (µs) | Loaded p99 (µs) | Passed |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 256 KiB | 10 | tcp | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 766 [735–838] | 790 [740–856] | 1,148 [1,052–1,187] | 3/3 |
| 256 KiB | 10 | ipc | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 710 [709–750] | 720 [718–754] | 1,057 [1,042–1,116] | 3/3 |
| 256 KiB | 10 | inproc | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 466 [444–469] | 474 [452–479] | 634 [627–638] | 3/3 |
| 256 KiB | 30 | tcp | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 774 [755–847] | 1,083 [1,046–1,136] | 1,216 [1,107–1,241] | 3/3 |
| 256 KiB | 30 | ipc | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 703 [699–834] | 1,021 [985–1,048] | 1,061 [1,034–1,067] | 3/3 |
| 256 KiB | 30 | inproc | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 461 [458–470] | 587 [568–600] | 615 [606–626] | 3/3 |
| 640×480 RGB | 10 | tcp | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 2,502 [2,450–2,535] | 2,555 [2,520–2,631] | 2,829 [2,767–2,911] | 3/3 |
| 640×480 RGB | 10 | ipc | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 2,770 [2,297–2,801] | 2,744 [2,288–2,804] | 2,755 [2,652–2,805] | 3/3 |
| 640×480 RGB | 10 | inproc | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 1,229 [1,227–1,266] | 1,245 [1,233–1,286] | 1,943 [1,923–1,945] | 3/3 |
| 640×480 RGB | 30 | tcp | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 2,542 [2,532–2,546] | 2,841 [2,836–2,893] | 2,942 [2,924–3,226] | 3/3 |
| 640×480 RGB | 30 | ipc | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 2,461 [2,418–3,410] | 2,774 [2,768–2,797] | 2,840 [2,824–3,055] | 3/3 |
| 640×480 RGB | 30 | inproc | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 1,277 [1,269–1,281] | 1,877 [1,860–1,887] | 1,913 [1,895–1,931] | 3/3 |
| 1920×1080 RGB | 10 | tcp | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 23,255 [19,222–30,236] | 17,568 [16,741–18,546] | 23,577 [19,217–30,498] | 3/3 |
| 1920×1080 RGB | 10 | ipc | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 30,168 [23,369–30,179] | 27,784 [23,578–28,676] | 30,448 [27,289–31,225] | 3/3 |
| 1920×1080 RGB | 10 | inproc | 10 [10–10] | 10 [10–10] | 0.00 [0.00–0.00] | 5,501 [5,498–5,657] | 5,524 [5,505–5,830] | 12,124 [11,772–12,549] | 3/3 |
| 1920×1080 RGB | 30 | tcp | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 23,185 [23,016–23,535] | 18,436 [17,492–19,379] | 23,510 [23,270–23,532] | 3/3 |
| 1920×1080 RGB | 30 | ipc | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 25,237 [23,549–30,266] | 27,805 [27,489–28,127] | 29,979 [28,336–30,271] | 3/3 |
| 1920×1080 RGB | 30 | inproc | 30 [30–30] | 30 [30–30] | 0.00 [0.00–0.00] | 5,548 [5,361–5,689] | 12,005 [11,372–12,487] | 12,063 [11,541–13,112] | 3/3 |

All 18 rows delivered every frame in three rounds. At 1080p and 30 messages/s, about 187 MB/s, one-at-a-time p50 was 23.2 ms on TCP and 5.5 ms on inproc. Logs are in `bench-results/20261120-2132-sensor-payload/`.
