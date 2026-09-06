import { Node as RobotBusNode, consoleTopics } from 'robot-bus'
import {
  BrokerStatus as BrokerStatusMsg,
  TopicStatsList,
  ServiceStatsList,
  ActionStatsList,
  TopologySnapshot,
  ConsoleEvent,
  BridgeSnapshot,
} from 'robot-bus/robot_bus_interfaces/msg/v1/console_status'
import {
  EMPTY_BROKER,
  f64,
  u64,
  type ActionInfo,
  type BridgeInfo,
  type BridgeRouteInfo,
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
  onBridges: (bridges: BridgeInfo[]) => void
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
    msgPerSec: f64(msg.msgPerSec),
    bytesPerSec: u64(msg.bytesPerSec),
    svcCallsPerSec: f64(msg.svcCallsPerSec),
    actRunsPerSec: f64(msg.actRunsPerSec),
    totalMessages: u64(msg.totalMessages),
    totalErrors: u64(msg.totalErrors),
  }
}

function mapTopics(msg: TopicStatsList): TopicInfo[] {
  return (msg.topics ?? []).map((t) => {
    const rate = f64(t.msgPerSec)
    return {
      name: t.name,
      typeName: t.typeName || undefined,
      msgPerSec: rate,
      bytesPerSec: u64(t.bytesPerSec),
      lastSeen: u64(t.lastSeen),
      totalMsgs: u64(t.totalMsgs),
      sparkline: (t.sparkline?.length ? t.sparkline : Array(20).fill(rate)).map((v) => f64(v)),
      subscribers: u64(t.subscribers),
      publishers: u64(t.publishers),
    }
  })
}

function mapServices(msg: ServiceStatsList): ServiceInfo[] {
  return (msg.services ?? []).map((s) => ({
    name: s.name,
    calls: u64(s.calls),
    callsPerSec: f64(s.callsPerSec),
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
    runsPerSec: f64(a.runsPerSec),
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
      msgPerSec: n.msgPerSec != null ? f64(n.msgPerSec) : undefined,
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

function mapBridge(msg: BridgeSnapshot): BridgeInfo {
  return {
    bridgeId: msg.bridgeId || msg.bridgeName || 'bridge',
    bridgeName: msg.bridgeName || msg.bridgeId || 'bridge',
    routes: (msg.routes ?? []).map(
      (r): BridgeRouteInfo => ({
        kind: r.kind,
        direction: r.direction,
        rosName: r.rosName,
        busName: r.busName,
        typeName: r.typeName,
        rosQos: r.rosQos,
        busQos: r.busQos,
        lazy: !!r.lazy,
        enabled: !!r.enabled,
        rx: u64(r.rx),
        tx: u64(r.tx),
        convertFail: u64(r.convertFail),
        decodeFail: u64(r.decodeFail),
        publishFail: u64(r.publishFail),
        lastRxMs: u64(r.lastRxMs),
        idle: !!r.idle,
      }),
    ),
  }
}

const BRIDGE_TTL_MS = 3000

const DEFAULT_BROKER_PORT = '15570'
/** Next.js `pnpm dev` ports — browser WS RPC should hit the broker directly. */
const DEV_UI_PORTS = new Set(['3000', '3020'])

/**
 * Base URL for WebSocket RPC + console REST.
 *
 * - Embedded console on the broker port → same origin (SDK maps to `ws://…/ws`).
 * - `pnpm dev` on :3020/:3000 → talk to the broker directly (same hostname, port 15570).
 */
export function resolveBusUrl(): string {
  if (typeof window !== 'undefined' && window.location?.hostname) {
    const override = process.env.NEXT_PUBLIC_ROBOT_BUS_BROKER_URL
    if (override) return override.replace(/\/$/, '')

    const { protocol, hostname, port } = window.location
    if (DEV_UI_PORTS.has(port || '')) {
      return `${protocol}//${hostname}:${DEFAULT_BROKER_PORT}`
    }
    return window.location.origin
  }
  return `http://127.0.0.1:${DEFAULT_BROKER_PORT}`
}

/**
 * Start a console WsNode that subscribes to `/robot_bus/*` system topics.
 * Returns a dispose function.
 */
export function startConsoleBus(handlers: ConsoleBusHandlers): () => void {
  const url = resolveBusUrl()
  // Topology + topic-type registration enabled so console subscriptions show in
  // Pub/Sub counts and fill TYPE for `/robot_bus/*` system topics.
  const node = RobotBusNode.wsAt('console_ui', url)

  node.createSubscription(consoleTopics.STATUS, (msg) => {
    handlers.onStatus(mapStatus(msg))
  }, BrokerStatusMsg)

  node.createSubscription(consoleTopics.TOPICS, (msg) => {
    handlers.onTopics(mapTopics(msg))
  }, TopicStatsList)

  node.createSubscription(consoleTopics.SERVICES, (msg) => {
    handlers.onServices(mapServices(msg))
  }, ServiceStatsList)

  node.createSubscription(consoleTopics.ACTIONS, (msg) => {
    handlers.onActions(mapActions(msg))
  }, ActionStatsList)

  node.createSubscription(consoleTopics.TOPOLOGY, (msg) => {
    handlers.onTopology(mapTopology(msg))
  }, TopologySnapshot)

  node.createSubscription(consoleTopics.EVENTS, (msg) => {
    handlers.onEvent(mapEvent(msg))
  }, ConsoleEvent)

  const bridges = new Map<string, { info: BridgeInfo; seenAt: number }>()
  const emitBridges = () => {
    const now = Date.now()
    for (const [id, row] of bridges) {
      if (now - row.seenAt > BRIDGE_TTL_MS) bridges.delete(id)
    }
    handlers.onBridges([...bridges.values()].map((row) => row.info))
  }
  node.createSubscription(consoleTopics.BRIDGES, (msg) => {
    const info = mapBridge(msg)
    bridges.set(info.bridgeId, { info, seenAt: Date.now() })
    emitBridges()
  }, BridgeSnapshot)

  const pruneTimer = setInterval(emitBridges, 1000)

  try {
    node.start()
  } catch {
    handlers.onOffline?.()
  }

  return () => {
    clearInterval(pruneTimer)
    try {
      node.shutdown()
    } catch {
      /* ignore */
    }
  }
}
