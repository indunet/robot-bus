'use client'

import { memo, useEffect, useMemo } from 'react'
import {
  ReactFlow,
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  Handle,
  MarkerType,
  Position,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
  type NodeProps,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { Hexagon, Network } from 'lucide-react'
import { PanelHeader } from './BrokerOverview'
import { useI18n } from '@/lib/i18n'
import type { TopologyInfo } from '@/lib/mock-data'

interface Props {
  topology: TopologyInfo
}

type Port = {
  topic: string
  typeName?: string
  msgPerSec?: number
}

type RpcPort = {
  name: string
  role: 'server' | 'client'
  msgPerSec?: number
}

type Channel = 'topic' | 'service' | 'action'

type ProcessNodeData = {
  label: string
  inputs: Port[]
  outputs: Port[]
  services: RpcPort[]
  actions: RpcPort[]
}

type Wire = {
  id: string
  source: string
  target: string
  name: string
  channel: Channel
  typeName?: string
  msgPerSec?: number
}

function portHandleId(dir: 'in' | 'out', topic: string): string {
  return `${dir}:${topic}`
}

function rpcHandleId(channel: 'service' | 'action', dir: 'in' | 'out', name: string): string {
  const prefix = channel === 'service' ? 'svc' : 'act'
  return `${prefix}-${dir}:${name}`
}

function shortTopic(topic: string): string {
  if (topic.length <= 28) return topic
  const parts = topic.split('/').filter(Boolean)
  if (parts.length <= 1) return topic.slice(0, 26) + '…'
  return `…/${parts[parts.length - 1]}`
}

function pushUnique(list: string[], id: string): void {
  if (!list.includes(id)) list.push(id)
}

function addRpcPort(map: Map<string, RpcPort[]>, nodeId: string, port: RpcPort): void {
  const ports = map.get(nodeId) ?? []
  if (!ports.some((p) => p.name === port.name && p.role === port.role)) {
    ports.push(port)
  }
  map.set(nodeId, ports)
}

function deriveProcessGraph(topo: TopologyInfo): {
  processes: {
    id: string
    label: string
    inputs: Port[]
    outputs: Port[]
    services: RpcPort[]
    actions: RpcPort[]
  }[]
  wires: Wire[]
} {
  const topicMeta = new Map<string, { typeName?: string; msgPerSec?: number }>()
  const serviceMeta = new Map<string, { msgPerSec?: number }>()
  const actionMeta = new Map<string, { msgPerSec?: number }>()
  for (const n of topo.nodes) {
    if (n.kind === 'topic') {
      topicMeta.set(n.label, { typeName: n.typeName, msgPerSec: n.msgPerSec })
    } else if (n.kind === 'service') {
      serviceMeta.set(n.label, { msgPerSec: n.msgPerSec })
    } else if (n.kind === 'action') {
      actionMeta.set(n.label, { msgPerSec: n.msgPerSec })
    }
  }

  const pubsByTopic = new Map<string, string[]>()
  const subsByTopic = new Map<string, string[]>()
  const svcClients = new Map<string, string[]>()
  const svcServers = new Map<string, string[]>()
  const actClients = new Map<string, string[]>()
  const actServers = new Map<string, string[]>()
  const processLabels = new Map<string, string>()

  for (const n of topo.nodes) {
    if (n.kind === 'process') processLabels.set(n.id, n.label)
  }

  const rememberProcess = (id: string) => {
    if (!processLabels.has(id)) {
      processLabels.set(id, id.replace(/^node:/, ''))
    }
  }

  for (const e of topo.edges) {
    if (e.kind === 'publisher' && e.source.startsWith('node:')) {
      const list = pubsByTopic.get(e.topic) ?? []
      pushUnique(list, e.source)
      pubsByTopic.set(e.topic, list)
      rememberProcess(e.source)
    } else if (e.kind === 'subscriber' && e.target.startsWith('node:')) {
      const list = subsByTopic.get(e.topic) ?? []
      pushUnique(list, e.target)
      subsByTopic.set(e.topic, list)
      rememberProcess(e.target)
    } else if (e.kind === 'service_client' && e.source.startsWith('node:')) {
      const list = svcClients.get(e.topic) ?? []
      pushUnique(list, e.source)
      svcClients.set(e.topic, list)
      rememberProcess(e.source)
    } else if (e.kind === 'service_server' && e.target.startsWith('node:')) {
      const list = svcServers.get(e.topic) ?? []
      pushUnique(list, e.target)
      svcServers.set(e.topic, list)
      rememberProcess(e.target)
    } else if (e.kind === 'action_client' && e.source.startsWith('node:')) {
      const list = actClients.get(e.topic) ?? []
      pushUnique(list, e.source)
      actClients.set(e.topic, list)
      rememberProcess(e.source)
    } else if (e.kind === 'action_server' && e.target.startsWith('node:')) {
      const list = actServers.get(e.topic) ?? []
      pushUnique(list, e.target)
      actServers.set(e.topic, list)
      rememberProcess(e.target)
    }
  }

  const inputsByNode = new Map<string, Port[]>()
  const outputsByNode = new Map<string, Port[]>()
  const servicesByNode = new Map<string, RpcPort[]>()
  const actionsByNode = new Map<string, RpcPort[]>()

  for (const [topic, nodes] of pubsByTopic) {
    const meta = topicMeta.get(topic) ?? {}
    for (const id of nodes) {
      const ports = outputsByNode.get(id) ?? []
      if (!ports.some((p) => p.topic === topic)) {
        ports.push({ topic, ...meta })
      }
      outputsByNode.set(id, ports)
    }
  }
  for (const [topic, nodes] of subsByTopic) {
    const meta = topicMeta.get(topic) ?? {}
    for (const id of nodes) {
      const ports = inputsByNode.get(id) ?? []
      if (!ports.some((p) => p.topic === topic)) {
        ports.push({ topic, ...meta })
      }
      inputsByNode.set(id, ports)
    }
  }
  for (const [name, nodes] of svcServers) {
    const meta = serviceMeta.get(name) ?? {}
    for (const id of nodes) addRpcPort(servicesByNode, id, { name, role: 'server', ...meta })
  }
  for (const [name, nodes] of svcClients) {
    const meta = serviceMeta.get(name) ?? {}
    for (const id of nodes) addRpcPort(servicesByNode, id, { name, role: 'client', ...meta })
  }
  for (const [name, nodes] of actServers) {
    const meta = actionMeta.get(name) ?? {}
    for (const id of nodes) addRpcPort(actionsByNode, id, { name, role: 'server', ...meta })
  }
  for (const [name, nodes] of actClients) {
    const meta = actionMeta.get(name) ?? {}
    for (const id of nodes) addRpcPort(actionsByNode, id, { name, role: 'client', ...meta })
  }

  const sortRpc = (a: RpcPort, b: RpcPort) =>
    a.role.localeCompare(b.role) || a.name.localeCompare(b.name)

  const processes = [...processLabels.entries()]
    .sort((a, b) => a[1].localeCompare(b[1]))
    .map(([id, label]) => ({
      id,
      label,
      inputs: (inputsByNode.get(id) ?? []).sort((a, b) => a.topic.localeCompare(b.topic)),
      outputs: (outputsByNode.get(id) ?? []).sort((a, b) => a.topic.localeCompare(b.topic)),
      services: (servicesByNode.get(id) ?? []).sort(sortRpc),
      actions: (actionsByNode.get(id) ?? []).sort(sortRpc),
    }))

  const wires: Wire[] = []

  for (const [topic, pubs] of pubsByTopic) {
    const subs = subsByTopic.get(topic) ?? []
    const meta = topicMeta.get(topic) ?? {}
    for (const source of pubs) {
      for (const target of subs) {
        if (source === target) continue
        wires.push({
          id: `${source}->${target}:${topic}`,
          source,
          target,
          name: topic,
          channel: 'topic',
          ...meta,
        })
      }
    }
  }

  const pairRpc = (
    clients: Map<string, string[]>,
    servers: Map<string, string[]>,
    meta: Map<string, { msgPerSec?: number }>,
    channel: 'service' | 'action',
  ) => {
    for (const [name, cli] of clients) {
      const srv = servers.get(name) ?? []
      const rate = meta.get(name) ?? {}
      for (const source of cli) {
        for (const target of srv) {
          if (source === target) continue
          wires.push({
            id: `${source}->${target}:${channel}:${name}`,
            source,
            target,
            name,
            channel,
            ...rate,
          })
        }
      }
    }
  }
  pairRpc(svcClients, svcServers, serviceMeta, 'service')
  pairRpc(actClients, actServers, actionMeta, 'action')

  return { processes, wires }
}

function estimateCardHeight(p: {
  inputs: Port[]
  outputs: Port[]
  services: RpcPort[]
  actions: RpcPort[]
}): number {
  const header = 54
  const sectionChrome = 42
  const row = 34
  let h = header
  if (p.inputs.length > 0) h += sectionChrome + p.inputs.length * row
  if (p.outputs.length > 0) h += sectionChrome + p.outputs.length * row
  if (p.services.length > 0) h += sectionChrome + p.services.length * row
  if (p.actions.length > 0) h += sectionChrome + p.actions.length * row
  if (p.inputs.length === 0 && p.outputs.length === 0 && p.services.length === 0 && p.actions.length === 0) {
    h += 48
  }
  return h
}

function layoutProcessNodes(
  processes: {
    id: string
    label: string
    inputs: Port[]
    outputs: Port[]
    services: RpcPort[]
    actions: RpcPort[]
  }[],
  prev: Node[],
): Node[] {
  const prevPos = new Map(prev.map((n) => [n.id, n.position]))
  const cardW = 360
  const gapX = 72
  const gapY = 56
  const cols = Math.max(1, Math.ceil(Math.sqrt(processes.length)))
  const positions: { x: number; y: number }[] = new Array(processes.length)
  let y = 42
  for (let start = 0; start < processes.length; start += cols) {
    const end = Math.min(start + cols, processes.length)
    let rowH = 0
    for (let i = start; i < end; i += 1) {
      rowH = Math.max(rowH, estimateCardHeight(processes[i]))
      positions[i] = { x: 48 + (i - start) * (cardW + gapX), y }
    }
    y += rowH + gapY
  }
  return processes.map((p, i) => ({
    id: p.id,
    type: 'process',
    position: prevPos.get(p.id) ?? positions[i],
    data: {
      label: p.label,
      inputs: p.inputs,
      outputs: p.outputs,
      services: p.services,
      actions: p.actions,
    } satisfies ProcessNodeData,
    sourcePosition: Position.Right,
    targetPosition: Position.Left,
  }))
}

function buildWireEdges(wires: Wire[]): Edge[] {
  return wires.map((w) => {
    const hot = (w.msgPerSec ?? 0) > 80
    const topicColor = hot ? '#f59e0b' : '#00d4ff'
    const color = w.channel === 'service' ? '#f59e0b' : w.channel === 'action' ? '#a78bfa' : topicColor
    const rpc = w.channel !== 'topic'
    return {
      id: w.id,
      source: w.source,
      target: w.target,
      sourceHandle: rpc
        ? rpcHandleId(w.channel, 'out', w.name)
        : portHandleId('out', w.name),
      targetHandle: rpc
        ? rpcHandleId(w.channel, 'in', w.name)
        : portHandleId('in', w.name),
      label: shortTopic(w.name),
      animated: (w.msgPerSec ?? 0) > 0,
      style: {
        stroke: color,
        strokeWidth: 1.5,
        ...(rpc ? { strokeDasharray: '6 3' } : {}),
      },
      labelStyle: {
        fill: '#9ca3af',
        fontSize: 10,
        fontFamily: 'ui-monospace, monospace',
      },
      labelBgStyle: { fill: '#0e1012', fillOpacity: 0.85 },
      markerEnd: { type: MarkerType.ArrowClosed, color, width: 16, height: 16 },
    }
  })
}

const ProcessNode = memo(function ProcessNode({ data }: NodeProps) {
  const { t, labelCase } = useI18n()
  const d = data as ProcessNodeData
  const portCount = d.inputs.length + d.outputs.length + d.services.length + d.actions.length

  return (
    <div
      className="relative w-[360px] overflow-hidden border border-bus-cyan/45 bg-[#12171b]/95 shadow-[0_10px_30px_rgba(0,0,0,0.35)] text-bus-text font-mono"
      style={{
        clipPath:
          'polygon(0 0, calc(100% - 12px) 0, 100% 12px, 100% 100%, 8px 100%, 0 calc(100% - 8px))',
      }}
    >
      <span className="pointer-events-none absolute bottom-0 right-4 z-10 h-px w-10 bg-bus-cyan/70" />

      <div className="flex items-center gap-2.5 border-b border-bus-border bg-gradient-to-r from-bus-cyan/15 to-transparent px-3 py-2.5">
        <Hexagon size={26} className="shrink-0 text-bus-cyan" strokeWidth={1.75} />
        <div className="min-w-0 flex-1">
          <div className={`text-[9px] tracking-[0.2em] text-bus-cyan/65 ${labelCase}`}>
            {t('topologyNode')}
          </div>
          <div className="truncate text-[13px] font-semibold tracking-wide" title={d.label}>
            {d.label}
          </div>
        </div>
        <span className="flex h-6 min-w-6 items-center justify-center border border-bus-border bg-black/20 px-1.5 text-[10px] text-bus-muted">
          {portCount}
        </span>
      </div>

      {d.inputs.length > 0 && (
        <div className="border-b border-bus-border/60 py-2">
          <div className={`px-3 pb-1.5 text-[9px] tracking-[0.18em] text-bus-green/70 ${labelCase}`}>
            {t('topologyInput')}
          </div>
          {d.inputs.map((p) => (
            <div
              key={`in-${p.topic}`}
              className="relative mx-2 grid min-h-[30px] grid-cols-[minmax(0,1fr)_112px_40px] items-center gap-2 border-l-2 border-bus-green/35 bg-white/[0.025] py-1.5 pl-2.5 pr-2"
            >
              <Handle
                type="target"
                position={Position.Left}
                id={portHandleId('in', p.topic)}
                className="!-left-[11px] !h-2.5 !w-2.5 !border !border-[#0e1012] !bg-bus-green"
                title={p.topic}
              />
              <div className="truncate text-[11px] font-medium text-bus-text" title={p.topic}>
                {shortTopic(p.topic)}
              </div>
              <div className="truncate text-[9px] text-[#647786]" title={p.typeName}>
                {p.typeName ?? '—'}
              </div>
              <span className="text-right text-[9px] tabular-nums text-bus-muted">
                {Math.round(p.msgPerSec ?? 0)}/s
              </span>
            </div>
          ))}
        </div>
      )}

      {d.outputs.length > 0 && (
        <div className="border-b border-bus-border/60 py-2">
          <div className={`px-3 pb-1.5 text-right text-[9px] tracking-[0.18em] text-bus-cyan/70 ${labelCase}`}>
            {t('topologyOutput')}
          </div>
          {d.outputs.map((p) => (
            <div
              key={`out-${p.topic}`}
              className="relative mx-2 grid min-h-[30px] grid-cols-[minmax(0,1fr)_112px_40px] items-center gap-2 border-r-2 border-bus-cyan/35 bg-bus-cyan/[0.035] py-1.5 pl-2 pr-2.5"
            >
              <div className="truncate text-[11px] font-medium text-bus-cyan" title={p.topic}>
                {shortTopic(p.topic)}
              </div>
              <div className="truncate text-[9px] text-[#647786]" title={p.typeName}>
                {p.typeName ?? '—'}
              </div>
              <span className="text-right text-[9px] tabular-nums text-bus-muted">
                {Math.round(p.msgPerSec ?? 0)}/s
              </span>
              <Handle
                type="source"
                position={Position.Right}
                id={portHandleId('out', p.topic)}
                className="!-right-[11px] !h-2.5 !w-2.5 !border !border-[#0e1012] !bg-bus-cyan"
                title={p.topic}
              />
            </div>
          ))}
        </div>
      )}

      <RpcSection
        title={t('topologyService')}
        titleClass={`text-bus-amber/75 ${labelCase}`}
        ports={d.services}
        channel="service"
        accent="amber"
        srvLabel={t('topologySrv')}
        cliLabel={t('topologyCli')}
      />

      <RpcSection
        title={t('topologyAction')}
        titleClass={`text-violet-300/80 ${labelCase}`}
        ports={d.actions}
        channel="action"
        accent="violet"
        srvLabel={t('topologySrv')}
        cliLabel={t('topologyCli')}
      />

      {portCount === 0 && (
        <div className="px-3 py-4 text-[11px] text-bus-muted">{t('topologyNoPorts')}</div>
      )}
    </div>
  )
})

function RpcSection({
  title,
  titleClass,
  ports,
  channel,
  accent,
  srvLabel,
  cliLabel,
}: {
  title: string
  titleClass: string
  ports: RpcPort[]
  channel: 'service' | 'action'
  accent: 'amber' | 'violet'
  srvLabel: string
  cliLabel: string
}) {
  if (ports.length === 0) return null
  const bar = accent === 'amber' ? 'border-bus-amber/40' : 'border-violet-400/40'
  const handle = accent === 'amber' ? '!bg-bus-amber' : '!bg-violet-400'
  const serverBg = accent === 'amber' ? 'bg-bus-amber/[0.04]' : 'bg-violet-400/[0.04]'
  const clientBg = accent === 'amber' ? 'bg-bus-amber/[0.07]' : 'bg-violet-400/[0.07]'

  return (
    <div className="border-b border-bus-border/60 py-2 last:border-b-0">
      <div className={`px-3 pb-1.5 text-[9px] tracking-[0.18em] ${titleClass}`}>{title}</div>
      {ports.map((p) => {
        const server = p.role === 'server'
        return (
          <div
            key={`${channel}-${p.role}-${p.name}`}
            className={`relative mx-2 grid min-h-[30px] grid-cols-[40px_minmax(0,1fr)_40px] items-center gap-2 py-1.5 ${
              server ? `border-l-2 pl-2.5 pr-2 ${bar} ${serverBg}` : `border-r-2 pl-2 pr-2.5 ${bar} ${clientBg}`
            }`}
          >
            {server && (
              <Handle
                type="target"
                position={Position.Left}
                id={rpcHandleId(channel, 'in', p.name)}
                className={`!-left-[11px] !h-2.5 !w-2.5 !border !border-[#0e1012] ${handle}`}
                title={p.name}
              />
            )}
            <span className="text-[8px] tracking-wide text-bus-muted">{server ? srvLabel : cliLabel}</span>
            <div className="truncate text-[11px] font-medium text-bus-text" title={p.name}>
              {shortTopic(p.name)}
            </div>
            <span className="text-right text-[9px] tabular-nums text-bus-muted">
              {Math.round(p.msgPerSec ?? 0)}/s
            </span>
            {!server && (
              <Handle
                type="source"
                position={Position.Right}
                id={rpcHandleId(channel, 'out', p.name)}
                className={`!-right-[11px] !h-2.5 !w-2.5 !border !border-[#0e1012] ${handle}`}
                title={p.name}
              />
            )}
          </div>
        )
      })}
    </div>
  )
}

const nodeTypes = { process: ProcessNode }

export default function TopologyView({ topology }: Props) {
  const { t } = useI18n()
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([])
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([])

  const graph = useMemo(() => deriveProcessGraph(topology), [topology])

  const signature = useMemo(
    () =>
      JSON.stringify({
        processes: graph.processes.map((p) => [
          p.id,
          p.label,
          p.inputs.map((x) => [x.topic, x.typeName, x.msgPerSec]),
          p.outputs.map((x) => [x.topic, x.typeName, x.msgPerSec]),
          p.services.map((x) => [x.name, x.role, x.msgPerSec]),
          p.actions.map((x) => [x.name, x.role, x.msgPerSec]),
        ]),
        wires: graph.wires.map((w) => [w.id, w.msgPerSec, w.channel]),
      }),
    [graph],
  )

  useEffect(() => {
    setNodes((prev) => layoutProcessNodes(graph.processes, prev))
    setEdges(buildWireEdges(graph.wires))
  }, [signature, graph, setNodes, setEdges])

  const empty = graph.processes.length === 0

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm flex flex-col h-full min-h-0">
      <PanelHeader
        icon={<Network size={14} />}
        title={t('topologyTitle')}
        sub={t('topologySub', {
          n: graph.processes.length,
          e: graph.wires.length,
        })}
      />
      <div className="relative flex-1 min-h-0" style={{ height: 'calc(100vh - 160px)' }}>
        <ReactFlow
          className="robot-bus-flow"
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          fitView
          fitViewOptions={{ padding: 0.18, maxZoom: 1.05 }}
          minZoom={0.25}
          maxZoom={1.8}
          proOptions={{ hideAttribution: true }}
          nodesDraggable
          nodesConnectable={false}
          elementsSelectable
        >
          <Background
            id="topo-dots"
            variant={BackgroundVariant.Dots}
            gap={18}
            size={1}
            color="rgba(92, 138, 154, 0.12)"
            bgColor="transparent"
          />
          <Background
            id="topo-minor"
            variant={BackgroundVariant.Lines}
            gap={36}
            size={1}
            color="rgba(0, 212, 255, 0.022)"
          />
          <Background
            id="topo-major"
            variant={BackgroundVariant.Lines}
            gap={144}
            size={1}
            color="rgba(0, 212, 255, 0.045)"
          />
          <Controls className="robot-bus-flow-controls" />
          {!empty && (
            <MiniMap
              className="robot-bus-flow-minimap"
              nodeColor="#007fa0"
              nodeStrokeColor="#00d4ff"
              nodeStrokeWidth={2}
              maskColor="rgba(10, 13, 15, 0.72)"
              pannable
              zoomable
            />
          )}
        </ReactFlow>
        {empty && (
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center font-mono text-xs text-bus-muted">
            {t('topologyEmpty')}
          </div>
        )}
      </div>
    </section>
  )
}
