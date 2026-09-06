// Types + formatters for the console. Live data comes from `/robot_bus/*` via WsNode.

/** `CONNECTING` = client placeholder before the first status message arrives. */
export type BrokerStatus = 'CONNECTING' | 'ONLINE' | 'DEGRADED' | 'OFFLINE'

export interface BrokerInfo {
  status: BrokerStatus
  version: string
  uptime: number // seconds
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
  svcCallsPerSec: number
  actRunsPerSec: number
  totalMessages: number
  totalErrors: number
}

export interface TopicInfo {
  name: string
  typeName?: string
  msgPerSec: number
  bytesPerSec: number
  lastSeen: number // unix ms
  totalMsgs: number
  sparkline: number[] // recent samples (client-maintained)
  subscribers: number
  publishers: number
}

export interface TopologyNodeInfo {
  id: string
  kind: 'process' | 'topic' | 'service' | 'action' | string
  label: string
  typeName?: string
  msgPerSec?: number
}

export interface TopologyEdgeInfo {
  id: string
  source: string
  target: string
  kind: 'publisher' | 'subscriber' | 'service_client' | 'service_server' | 'action_client' | 'action_server' | string
  topic: string
}

export interface TopologyInfo {
  nodes: TopologyNodeInfo[]
  edges: TopologyEdgeInfo[]
}

export interface ServiceInfo {
  name: string
  calls: number
  callsPerSec: number
  errors: number
  timeouts: number
  avgLatencyMs: number
  lastCallAt: number
  workers: number
}

export interface ActionInfo {
  name: string
  runs: number
  runsPerSec: number
  active: number
  errors: number
  avgDurationMs: number
  lastRunAt: number
}

export interface BridgeRouteInfo {
  kind: string
  direction: string
  rosName: string
  busName: string
  typeName: string
  rosQos: string
  busQos: string
  lazy: boolean
  enabled: boolean
  rx: number
  tx: number
  convertFail: number
  decodeFail: number
  publishFail: number
  lastRxMs: number
  idle: boolean
}

export interface BridgeInfo {
  bridgeId: string
  bridgeName: string
  routes: BridgeRouteInfo[]
}

export type LogLevel = 'INFO' | 'WARN' | 'ERROR' | 'DEBUG'

export interface LogEntry {
  id: string
  ts: number // unix ms
  level: LogLevel
  source: string
  message: string
}

/** Placeholder until the first `/robot_bus/status` message arrives. */
export const EMPTY_BROKER: BrokerInfo = {
  status: 'CONNECTING',
  version: '—',
  uptime: 0,
  pid: 0,
  grpcAddr: '—',
  webAddr: '—',
  msgBusXSub: '—',
  msgBusXPub: '—',
  svcFE: '—',
  svcBE: '—',
  actFE: '—',
  actBE: '—',
  msgPerSec: 0,
  bytesPerSec: 0,
  svcCallsPerSec: 0,
  actRunsPerSec: 0,
  totalMessages: 0,
  totalErrors: 0,
}

/** Display a connectable WebSocket RPC URL (`ws://host:port/ws-rpc`). */
export function formatWsRpcAddr(addr: string): string {
  const raw = addr.trim()
  const path = '/ws-rpc'
  if (!raw || raw === '—') return '—'
  if (raw.startsWith('ws://') || raw.startsWith('wss://')) {
    const trimmed = raw.replace(/\/$/, '')
    if (trimmed.endsWith(path)) return trimmed
    if (trimmed.endsWith('/ws')) return `${trimmed.slice(0, -'/ws'.length)}${path}`
    return `${trimmed}${path}`
  }
  if (raw.startsWith('http://') || raw.startsWith('https://')) {
    const trimmed = raw.replace(/\/$/, '')
    const rest = trimmed.startsWith('https://')
      ? trimmed.slice('https://'.length)
      : trimmed.slice('http://'.length)
    const scheme = trimmed.startsWith('https://') ? 'wss' : 'ws'
    let host = rest
    if (host.endsWith(path)) host = host.slice(0, -path.length)
    else if (host.endsWith('/ws')) host = host.slice(0, -'/ws'.length)
    return `${scheme}://${host}${path}`
  }
  let hostport = raw.replace(/\/ws-rpc\/?$/, '').replace(/\/ws\/?$/, '')
  if (hostport.startsWith('[::]:') || hostport.startsWith('[::0]:')) {
    hostport = `[::1]${hostport.slice(hostport.indexOf(']:') + 1)}`
  } else if (hostport.startsWith('0.0.0.0:')) {
    hostport = `127.0.0.1${hostport.slice('0.0.0.0'.length)}`
  }
  return `ws://${hostport}${path}`
}

export function u64(value: string | number | bigint | undefined | null): number {
  if (value == null || value === '') return 0
  if (typeof value === 'number') return value
  if (typeof value === 'bigint') return Number(value)
  const n = Number(value)
  return Number.isFinite(n) ? n : 0
}

/** Keep client-side sparklines when merging topic snapshots. */
export function mergeTopics(prev: TopicInfo[], next: TopicInfo[]): TopicInfo[] {
  const prevMap = new Map(prev.map((t) => [t.name, t]))
  return next.map((t) => {
    const old = prevMap.get(t.name)
    const spark = old
      ? [...old.sparkline.slice(1), t.msgPerSec]
      : Array.from({ length: 20 }, () => t.msgPerSec)
    return { ...t, sparkline: spark }
  })
}

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

/** Format ops/sec so sub-1 Hz values stay visible (0.20, 0.50, 1.2, 48). */
export function fmtRate(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return '0'
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  if (n >= 100) return n.toFixed(0)
  if (n >= 10) return n.toFixed(1)
  if (n >= 1) return n.toFixed(1)
  return n.toFixed(2)
}

/** Parse protobuf number / string / bigint into a finite float. */
export function f64(value: string | number | bigint | undefined | null): number {
  if (value == null || value === '') return 0
  if (typeof value === 'number') return Number.isFinite(value) ? value : 0
  if (typeof value === 'bigint') return Number(value)
  const n = Number(value)
  return Number.isFinite(n) ? n : 0
}

export function topicIsIdle(lastSeen: number, msgPerSec: number, now = Date.now()): boolean {
  if (!lastSeen && msgPerSec < 0.05) return true
  if (!lastSeen) return msgPerSec < 0.05
  if (msgPerSec < 0.05) return now - lastSeen > 8_000
  const periodMs = 1000 / msgPerSec
  const idleAfter = Math.min(15_000, Math.max(2_500, 2.5 * periodMs))
  return now - lastSeen > idleAfter
}

export function fmtAge(ts: number): string {
  if (!ts) return '—'
  const age = Date.now() - ts
  if (age < 1000) return `${age}ms ago`
  if (age < 60000) return `${Math.floor(age / 1000)}s ago`
  if (age < 3600000) return `${Math.floor(age / 60000)}m ago`
  return `${Math.floor(age / 3600000)}h ago`
}
