#!/usr/bin/env python3
"""Validate and summarize the fixed-rate CDR Topic benchmark logs."""

import argparse
import csv
import json
from collections import defaultdict
from datetime import datetime
from math import ceil
from pathlib import Path
from statistics import median

ROUTES = {"bus": ("tcp", "ipc", "inproc"), "shm": ("shm",), "udp": ("udp",)}
RATES = (1000, 10000, 50000)
ROUNDS = (1, 2, 3)
SCENARIO = "message pub/sub"
CDR_LAYOUT = {
    "format": "cdr",
    "data_bytes": 64,
    "prefix": [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0],
}


def need(condition, detail):
    if not condition:
        raise SystemExit(detail)


def read_records(log):
    for line in log.read_text().splitlines():
        if line.startswith("RAW "):
            _, kind, body = line.split(" ", 2)
            yield kind, json.loads(body)


def percentile_us(samples, q):
    return sorted(samples)[ceil(q * len(samples)) - 1] / 1000 if samples else None


def triplet(values, digits=0):
    values = [value for value in values if value is not None]
    if not values:
        return "—"

    def fmt(value):
        return f"{value:,.{digits}f}"

    return f"{fmt(median(values))} [{fmt(min(values))}–{fmt(max(values))}]"


def extract_run(root, row):
    round_id = int(row["round"])
    rate = int(row["rate"])
    workload = row["workload"]
    name = f"round-{round_id}-r{rate}-{workload}"
    need(row["exit"] == "0", f"{name}: benchmark exited {row['exit']}")
    log = root / f"{name}.log"
    records = list(read_records(log))
    layouts = [layout for kind, layout in records if kind == "payload"]
    need(layouts == [CDR_LAYOUT], f"{name}: CDR fixture mismatch")

    metrics = {}
    for route in ROUTES[workload]:
        scenario_records = [
            (kind, data) for kind, data in records
            if data.get("transport") == route and data.get("scenario") == SCENARIO
        ]
        by_kind = dict(scenario_records)
        need(
            len(scenario_records) == 3 and set(by_kind) == {"latency", "trial", "result"},
            f"{name}/{route}: missing or duplicated records",
        )
        latency = by_kind["latency"]
        need(
            latency["sent"] == latency["received"] == len(latency["samples_ns"]) == 500
            and latency["invalid"] == latency["duplicates"] == 0,
            f"{name}/{route}: invalid one-at-a-time latency samples",
        )

        trial = by_kind["trial"]
        sent, in_window, drained = (trial[k] for k in ("sent", "recv_send", "recv_final"))
        duration = (trial["send_end_ns"] - trial["start_ns"]) / 1e9
        samples = trial["samples_ns"]
        expected_samples = ceil(sent / 16)
        need(
            trial["target_hz"] == rate and trial["valid"] and trial["latency_stride"] == 16
            and trial["invalid"] == trial["duplicates"] == 0
            and 2.95 <= duration <= 3.1 and 0 <= in_window <= drained <= sent and sent > 0
            and trial["send_end_ns"] < trial["settled_ns"]
            and len(samples) <= min(drained, expected_samples)
            and (drained != sent or len(samples) == expected_samples),
            f"{name}/{route}: inconsistent fixed-rate trial counts or samples",
        )
        offered_rate, receive_rate = sent / duration, in_window / duration
        loss = 100 * (sent - drained) / sent
        qualifies = drained > 0 and loss <= 1 and min(offered_rate, receive_rate) >= .90 * rate
        need((by_kind["result"]["note"] is None) == qualifies,
             f"{name}/{route}: harness and raw trial disagree about qualification")
        metrics[route] = (
            offered_rate, receive_rate, 100 - loss,
            percentile_us(latency["samples_ns"], .50), percentile_us(samples, .50),
            percentile_us(samples, .99), loss, qualifies,
        )
    need(len(records) == 1 + 3 * len(metrics), f"{name}: unexpected scenarios")
    return metrics, [dict(body, kind=kind, workload=workload, round=round_id, rate=rate)
                     for kind, body in records]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    root = parser.parse_args().directory
    rates_file = root / "rates.txt"
    rates = tuple(sorted(map(int, rates_file.read_text().split()))) if rates_file.exists() else RATES
    need(rates and min(rates) > 0 and len(set(rates)) == len(rates), "invalid recorded test rates")
    with (root / "execution.tsv").open() as file:
        executions = list(csv.DictReader(file, delimiter="\t"))
    run_keys = [(int(row["round"]), int(row["rate"]), row["workload"]) for row in executions]
    expected = [(round_id, rate, workload)
                for round_id in ROUNDS for rate in rates for workload in ROUTES]
    need(sorted(run_keys) == sorted(expected), "run matrix is incomplete or has duplicates")

    values = defaultdict(list)
    archive = []
    previous_end = None
    for row in executions:
        start = datetime.fromisoformat(row["start"])
        end = datetime.fromisoformat(row["end"])
        need(end >= start and (previous_end is None or start >= previous_end),
             "benchmark process windows overlap")
        previous_end = end
        metrics, records = extract_run(root, row)
        udp_out = int(row["udp_out"])
        if row["workload"] == "udp":
            received = next(r["recv_final"] for r in records if r["kind"] == "trial")
            need(udp_out >= max(1, received // 2), f"missing UDP traffic in {row}")
        else:
            need(udp_out == 0, f"unexpected UDP traffic in {row}")
        archive.extend(records)
        for route, metric in metrics.items():
            values[int(row["rate"]), route].append(metric)

    (root / "raw.jsonl").write_text("".join(json.dumps(row) + "\n" for row in archive))
    lines = [
        "| Target/s | Route | Sent/s | Received in window/s | Delivery % |"
        " One-at-a-time p50 µs | Loaded p50 µs | Loaded p99 µs |",
        "| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    throughput_lines = [
        "| Target/s | Route | Sent/s | Received in window/s | Loss after drain (%) | Passed |",
        "| ---: | --- | ---: | ---: | ---: | ---: |",
    ]
    for rate in rates:
        for route in ("tcp", "ipc", "inproc", "shm", "udp"):
            results = values[rate, route]
            need(len(results) == 3, f"missing rounds for {rate}/{route}")
            columns = list(zip(*results))
            cells = [triplet(column, digits)
                     for column, digits in zip(columns, (0, 0, 2, 0, 0, 0))]
            lines.append("| " + " | ".join([f"{rate:,}", route, *cells]) + " |")
            offered_rate, receive_rate, _, _, _, _, loss, passes = columns
            throughput_cells = [
                f"{rate:,}", route, triplet(offered_rate), triplet(receive_rate),
                triplet(loss, 2), f"{sum(passes)}/{len(results)}",
            ]
            throughput_lines.append("| " + " | ".join(throughput_cells) + " |")
    summary = "\n".join(lines) + "\n"
    (root / "summary.md").write_text(summary)
    (root / "throughput.md").write_text("\n".join(throughput_lines) + "\n")
    print(f"Validated {len(executions)} process runs and {sum(map(len, values.values()))} fixed-rate trials.")
    print(summary, end="")


if __name__ == "__main__":
    main()
