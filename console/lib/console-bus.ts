import { Node as RobotBusNode, consoleTopics } from 'robot-bus'
import {
  BrokerStatus as BrokerStatusMsg,
  TopicStatsList,
  ServiceStatsList,
  ActionStatsList,
  TopologySnapshot,
  ConsoleEvent,
} from 'robot-bus/robot_bus_interface/msg/v1/console_status'
import {
  EMPTY_BROKER,
  u64,
  type ActionInfo,
  type BrokerInfo,
  type LogEntry,
  type ServiceInfo,
  type TopicInfo,
  type TopologyInfo,
} from '@/lib/mock-data'

export type ConsoleBusHandlers = {
  onStatus: (status: BrokerInfo) => void
  onTopics: (topics: TopicInfo[]) => void
  onServices: (services: ServiceInfo[]) => void
  onActions: (actions: ActionInfo[]) => void
  onTopology: (topology: TopologyInfo) => void
  onEvent: (entry: LogEntry) => void
  onOffline?: () => void
}

function mapStatus(msg: BrokerStatusMsg): BrokerInfo {
  const status = (msg.status || 'ONLINE').toUpperCase()
  return {
    ...EMPTY_BROKER,
    status: status === 'ONLINE' || status === 'DEGRADED' || status === 'OFFLINE' ? status : 'ONLINE',
    version: msg.version || '—',
    uptime: u64(msg.uptime),
    pid: u64(msg.pid),
    grpcAddr: msg.grpcAddr || '—',
    webAddr: msg.webAddr || '—',
    msgBusXSub: msg.msgBusXSub || '—',
    msgBusXPub: msg.msgBusXPub || '—',
    svcFE: msg.svcFe || '—',
    svcBE: msg.svcBe || '—',
    actFE: msg.actFe || '—',
    actBE: msg.actBe || '—',
    msgPerSec: u64(msg.msgPerSec),
    bytesPerSec: u64(msg.bytesPerSec),
    svcCallsPerSec: u64(msg.svcCallsPerSec),
    actRunsPerSec: u64(msg.actRunsPerSec),
    totalMessages: u64(msg.totalMessages),
    totalErrors: u64(msg.totalErrors),
  }
}

function mapTopics(msg: TopicStatsList): TopicInfo[] {
  return (msg.topics ?? []).map((t) => {
    const rate = u64(t.msgPerSec)
    return {
      name: t.name,
      typeName: t.typeName || undefined,
      msgPerSec: rate,
      bytesPerSec: u64(t.bytesPerSec),
      lastSeen: u64(t.lastSeen),
      totalMsgs: u64(t.totalMsgs),
      sparkline: (t.sparkline?.length ? t.sparkline : Array(20).fill(rate)).map((v) => u64(v)),
      subscribers: u64(t.subscribers),
      publishers: u64(t.publishers),
    }
  })
}

function mapServices(msg: ServiceStatsList): ServiceInfo[] {
  return (msg.services ?? []).map((s) => ({
    name: s.name,
    calls: u64(s.calls),
    callsPerSec: u64(s.callsPerSec),
    errors: u64(s.errors),
    timeouts: u64(s.timeouts),
    avgLatencyMs: u64(s.avgLatencyMs),
    lastCallAt: u64(s.lastCallAt),
    workers: u64(s.workers),
  }))
}

function mapActions(msg: ActionStatsList): ActionInfo[] {
  return (msg.actions ?? []).map((a) => ({
    name: a.name,
    runs: u64(a.runs),
    runsPerSec: u64(a.runsPerSec),
    active: u64(a.active),
    errors: u64(a.errors),
    avgDurationMs: u64(a.avgDurationMs),
    lastRunAt: u64(a.lastRunAt),
  }))
}

function mapTopology(msg: TopologySnapshot): TopologyInfo {
  return {
    nodes: (msg.nodes ?? []).map((n) => ({
      id: n.id,
      kind: n.kind,
      label: n.label,
      typeName: n.typeName || undefined,
      msgPerSec: n.msgPerSec != null && n.msgPerSec !== '' ? u64(n.msgPerSec) : undefined,
    })),
    edges: (msg.edges ?? []).map((e) => ({
      id: e.id,
      source: e.source,
      target: e.target,
      kind: e.kind,
      topic: e.topic,
    })),
  }
}

function mapEvent(msg: ConsoleEvent): LogEntry {
  return {
    id: msg.id,
    ts: u64(msg.ts),
    level: (msg.level as LogEntry['level']) || 'INFO',
    source: msg.source,
    message: msg.message,
  }
}

/** Same-origin gRPC-Web URL (single-port console). */
export function resolveBusUrl(): string {
  if (typeof window !== 'undefined' && window.location?.origin) {
    return window.location.origin
  }
  return 'http://127.0.0.1:15770'
}

/**
 * Start a console GrpcNode that subscribes to `/robot_bus/*` system topics.
 * Returns a dispose function.
 */
export function startConsoleBus(handlers: ConsoleBusHandlers): () => void {
  const url = resolveBusUrl()
  const node = RobotBusNode.grpcAt('console_ui', url, { consoleUrl: null })

  node.createSubscription(consoleTopics.STATUS, (_t, msg) => {
    handlers.onStatus(mapStatus(msg))
  }, BrokerStatusMsg)

  node.createSubscription(consoleTopics.TOPICS, (_t, msg) => {
    handlers.onTopics(mapTopics(msg))
  }, TopicStatsList)

  node.createSubscription(consoleTopics.SERVICES, (_t, msg) => {
    handlers.onServices(mapServices(msg))
  }, ServiceStatsList)

  node.createSubscription(consoleTopics.ACTIONS, (_t, msg) => {
    handlers.onActions(mapActions(msg))
  }, ActionStatsList)

  node.createSubscription(consoleTopics.TOPOLOGY, (_t, msg) => {
    handlers.onTopology(mapTopology(msg))
  }, TopologySnapshot)

  node.createSubscription(consoleTopics.EVENTS, (_t, msg) => {
    handlers.onEvent(mapEvent(msg))
  }, ConsoleEvent)

  try {
    node.start()
  } catch {
    handlers.onOffline?.()
  }

  return () => {
    try {
      node.shutdown()
    } catch {
      /* ignore */
    }
  }
}
