/** Plumbing flow YAML: parse / stringify / validate / launch helpers. */

import { parse as parseYaml, stringify as stringifyYaml } from 'yaml'
import {
  getNodeType,
  topicFromParams,
} from './flow-catalog'
import {
  ACTION_TYPES,
  CAMERA_H264_EXAMPLE,
  SERVICE_TYPES,
  TOPIC_TYPES,
  emptyConfig,
  exportBridgeYaml,
  parseBridgeYaml,
  type ActionRoute,
  type BridgeConfig,
  type RobotBusSection,
  type ServiceRoute,
  type TopicDirection,
  type TopicRoute,
  type SrvActDirection,
  validateConfig as validateBridgeConfig,
} from './bridge-yaml'

export type LiveStatus = 'live' | 'missing' | 'external'

export interface FlowEndpoint {
  node: string
  port: string
}

export interface FlowEdge {
  id: string
  from: FlowEndpoint
  to: FlowEndpoint
  topic: string
}

export interface FlowNode {
  id: string
  type: string
  name: string
  params: Record<string, unknown>
  /** Canvas position (optional; preserved across export when present). */
  position?: { x: number; y: number }
}

export interface FlowConfig {
  version: 1
  robot_bus: RobotBusSection
  nodes: FlowNode[]
  edges: FlowEdge[]
}

function newId(prefix: string): string {
  return `${prefix}-${Math.random().toString(36).slice(2, 10)}`
}

export function emptyFlow(): FlowConfig {
  return {
    version: 1,
    robot_bus: { transport: 'tcp', host: 'localhost' },
    nodes: [],
    edges: [],
  }
}

export function createFlowNode(
  type: string,
  position?: { x: number; y: number },
): FlowNode | null {
  const def = getNodeType(type)
  if (!def) return null
  return {
    id: newId(type),
    type,
    name: def.defaultName,
    params: structuredClone(def.defaultParams),
    position,
  }
}

function asTopicRoutes(raw: unknown): TopicRoute[] {
  if (!Array.isArray(raw)) return []
  return raw.map((r: Record<string, unknown>) => ({
    id: typeof r.id === 'string' ? r.id : newId('route'),
    ros_topic: String(r.ros_topic ?? ''),
    bus_topic: String(r.bus_topic ?? ''),
    type: String(r.type ?? TOPIC_TYPES[0]),
    direction: String(r.direction ?? 'both') as TopicDirection,
  }))
}

function asServiceRoutes(raw: unknown): ServiceRoute[] {
  if (!Array.isArray(raw)) return []
  return raw.map((s: Record<string, unknown>) => ({
    id: typeof s.id === 'string' ? s.id : newId('svc'),
    ros_service: String(s.ros_service ?? ''),
    bus_service: String(s.bus_service ?? ''),
    type: String(s.type ?? SERVICE_TYPES[0]),
    direction: String(s.direction ?? 'ros_to_bus') as SrvActDirection,
  }))
}

function asActionRoutes(raw: unknown): ActionRoute[] {
  if (!Array.isArray(raw)) return []
  return raw.map((a: Record<string, unknown>) => ({
    id: typeof a.id === 'string' ? a.id : newId('act'),
    ros_action: String(a.ros_action ?? ''),
    bus_action: String(a.bus_action ?? ''),
    type: String(a.type ?? ACTION_TYPES[0]),
    direction: String(a.direction ?? 'ros_to_bus') as SrvActDirection,
  }))
}

export function bridgeParamsFromConfig(cfg: BridgeConfig): Record<string, unknown> {
  return {
    routes: cfg.routes,
    services: cfg.services,
    actions: cfg.actions,
  }
}

export function bridgeConfigFromNode(
  node: FlowNode,
  robotBus: RobotBusSection,
): BridgeConfig {
  return {
    robot_bus: robotBus,
    routes: asTopicRoutes(node.params.routes),
    services: asServiceRoutes(node.params.services),
    actions: asActionRoutes(node.params.actions),
  }
}

/** Dynamic bus-side ports for a ros2_bridge node from its routes. */
export function bridgeBusPorts(
  node: FlowNode,
): { id: string; direction: 'in' | 'out'; label: string; topic: string; routeId: string }[] {
  const routes = asTopicRoutes(node.params.routes)
  const ports: {
    id: string
    direction: 'in' | 'out'
    label: string
    topic: string
    routeId: string
  }[] = []
  for (const r of routes) {
    const topic = r.bus_topic.trim()
    if (!topic) continue
    if (r.direction === 'ros_to_bus' || r.direction === 'both') {
      ports.push({
        id: `bus_out:${r.id}`,
        direction: 'out',
        label: topic,
        topic,
        routeId: r.id,
      })
    }
    if (r.direction === 'bus_to_ros' || r.direction === 'both') {
      ports.push({
        id: `bus_in:${r.id}`,
        direction: 'in',
        label: topic,
        topic,
        routeId: r.id,
      })
    }
  }
  return ports
}

export function portsForNode(
  node: FlowNode,
): { id: string; direction: 'in' | 'out'; label: string; topic: string; optional?: boolean }[] {
  if (node.type === 'ros2_bridge') {
    return bridgeBusPorts(node).map((p) => ({
      id: p.id,
      direction: p.direction,
      label: p.label,
      topic: p.topic,
    }))
  }
  const def = getNodeType(node.type)
  if (!def) return []
  return def.ports.map((p) => ({
    id: p.id,
    direction: p.direction,
    label: p.label,
    topic: topicFromParams(node.params, p.id),
    optional: p.optional,
  }))
}

export function validateFlow(flow: FlowConfig): string[] {
  const errors: string[] = []
  const ids = new Set<string>()
  const names = new Map<string, string>()

  for (const n of flow.nodes) {
    if (ids.has(n.id)) errors.push(`Duplicate node id ${n.id}`)
    ids.add(n.id)
    const def = getNodeType(n.type)
    if (!def) {
      errors.push(`Unknown node type ${n.type} (${n.id})`)
      continue
    }
    if (!n.name.trim()) errors.push(`Node ${n.id}: name required`)
    const prev = names.get(n.name)
    if (prev) errors.push(`Duplicate node name "${n.name}" (${prev}, ${n.id})`)
    names.set(n.name, n.id)

    if (n.type === 'ros2_bridge') {
      const bridge = bridgeConfigFromNode(n, flow.robot_bus)
      for (const e of validateBridgeConfig(bridge)) {
        errors.push(`Bridge ${n.id}: ${e}`)
      }
    } else {
      for (const p of def.ports) {
        if (p.optional) continue
        const topic = topicFromParams(n.params, p.id)
        if (!topic) errors.push(`Node ${n.name}: ${p.id} required`)
        else if (!topic.startsWith('/')) errors.push(`Node ${n.name}: ${p.id} must start with /`)
      }
      if (n.type === 'webrtc') {
        const img = topicFromParams(n.params, 'image_topic')
        const aud = topicFromParams(n.params, 'audio_topic')
        const data = String(n.params.data_topics ?? '').trim()
        if (!img && !aud && !data) {
          errors.push(`Node ${n.name}: need image_topic, audio_topic, or data_topics`)
        }
      }
    }
  }

  for (const e of flow.edges) {
    const from = flow.nodes.find((n) => n.id === e.from.node)
    const to = flow.nodes.find((n) => n.id === e.to.node)
    if (!from) errors.push(`Edge ${e.id}: missing from node`)
    if (!to) errors.push(`Edge ${e.id}: missing to node`)
    if (!e.topic.trim()) errors.push(`Edge ${e.id}: topic required`)
    else if (!e.topic.startsWith('/')) errors.push(`Edge ${e.id}: topic must start with /`)
  }

  return errors
}

function parseFlowDoc(raw: Record<string, unknown>): FlowConfig {
  const rb = (raw.robot_bus as Record<string, unknown> | undefined) ?? {}
  const nodesRaw = Array.isArray(raw.nodes) ? raw.nodes : []
  const edgesRaw = Array.isArray(raw.edges) ? raw.edges : []

  const nodes: FlowNode[] = nodesRaw.map((n: Record<string, unknown>) => {
    const type = String(n.type ?? '')
    const def = getNodeType(type)
    const paramsRaw =
      n.params && typeof n.params === 'object' && !Array.isArray(n.params)
        ? (n.params as Record<string, unknown>)
        : {}
    let params: Record<string, unknown>
    if (type === 'ros2_bridge') {
      params = {
        routes: asTopicRoutes(paramsRaw.routes),
        services: asServiceRoutes(paramsRaw.services),
        actions: asActionRoutes(paramsRaw.actions),
      }
    } else {
      params = { ...(def?.defaultParams ?? {}), ...paramsRaw }
    }
    const pos = n.position as { x?: number; y?: number } | undefined
    return {
      id: String(n.id ?? newId(type || 'node')),
      type,
      name: String(n.name ?? def?.defaultName ?? type),
      params,
      position:
        pos && typeof pos.x === 'number' && typeof pos.y === 'number'
          ? { x: pos.x, y: pos.y }
          : undefined,
    }
  })

  const edges: FlowEdge[] = edgesRaw.map((e: Record<string, unknown>) => {
    const from = (e.from as Record<string, unknown>) ?? {}
    const to = (e.to as Record<string, unknown>) ?? {}
    return {
      id: String(e.id ?? newId('edge')),
      from: { node: String(from.node ?? ''), port: String(from.port ?? '') },
      to: { node: String(to.node ?? ''), port: String(to.port ?? '') },
      topic: String(e.topic ?? ''),
    }
  })

  return {
    version: 1,
    robot_bus: {
      transport: String(rb.transport ?? 'tcp'),
      host: rb.host !== undefined ? String(rb.host) : 'localhost',
      ipc_path: rb.ipc_path !== undefined ? String(rb.ipc_path) : undefined,
      discover: rb.discover as RobotBusSection['discover'],
    },
    nodes,
    edges,
  }
}

function looksLikeLegacyBridge(raw: Record<string, unknown>): boolean {
  if (raw.version !== undefined || Array.isArray(raw.nodes)) return false
  return (
    Array.isArray(raw.routes) ||
    Array.isArray(raw.services) ||
    Array.isArray(raw.actions)
  )
}

export function flowFromBridgeConfig(cfg: BridgeConfig): FlowConfig {
  const node = createFlowNode('ros2_bridge', { x: 320, y: 120 })!
  node.params = bridgeParamsFromConfig(cfg)
  return {
    version: 1,
    robot_bus: cfg.robot_bus,
    nodes: [node],
    edges: [],
  }
}

export function parseFlowYaml(text: string): {
  flow: FlowConfig
  errors: string[]
  upgraded: boolean
} {
  try {
    const raw = parseYaml(text) as Record<string, unknown> | null
    if (!raw || typeof raw !== 'object') {
      return { flow: emptyFlow(), errors: ['YAML root must be a mapping'], upgraded: false }
    }
    if (looksLikeLegacyBridge(raw)) {
      const { config, errors } = parseBridgeYaml(text)
      const flow = flowFromBridgeConfig(config)
      return {
        flow,
        errors: errors.length ? errors : validateFlow(flow),
        upgraded: true,
      }
    }
    const flow = parseFlowDoc(raw)
    return { flow, errors: validateFlow(flow), upgraded: false }
  } catch (err) {
    return {
      flow: emptyFlow(),
      errors: [err instanceof Error ? err.message : 'Failed to parse YAML'],
      upgraded: false,
    }
  }
}

function serializeNodeParams(node: FlowNode): Record<string, unknown> {
  if (node.type === 'ros2_bridge') {
    const routes = asTopicRoutes(node.params.routes).map(
      ({ ros_topic, bus_topic, type, direction }) => ({
        ros_topic,
        bus_topic,
        type,
        direction,
      }),
    )
    const services = asServiceRoutes(node.params.services).map(
      ({ ros_service, bus_service, type, direction }) => ({
        ros_service,
        bus_service,
        type,
        direction,
      }),
    )
    const actions = asActionRoutes(node.params.actions).map(
      ({ ros_action, bus_action, type, direction }) => ({
        ros_action,
        bus_action,
        type,
        direction,
      }),
    )
    return { routes, services, actions }
  }
  return { ...node.params }
}

export function exportFlowYaml(flow: FlowConfig): string {
  const doc: Record<string, unknown> = {
    version: 1,
    robot_bus: {
      transport: flow.robot_bus.transport,
      ...(flow.robot_bus.host !== undefined ? { host: flow.robot_bus.host } : {}),
      ...(flow.robot_bus.ipc_path ? { ipc_path: flow.robot_bus.ipc_path } : {}),
      ...(flow.robot_bus.discover ? { discover: flow.robot_bus.discover } : {}),
    },
    nodes: flow.nodes.map((n) => ({
      id: n.id,
      type: n.type,
      name: n.name,
      ...(n.position ? { position: n.position } : {}),
      params: serializeNodeParams(n),
    })),
    edges: flow.edges.map((e) => ({
      id: e.id,
      from: e.from,
      to: e.to,
      topic: e.topic,
    })),
  }
  return stringifyYaml(doc)
}

/** ros__parameters YAML for a single rbus_* node. */
export function toParamsYaml(node: FlowNode): string | null {
  const def = getNodeType(node.type)
  if (!def || !def.binary) return null
  return stringifyYaml({ ros__parameters: { ...node.params } })
}

/** Standalone Ros2Bridge YAML for a bridge node. */
export function toBridgeYaml(node: FlowNode, robotBus: RobotBusSection): string | null {
  if (node.type !== 'ros2_bridge') return null
  return exportBridgeYaml(bridgeConfigFromNode(node, robotBus))
}

export function toLaunchCommands(flow: FlowConfig): string[] {
  const transport = flow.robot_bus.transport || 'tcp'
  const host = flow.robot_bus.host || 'localhost'
  const lines: string[] = [
    '# Manual launch (v1 — console does not start processes)',
    `# broker transport: ${transport} host: ${host}`,
  ]
  for (const n of flow.nodes) {
    const def = getNodeType(n.type)
    if (!def) continue
    if (def.binary) {
      const paramsFile = `${n.name}.yaml`
      lines.push(`# ${n.type} → ${paramsFile}`)
      lines.push(
        `${def.binary} --name ${n.name} --params ${paramsFile} --transport ${transport} --host ${host}`,
      )
    } else if (n.type === 'ros2_bridge') {
      lines.push(
        `# ros2_bridge → ${n.name}.yaml  (Rust: Ros2Bridge::from_yaml("${n.name}.yaml"))`,
      )
    }
  }
  return lines
}

/** Sync edge topic into both endpoint param fields / bridge routes. */
export function applyEdgeTopic(flow: FlowConfig, edge: FlowEdge): FlowConfig {
  const topic = edge.topic.trim()
  const nodes = flow.nodes.map((n) => {
    if (n.id === edge.from.node) {
      return setPortTopic(n, edge.from.port, topic)
    }
    if (n.id === edge.to.node) {
      return setPortTopic(n, edge.to.port, topic)
    }
    return n
  })
  return {
    ...flow,
    nodes,
    edges: flow.edges.map((e) => (e.id === edge.id ? { ...e, topic } : e)),
  }
}

function setPortTopic(node: FlowNode, port: string, topic: string): FlowNode {
  if (node.type === 'ros2_bridge') {
    const routes = asTopicRoutes(node.params.routes).map((r) => {
      if (port === `bus_out:${r.id}` || port === `bus_in:${r.id}`) {
        return { ...r, bus_topic: topic }
      }
      return r
    })
    return { ...node, params: { ...node.params, routes } }
  }
  return { ...node, params: { ...node.params, [port]: topic } }
}

export function connectPorts(
  flow: FlowConfig,
  sourceNodeId: string,
  sourceHandle: string,
  targetNodeId: string,
  targetHandle: string,
): FlowConfig {
  const source = flow.nodes.find((n) => n.id === sourceNodeId)
  const target = flow.nodes.find((n) => n.id === targetNodeId)
  if (!source || !target) return flow

  const srcPorts = portsForNode(source)
  const tgtPorts = portsForNode(target)
  const sp = srcPorts.find((p) => p.id === sourceHandle && p.direction === 'out')
  const tp = tgtPorts.find((p) => p.id === targetHandle && p.direction === 'in')
  if (!sp || !tp) return flow

  const topic = (sp.topic || tp.topic || '/topic').trim() || '/topic'
  const edge: FlowEdge = {
    id: newId('edge'),
    from: { node: sourceNodeId, port: sourceHandle },
    to: { node: targetNodeId, port: targetHandle },
    topic,
  }

  const edges = flow.edges.filter(
    (e) => !(e.to.node === targetNodeId && e.to.port === targetHandle),
  )
  edges.push(edge)
  return applyEdgeTopic({ ...flow, edges }, edge)
}

export function removeEdge(flow: FlowConfig, edgeId: string): FlowConfig {
  return { ...flow, edges: flow.edges.filter((e) => e.id !== edgeId) }
}

export function updateNode(
  flow: FlowConfig,
  nodeId: string,
  patch: Partial<Pick<FlowNode, 'name' | 'params' | 'position'>>,
): FlowConfig {
  return {
    ...flow,
    nodes: flow.nodes.map((n) => (n.id === nodeId ? { ...n, ...patch } : n)),
  }
}

export function removeNode(flow: FlowConfig, nodeId: string): FlowConfig {
  return {
    ...flow,
    nodes: flow.nodes.filter((n) => n.id !== nodeId),
    edges: flow.edges.filter((e) => e.from.node !== nodeId && e.to.node !== nodeId),
  }
}

export function addNodeToFlow(
  flow: FlowConfig,
  type: string,
  position: { x: number; y: number },
): FlowConfig {
  const node = createFlowNode(type, position)
  if (!node) return flow
  let name = node.name
  const used = new Set(flow.nodes.map((n) => n.name))
  if (used.has(name)) {
    let i = 2
    while (used.has(`${node.name}_${i}`)) i++
    name = `${node.name}_${i}`
  }
  node.name = name
  return { ...flow, nodes: [...flow.nodes, node] }
}

export const CAMERA_PIPELINE_EXAMPLE = `# robot-bus plumbing flow — camera → encoder (+ optional ROS 2 bridge)
version: 1
robot_bus:
  transport: tcp
  host: localhost
nodes:
  - id: cam1
    type: usb_camera
    name: usb_camera
    position: { x: 40, y: 80 }
    params:
      output_topic: /camera/image_raw
      device: ""
      width: 640
      height: 480
      fps: 30
      frame_id: camera
  - id: enc1
    type: image_encoder
    name: image_encoder
    position: { x: 360, y: 80 }
    params:
      input_topic: /camera/image_raw
      output_topic: /camera/video
      codec: h264
      bitrate: 2000000
      gop_size: 30
      fps: 30
      encoder: ""
      width: 0
      height: 0
  - id: bridge1
    type: ros2_bridge
    name: ros2_bridge
    position: { x: 680, y: 40 }
    params:
      routes:
        - ros_topic: /camera/image_raw
          bus_topic: /camera/image_raw
          type: sensor_msgs/msg/Image
          direction: ros_to_bus
        - ros_topic: /camera/video
          bus_topic: /camera/video
          type: foxglove_msgs/msg/CompressedVideo
          direction: bus_to_ros
      services: []
      actions: []
edges:
  - id: e1
    from: { node: cam1, port: output_topic }
    to: { node: enc1, port: input_topic }
    topic: /camera/image_raw
`

export { CAMERA_H264_EXAMPLE, emptyConfig }
export type { BridgeConfig, TopicRoute, ServiceRoute, ActionRoute }
