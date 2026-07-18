// Types + formatters for the console. Live data comes from `/api/v1/*`.

export type BrokerStatus = 'ONLINE' | 'DEGRADED' | 'OFFLINE'

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
  totalMessages: number
  totalErrors: number
}

export interface TopicInfo {
  name: string
  msgPerSec: number
  bytesPerSec: number
  lastSeen: number // unix ms
  totalMsgs: number
  sparkline: number[] // recent samples (client-maintained)
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
  ts: number // unix ms
  level: LogLevel
  source: string
  message: string
}

/** Placeholder until status loads. */
export const EMPTY_BROKER: BrokerInfo = {
  status: 'OFFLINE',
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
  totalMessages: 0,
  totalErrors: 0,
}

export async function fetchStatus(): Promise<BrokerInfo> {
  const res = await fetch('/api/v1/status')
  if (!res.ok) throw new Error(`status ${res.status}`)
  return res.json()
}

export async function fetchTopics(): Promise<TopicInfo[]> {
  const res = await fetch('/api/v1/topics')
  if (!res.ok) throw new Error(`topics ${res.status}`)
  const body = await res.json()
  return (body.topics ?? []) as TopicInfo[]
}

/** Merge server topic stats into previous rows, preserving client sparklines. */
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

export function fmtAge(ts: number): string {
  if (!ts) return '—'
  const age = Date.now() - ts
  if (age < 1000) return `${age}ms ago`
  if (age < 60000) return `${Math.floor(age / 1000)}s ago`
  if (age < 3600000) return `${Math.floor(age / 60000)}m ago`
  return `${Math.floor(age / 3600000)}h ago`
}
