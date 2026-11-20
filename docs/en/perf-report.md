English | [中文](../zh/perf-report.md)

# Robot Bus performance report

The 2026-11-20 single-host retest covered TCP, IPC, inproc, WebSocket and two-broker federation, with three rounds per scenario. The release build used Robot Bus 2.4.0, Rust 1.100.0 and ZeroMQ 4.3.4. The measured source was based on commit `b9b414e` with the archived benchmark revisions applied. See the [performance guide](performance.md) for the shared hardware and container configuration.

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
| tcp | action send_goal | 4 [4–4] | 100.0 [100.0–100.0] | 251,924 [251,915–251,947] | 252,041 [252,032–252,053] | 252,090 [252,069–252,205] |
| ipc | message pub/sub | 374,933 [374,928–374,934] | 100.0 [100.0–100.0] | 109 [106–110] | 115 [112–116] | 149 [126–154] |
| ipc | service call | 5,422 [5,047–5,764] | 100.0 [100.0–100.0] | 183 [172–196] | 192 [177–206] | 208 [199–233] |
| ipc | action send_goal | 4 [4–4] | 100.0 [100.0–100.0] | 251,638 [251,616–251,680] | 251,724 [251,709–251,767] | 274,877 [251,745–289,525] |
| inproc | message pub/sub | 374,950 [374,950–374,951] | 100.0 [100.0–100.0] | 49 [47–51] | 53 [53–58] | 92 [67–93] |
| inproc | service call | 12,751 [12,224–13,461] | 100.0 [100.0–100.0] | 80 [68–81] | 89 [83–94] | 109 [91–161] |
| inproc | action send_goal | 4 [4–4] | 100.0 [100.0–100.0] | 251,126 [251,120–251,185] | 251,187 [251,185–251,240] | 251,192 [251,188–251,263] |
| ws | message Publish | 6,074 [6,074–6,080] | 99.6 [99.6–99.7] | 201 [199–206] | 241 [226–242] | 387 [372–391] |
| ws | message Subscribe | 228,739 [226,563–238,961] | 100.0 [99.3–100.0] | 4,558 [4,534–4,559] | 5,011 [5,010–5,034] | 5,057 [5,052–5,082] |
| ws | service Call | 23 [23–23] | 100.0 [100.0–100.0] | 43,998 [43,992–44,005] | 44,243 [44,208–44,246] | 44,277 [44,224–48,029] |
| ws | action SendGoal | 23 [23–23] | 100.0 [100.0–100.0] | 43,992 [43,991–44,001] | 44,212 [44,199–44,251] | 44,326 [44,217–48,114] |
| federation | message pub/sub | 592,504 [580,645–598,393] | 100.0 [100.0–100.0] | 239 [231–248] | 276 [266–316] | 339 [334–356] |
| federation | service call | 3,122 [2,924–3,820] | 100.0 [100.0–100.0] | 306 [262–339] | 363 [276–371] | 421 [284–432] |
| federation | action send_goal | 3,009 [2,833–3,244] | 100.0 [100.0–100.0] | 331 [303–351] | 348 [340–370] | 377 [375–398] |

## WS counts and Action latency

- A historical WS Subscribe trial reported 100.1% delivery. The old counter reset at trial boundaries while messages from the previous trial could still be queued. The historical logs lack per-message identifiers. The new benchmark attributes messages by trial and sequence. In a controlled WS regression, the current trial counted one send and observed three arrivals consisting of one unique in-trial receive, one cross-trial arrival and one duplicate. In all 77 measured WS throughput trials, unique in-trial receives were at or below sends.
- TCP, IPC and inproc Action latency was about 250 ms with the default SDK executor. In a separate eight-sample TCP diagnostic, adding a no-op 1 ms server timer changed the median from 252.13 ms to 0.84 ms. This is consistent with Action replies waiting for the default 250 ms poll to wake. The table uses the measured rounds with the default executor.

See the [performance guide](performance.md) for reproduction commands and the archive location for per-round logs and raw samples.
