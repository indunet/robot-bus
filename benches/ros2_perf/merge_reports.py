#!/usr/bin/env python3
"""Merge shm/udp ROS 2 perf partials into docs/zh + docs/en reports."""
from __future__ import annotations

import re
import sys
from pathlib import Path


def parse(path: Path):
    text = path.read_text()
    env = []
    if m := re.search(r"## 环境\n\n(.*?)\n## ", text, re.S):
        env = [ln for ln in m.group(1).strip().splitlines() if ln.startswith("- ")]
    rows = {}
    for m in re.finditer(
        r"\| (message pub/sub|service call|action send_goal) \| (\d+) \| (\d+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \|",
        text,
    ):
        rows[m.group(1)] = {
            "sent": m.group(2),
            "recv": m.group(3),
            "elapsed": m.group(4).strip(),
            "pub": m.group(5).strip(),
            "sub": m.group(6).strip(),
            "delivery": m.group(7).strip(),
            "p50": m.group(8).strip(),
            "p95": m.group(9).strip(),
            "p99": m.group(10).strip(),
            "mean": m.group(11).strip(),
        }
    return env, rows


def cell_msg_pub(rows):
    r = rows.get("message pub/sub")
    return "—" if not r else f"{r['pub']}/s"


def cell_msg_sub(rows):
    r = rows.get("message pub/sub")
    return "—" if not r else f"{r['sub']}/s ({r['delivery']}% delivered)"


def cell_rpc(rows, scenario):
    r = rows.get(scenario)
    return "—" if not r else f"{r['sub']}/s"


def detail_table(rows, lang: str) -> str:
    order = ["message pub/sub", "service call", "action send_goal"]
    if lang == "zh":
        hdr = (
            "| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |\n"
            "|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|\n"
        )
    else:
        hdr = (
            "| Scenario | Sent | Recv | Time | Pub/s | Sub/s | Delivery% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |\n"
            "|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|\n"
        )
    body = []
    for name in order:
        r = rows.get(name)
        if not r:
            continue
        body.append(
            f"| {name} | {r['sent']} | {r['recv']} | {r['elapsed']} | {r['pub']} | "
            f"{r['sub']} | {r['delivery']} | {r['p50']} | {r['p95']} | {r['p99']} | {r['mean']} |"
        )
    return hdr + "\n".join(body) + "\n"


def section_block(rows, mode: str, lang: str) -> str:
    titles = {
        ("shm", "zh"): "## shm（Fast DDS Shared Memory）",
        ("udp", "zh"): "## udp（Fast DDS UDPv4，无 SHM）",
        ("shm", "en"): "## shm (Fast DDS Shared Memory)",
        ("udp", "en"): "## udp (Fast DDS UDPv4, no SHM)",
    }
    title = titles[(mode, lang)]
    if not rows:
        return f"{title}\n\n_(missing)_\n"
    return f"{title}\n\n{detail_table(rows, lang)}"


def main() -> int:
    if len(sys.argv) != 5:
        print(
            f"usage: {sys.argv[0]} <shm.partial.md> <udp.partial.md> <out-zh.md> <out-en.md>",
            file=sys.stderr,
        )
        return 2
    shm_path, udp_path, out_zh, out_en = map(Path, sys.argv[1:5])
    env_s, rows_s = parse(shm_path)
    _, rows_u = parse(udp_path)

    zh = [
        "[English](../en/ros2-perf-report.md) | 中文\n",
        "# ROS 2 性能测试报告\n",
        "由 `benches/ros2_perf/run.sh`（容器内 `ros2_perf_bench`）生成，方法对齐 [`perf-report.md`](perf-report.md)。\n",
        "## 环境\n",
    ]
    for ln in env_s:
        if ln.startswith("- Mode:"):
            continue
        zh.append(ln)
    zh.extend(
        [
            "- Modes: **shm** (Fast DDS Shared Memory) + **udp** (Fast DDS UDPv4 only)\n",
            "## 方法\n",
            "- RMW: `rmw_fastrtps_cpp`；传输由 Fast DDS XML 固定为 **SHM** 或 **UDPv4**。",
            "- 单进程多 Node + `MultiThreadedExecutor`（本机回环，非跨机）。",
            "- Payload：64 字节；QoS `KeepLast(2048)` best_effort。",
            "- Message **吞吐（主指标）**：按目标速率限速发送约 1s，**二分搜索**丢包率 ≤ 1% 且发送窗口内 pub/sub 均 ≥90% 目标速率的最大可持续速率（max goodput）。",
            "- Message **延迟**：另做限速抽样（发一条等收到再发）。",
            "- Service / action 延迟：每次 call / send_goal 本地计时。",
            "- 指标机器相关，不作为 CI 门槛。\n",
            "## 横比\n",
            "message 为 **max goodput**（丢包预算内的最大可持续订阅速率）；括号为该档实测投递率。\n",
            "| 场景 | shm | udp |",
            "|------|-----|-----|",
            f"| message 发布 | {cell_msg_pub(rows_s)} | {cell_msg_pub(rows_u)} |",
            f"| message max goodput | {cell_msg_sub(rows_s)} | {cell_msg_sub(rows_u)} |",
            f"| service call | {cell_rpc(rows_s, 'service call')} | {cell_rpc(rows_u, 'service call')} |",
            f"| action send_goal | {cell_rpc(rows_s, 'action send_goal')} | {cell_rpc(rows_u, 'action send_goal')} |",
            "",
            section_block(rows_s, "shm", "zh"),
            "",
            section_block(rows_u, "udp", "zh"),
            "## 复现\n",
            "```bash",
            "./benches/ros2_perf/run.sh",
            "ROS2_PERF_ONLY=message ./benches/ros2_perf/run.sh",
            "```",
        ]
    )

    en = [
        "English | [中文](../zh/ros2-perf-report.md)\n",
        "# ROS 2 performance report\n",
        "Generated by `benches/ros2_perf/run.sh` (`ros2_perf_bench` in container). Method aligned with [`perf-report.md`](perf-report.md).\n",
        "## Environment\n",
    ]
    for ln in env_s:
        if ln.startswith("- Mode:"):
            continue
        en.append(ln)
    en.extend(
        [
            "- Modes: **shm** (Fast DDS Shared Memory) + **udp** (Fast DDS UDPv4 only)\n",
            "## Method\n",
            "- RMW: `rmw_fastrtps_cpp`; transport fixed by Fast DDS XML to **SHM** or **UDPv4**.",
            "- Single process, multiple Nodes + `MultiThreadedExecutor` (localhost loopback, not multi-host).",
            "- Payload: 64 bytes; QoS `KeepLast(2048)` best_effort.",
            "- Message **throughput (primary)**: paced send for ~1s; **binary search** for max sustainable rate (max goodput) with loss ≤ 1% and pub/sub ≥90% of the target rate in the send window.",
            "- Message **latency**: separate paced sample (send one, wait for receive).",
            "- Service / action latency: timed per call / send_goal.",
            "- Numbers are machine-dependent; not CI gates.\n",
            "## Cross-compare\n",
            "message is **max goodput** (max sustainable subscribe rate within the loss budget); parentheses show measured delivery at that rate.\n",
            "| Scenario | shm | udp |",
            "|------|-----|-----|",
            f"| message publish | {cell_msg_pub(rows_s)} | {cell_msg_pub(rows_u)} |",
            f"| message max goodput | {cell_msg_sub(rows_s)} | {cell_msg_sub(rows_u)} |",
            f"| service call | {cell_rpc(rows_s, 'service call')} | {cell_rpc(rows_u, 'service call')} |",
            f"| action send_goal | {cell_rpc(rows_s, 'action send_goal')} | {cell_rpc(rows_u, 'action send_goal')} |",
            "",
            section_block(rows_s, "shm", "en"),
            "",
            section_block(rows_u, "udp", "en"),
            "## Reproduce\n",
            "```bash",
            "./benches/ros2_perf/run.sh",
            "ROS2_PERF_ONLY=message ./benches/ros2_perf/run.sh",
            "```",
        ]
    )

    out_zh.parent.mkdir(parents=True, exist_ok=True)
    out_en.parent.mkdir(parents=True, exist_ok=True)
    out_zh.write_text("\n".join(zh) + "\n")
    out_en.write_text("\n".join(en) + "\n")
    print(f"wrote {out_zh}")
    print(f"wrote {out_en}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
