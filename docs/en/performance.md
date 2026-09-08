English | [中文](../zh/performance.md)

# Performance interpretation and reproduction

Repository reports are local experiment records, not cross-platform guarantees or a sufficient basis for ranking Robot Bus against ROS 2. Read throughput and latency together with transport, process layout and reliability constraints.

## Boundaries of the saved results

This is the state reviewed on 2026-09-08, not a new measurement.

| Report | Available information | Gaps and limits |
| --- | --- | --- |
| [Robot Bus](perf-report.md) | Some TCP/inproc/WS 64B single-host results, methodology and commands | Environment records only version and logical CPU count; CPU model, OS, commit, run date, full configuration and repeated-run variation are missing; IPC/service/action have no results |
| [ROS 2](ros2-perf-report.md) | Humble, Fast DDS, SHM/UDP configuration and results | No established evidence of matching hardware, time or resource limits with the bus report; direct speed ratios are unsupported |
| [ROS 2 bridge](ros2-bridge-perf-report.md) | The 2026-08-31 non-execution/build-failure record and methodology | No usable throughput or latency results; rerun with a matching live ROS environment |

`—` means data was not provided, not zero performance or an unsupported feature. The saved record does not fully explain each missing scenario; do not assume it was measured but omitted.

## WS counts require verification

The saved WS Subscribe row reports `sent=151447`, `recv=151595`, `delivery=100.1%`. If each message is expected once within one trial, these counts cannot directly represent successful delivery. Cross-trial messages, mismatched windows or duplicate counting need investigation against raw samples and counting code. This documentation revision neither establishes a root cause nor recalculates results.

WS Publish at approximately 2,659/s and WS Subscribe at approximately 138,793/s represent different directional scenarios, not a paired end-to-end loss measurement. Latency uses separate sampling and is not measured at the listed maximum-throughput load.

Do not use these WS figures for selection decisions until counts and windows are reconciled. Historical tables remain available for traceability.

## Record on a rerun

1. **Environment:** date, Git commit/tag, CPU model/core count, memory, OS/kernel, container/VM and CPU limits, build profile and toolchain. For ROS, add distribution, RMW, DDS and overlays.
2. **Load:** payload type/size, publisher/subscriber counts, process layout, transport, pacing/trial duration, HWM/QoS, warm-up/draining and all environment overrides.
3. **Raw results:** per-trial sent/recv, counting windows, duration, loss/duplicate definitions, latency samples and failure logs. Explain skipped scenarios.
4. **Repeatability:** keep multiple runs of each configuration and report medians and variation, rather than only the best run.
5. **Comparison:** match hardware, resources, payload, process layout, QoS, counting windows and run count; state unavoidable differences.

Validate count consistency before interpreting performance. Receives exceeding sends, inconsistent windows and missing samples require investigation rather than a maximum-sustainable-delivery claim.

## Reproduction entry points

See [contributing](../../CONTRIBUTING.md) for source-build prerequisites. These commands overwrite the matching English/Chinese reports; retain previous results and raw output first.

```bash
just perf
# Message only; this configuration cannot establish service/action performance.
ROBOT_BUS_PERF_ONLY=message cargo run --release --bin robot_bus_perf
```

ROS benches additionally need ROS. Default tasks use a container named `ros2`; do not assume it exists on every machine. Dependencies and optional local execution are documented in the [ROS bench](../../benches/ros2_perf/README.md) and [bridge bench](../../benches/ros2_bridge_perf/README.md).

```bash
just perf-ros2
just perf-ros2-bridge
```

Performance is not a gate for ordinary documentation checks. New performance claims should update raw evidence, environment details and [support status](support.md) together.
