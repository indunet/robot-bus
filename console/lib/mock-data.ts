// ─── robot-bus 前端 mock 数据层 ───────────────────────────────────────────────
// 生产环境替换为真实 REST / SSE / gRPC-Web 调用
// API 基准: GET /api/v1/status  GET /api/v1/topics  SSE /api/v1/events

export type BrokerStatus = 'ONLINE' | 'DEGRADED' | 'OFFLINE'

export interface BrokerInfo {
  status: BrokerStatus
  version: string
  uptime: number         // seconds
  pid: number
  grpcAddr: string
  webAddr: string
  msgBusXSub: string
  msgBusXPub: string
  svcFE: string
  svcBE: string
  actFE: string
  actBE: string
  msgPerSec: number
  bytesPerSec: number
  totalMessages: number
  totalErrors: number
}

export interface TopicInfo {
  name: string
  msgPerSec: number
  bytesPerSec: number
  lastSeen: number       // unix ms
  totalMsgs: number
  sparkline: number[]    // 最近 20 个采样点
  subscribers: number
  publishers: number
}

export interface ServiceInfo {
  name: string
  calls: number
  errors: number
  timeouts: number
  avgLatencyMs: number
  lastCallAt: number
}

export interface ActionInfo {
  name: string
  runs: number
  active: number
  errors: number
  avgDurationMs: number
  lastRunAt: number
}

export type LogLevel = 'INFO' | 'WARN' | 'ERROR' | 'DEBUG'

export interface LogEntry {
  id: string
  ts: number             // unix ms
  level: LogLevel
  source: string
  message: string
}

// ─── 生成随机 sparkline ──────────────────────────────────────────────────────
function rndSparkline(base: number, variance: number, len = 20): number[] {
  let v = base
  return Array.from({ length: len }, () => {
    v = Math.max(0, v + (Math.random() - 0.5) * variance * 2)
    return Math.round(v)
  })
}

// ─── 模拟 Broker 基础状态 ────────────────────────────────────────────────────
export const INITIAL_BROKER: BrokerInfo = {
  status: 'ONLINE',
  version: '0.4.2',
  uptime: 284731,
  pid: 42839,
  grpcAddr: '0.0.0.0:15770',
  webAddr: '0.0.0.0:15780',
  msgBusXSub: 'tcp://*:15551',
  msgBusXPub: 'tcp://*:15552',
  svcFE: 'tcp://*:15561',
  svcBE: 'tcp://*:15562',
  actFE: 'tcp://*:15571',
  actBE: 'tcp://*:15572',
  msgPerSec: 1247,
  bytesPerSec: 312450,
  totalMessages: 8_432_917,
  totalErrors: 3,
}

// ─── 模拟 Topic 列表 ─────────────────────────────────────────────────────────
export const INITIAL_TOPICS: TopicInfo[] = [
  {
    name: '/robot/base/odom',
    msgPerSec: 50,
    bytesPerSec: 6400,
    lastSeen: Date.now() - 20,
    totalMsgs: 1_204_330,
    sparkline: rndSparkline(50, 15),
    subscribers: 3,
    publishers: 1,
  },
  {
    name: '/robot/arm/joint_states',
    msgPerSec: 100,
    bytesPerSec: 24000,
    lastSeen: Date.now() - 10,
    totalMsgs: 2_407_100,
    sparkline: rndSparkline(100, 20),
    subscribers: 2,
    publishers: 1,
  },
  {
    name: '/sensor/lidar/points',
    msgPerSec: 10,
    bytesPerSec: 204800,
    lastSeen: Date.now() - 100,
    totalMsgs: 241_311,
    sparkline: rndSparkline(10, 4),
    subscribers: 4,
    publishers: 1,
  },
  {
    name: '/camera/rgb/image',
    msgPerSec: 30,
    bytesPerSec: 921600,
    lastSeen: Date.now() - 33,
    totalMsgs: 720_800,
    sparkline: rndSparkline(30, 8),
    subscribers: 2,
    publishers: 2,
  },
  {
    name: '/robot/base/cmd_vel',
    msgPerSec: 25,
    bytesPerSec: 600,
    lastSeen: Date.now() - 40,
    totalMsgs: 601_440,
    sparkline: rndSparkline(25, 10),
    subscribers: 1,
    publishers: 3,
  },
  {
    name: '/diagnostics',
    msgPerSec: 5,
    bytesPerSec: 2048,
    lastSeen: Date.now() - 200,
    totalMsgs: 120_341,
    sparkline: rndSparkline(5, 3),
    subscribers: 1,
    publishers: 5,
  },
  {
    name: '/robot/battery/state',
    msgPerSec: 2,
    bytesPerSec: 256,
    lastSeen: Date.now() - 500,
    totalMsgs: 48_130,
    sparkline: rndSparkline(2, 1),
    subscribers: 3,
    publishers: 1,
  },
  {
    name: '/map/costmap',
    msgPerSec: 0,
    bytesPerSec: 0,
    lastSeen: Date.now() - 15000,
    totalMsgs: 9812,
    sparkline: rndSparkline(0, 2),
    subscribers: 2,
    publishers: 0,
  },
]

// ─── 模拟 Service 列表 ───────────────────────────────────────────────────────
export const INITIAL_SERVICES: ServiceInfo[] = [
  { name: '/robot/arm/move_to', calls: 4521, errors: 2, timeouts: 1, avgLatencyMs: 234, lastCallAt: Date.now() - 1200 },
  { name: '/robot/base/dock',   calls: 112,  errors: 0, timeouts: 0, avgLatencyMs: 89,  lastCallAt: Date.now() - 8000 },
  { name: '/system/get_params', calls: 9803, errors: 0, timeouts: 0, avgLatencyMs: 12,  lastCallAt: Date.now() - 300 },
  { name: '/sensor/calibrate',  calls: 44,   errors: 1, timeouts: 1, avgLatencyMs: 1204, lastCallAt: Date.now() - 60000 },
  { name: '/map/clear_costmap', calls: 231,  errors: 0, timeouts: 0, avgLatencyMs: 45,  lastCallAt: Date.now() - 3400 },
]

// ─── 模拟 Action 列表 ────────────────────────────────────────────────────────
export const INITIAL_ACTIONS: ActionInfo[] = [
  { name: '/robot/navigate_to_pose',  runs: 831,  active: 1, errors: 3,  avgDurationMs: 15400, lastRunAt: Date.now() - 2000 },
  { name: '/robot/arm/pick_object',   runs: 321,  active: 0, errors: 12, avgDurationMs: 8200,  lastRunAt: Date.now() - 30000 },
  { name: '/robot/dock_and_charge',   runs: 48,   active: 0, errors: 0,  avgDurationMs: 22000, lastRunAt: Date.now() - 300000 },
  { name: '/system/run_diagnostics',  runs: 1024, active: 0, errors: 1,  avgDurationMs: 5100,  lastRunAt: Date.now() - 900000 },
]

// ─── 模拟事件日志 ────────────────────────────────────────────────────────────
const LOG_TEMPLATES: Omit<LogEntry, 'id' | 'ts'>[] = [
  { level: 'INFO',  source: 'broker',          message: 'MessageBus XSUB/XPUB proxy running on tcp://*:15551↔15552' },
  { level: 'INFO',  source: 'grpc-web',        message: 'gRPC-Web gateway listening on 0.0.0.0:15770' },
  { level: 'DEBUG', source: 'msg-bus',         message: 'Subscriber connected: /robot/base/odom (client=192.168.1.42)' },
  { level: 'INFO',  source: 'svc-gateway',     message: 'ServiceGateway.Call /robot/arm/move_to latency=238ms' },
  { level: 'WARN',  source: 'action-gateway',  message: 'No worker for /sensor/calibrate, queued (timeout=5000ms)' },
  { level: 'DEBUG', source: 'msg-bus',         message: 'Topic /sensor/lidar/points: 204.8 KB/s (1 pub, 4 sub)' },
  { level: 'INFO',  source: 'broker',          message: 'Heartbeat OK — uptime=3d 7h 5m 31s, msg/s=1247' },
  { level: 'ERROR', source: 'svc-gateway',     message: '/sensor/calibrate timeout after 5000ms — no worker responded' },
  { level: 'INFO',  source: 'action-gateway',  message: 'Action /robot/navigate_to_pose started, goal_id=nav-0x3f1a' },
  { level: 'DEBUG', source: 'msg-bus',         message: 'Publisher disconnected: /map/costmap (idle > 15s)' },
  { level: 'WARN',  source: 'metrics',         message: 'Ring buffer 80% full, consider increasing --metrics-ring-size' },
  { level: 'INFO',  source: 'action-gateway',  message: 'Action /robot/navigate_to_pose succeeded, result=reached' },
  { level: 'INFO',  source: 'broker',          message: 'WebConsole client connected from 192.168.1.100' },
]

export function generateInitialLogs(count = 40): LogEntry[] {
  const logs: LogEntry[] = []
  let ts = Date.now() - count * 3000
  for (let i = 0; i < count; i++) {
    const template = LOG_TEMPLATES[Math.floor(Math.random() * LOG_TEMPLATES.length)]
    logs.push({ ...template, id: `log-${i}`, ts })
    ts += Math.floor(Math.random() * 6000) + 500
  }
  return logs
}

// ─── 格式化辅助 ──────────────────────────────────────────────────────────────
export function fmtUptime(s: number): string {
  const d = Math.floor(s / 86400)
  const h = Math.floor((s % 86400) / 3600)
  const m = Math.floor((s % 3600) / 60)
  const ss = s % 60
  if (d > 0) return `${d}d ${h}h ${m}m ${ss}s`
  if (h > 0) return `${h}h ${m}m ${ss}s`
  return `${m}m ${ss}s`
}

export function fmtBytes(b: number): string {
  if (b >= 1_048_576) return `${(b / 1_048_576).toFixed(1)} MB`
  if (b >= 1024) return `${(b / 1024).toFixed(1)} KB`
  return `${b} B`
}

export function fmtNum(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return `${n}`
}

export function fmtAge(ts: number): string {
  const age = Date.now() - ts
  if (age < 1000) return `${age}ms ago`
  if (age < 60000) return `${Math.floor(age / 1000)}s ago`
  if (age < 3600000) return `${Math.floor(age / 60000)}m ago`
  return `${Math.floor(age / 3600000)}h ago`
}
