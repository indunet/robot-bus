#!/usr/bin/env python3
"""Summarize the camera/lidar-sized CDR topic comparison."""

import csv
import json
from collections import defaultdict
from math import ceil
from pathlib import Path
from statistics import median
import sys

ROUTES = {"bus": ("tcp", "ipc", "inproc"), "shm": ("shm",), "udp": ("udp",)}
LABELS = {
    262144: "256 KiB",
    921600: "640×480 RGB",
    6220800: "1920×1080 RGB",
}


def percentile_us(samples, q):
    if not samples:
        return None
    return sorted(samples)[ceil(q * len(samples)) - 1] / 1000


def triplet(values, digits=0):
    values = [value for value in values if value is not None]
    if not values:
        return "—"

    def fmt(value):
        return f"{value:,.{digits}f}"

    return f"{fmt(median(values))} [{fmt(min(values))}–{fmt(max(values))}]"


def prefix_for(payload):
    prefix = [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    prefix[12:16] = list(payload.to_bytes(4, "little"))
    return prefix


def read_records(log):
    if not log.exists():
        return []
    records = []
    for line in log.read_text(errors="replace").splitlines():
        if line.startswith("RAW "):
            _, kind, body = line.split(" ", 2)
            records.append((kind, json.loads(body)))
    return records


def main():
    root = Path(sys.argv[1])
    samples_expected = int((root / "latency-samples.txt").read_text().strip())
    with (root / "execution.tsv").open() as handle:
        executions = list(csv.DictReader(handle, delimiter="\t"))

    values = defaultdict(list)
    failures = []
    for row in executions:
        payload = int(row["payload"])
        rate = int(row["rate"])
        name = f"round-{row['round']}-p{payload}-r{rate}-{row['workload']}"
        if row["exit"] != "0":
            failures.append(f"{name} exit {row['exit']}")
            for route in ROUTES[row["workload"]]:
                values[payload, rate, route].append(None)
            continue
        records = read_records(root / f"{name}.log")
        layouts = [item for kind, item in records if kind == "payload"]
        expected = {"format": "cdr", "data_bytes": payload, "prefix": prefix_for(payload)}
        if layouts != [expected]:
            failures.append(f"{name} CDR fixture mismatch")
            for route in ROUTES[row["workload"]]:
                values[payload, rate, route].append(None)
            continue
        for route in ROUTES[row["workload"]]:
            chosen = [
                (kind, data) for kind, data in records
                if data.get("transport") == route and data.get("scenario") == "message pub/sub"
            ]
            by_kind = dict(chosen)
            if set(by_kind) != {"latency", "trial", "result"}:
                failures.append(f"{name}/{route} incomplete")
                values[payload, rate, route].append(None)
                continue
            trial = by_kind["trial"]
            sent = trial["sent"]
            in_window = trial["recv_send"]
            drained = trial["recv_final"]
            duration = (trial["send_end_ns"] - trial["start_ns"]) / 1e9
            loss = 100 * (sent - drained) / sent if sent else 100
            offered = sent / duration if duration else 0
            received = in_window / duration if duration else 0
            passed = (
                sent > 0 and drained > 0 and loss <= 1
                and min(offered, received) >= 0.90 * rate
            )
            latency = by_kind["latency"]["samples_ns"]
            loaded = trial["samples_ns"]
            if len(latency) != samples_expected:
                failures.append(f"{name}/{route} latency samples {len(latency)}")
            values[payload, rate, route].append((
                offered, received, 100 - loss, loss,
                percentile_us(latency, 0.50), percentile_us(loaded, 0.50),
                percentile_us(loaded, 0.99), passed,
            ))

    lines = [
        "| 负载 | 目标条数/s | 路径 | 实际发送/s | 窗口内接收/s | 排空后丢失率 (%) | 逐条 p50 (µs) | 负载 p50 (µs) | 负载 p99 (µs) | 达标轮数 |",
        "| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    payloads = [int(line) for line in (root / "payloads.txt").read_text().split()]
    rates = [int(line) for line in (root / "rates.txt").read_text().split()]
    for payload in payloads:
        label = LABELS.get(payload, f"{payload} B")
        for rate in rates:
            for route in ("tcp", "ipc", "inproc", "shm", "udp"):
                rounds = values[payload, rate, route]
                good = [item for item in rounds if item is not None]
                passed = sum(item[-1] for item in good)
                columns = list(zip(*good)) if good else []
                def cell(index, digits):
                    return triplet(columns[index], digits) if columns else "—"
                lines.append(
                    "| " + " | ".join([
                        label, f"{rate}", route,
                        cell(0, 0), cell(1, 0), cell(3, 2),
                        cell(4, 0), cell(5, 0), cell(6, 0),
                        f"{passed}/{len(rounds) or 0}",
                    ]) + " |"
                )
    summary = "\n".join(lines) + "\n"
    (root / "summary.md").write_text(summary)
    if failures:
        (root / "failures.txt").write_text("\n".join(failures) + "\n")
    print(summary, end="")
    if failures:
        print(f"\n{len(failures)} problems recorded in failures.txt", file=sys.stderr)


if __name__ == "__main__":
    main()
