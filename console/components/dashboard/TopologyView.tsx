'use client'

import { memo, useEffect, useMemo } from 'react'
import {
  ReactFlow,
  Background,
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
import { Cpu, Network } from 'lucide-react'
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

type ProcessNodeData = {
  label: string
  inputs: Port[]
  outputs: Port[]
}

function portHandleId(dir: 'in' | 'out', topic: string): string {
  return `${dir}:${topic}`
}

function shortTopic(topic: string): string {
  if (topic.length <= 28) return topic
  const parts = topic.split('/').filter(Boolean)
  if (parts.length <= 1) return topic.slice(0, 26) + '…'
  return `…/${parts[parts.length - 1]}`
}

function deriveProcessGraph(topo: TopologyInfo): {
  processes: { id: string; label: string; inputs: Port[]; outputs: Port[] }[]
  wires: { id: string; source: string; target: string; topic: string; typeName?: string; msgPerSec?: number }[]
} {
  const topicMeta = new Map<string, { typeName?: string; msgPerSec?: number }>()
  for (const n of topo.nodes) {
    if (n.kind === 'topic') {
      topicMeta.set(n.label, { typeName: n.typeName, msgPerSec: n.msgPerSec })
    }
  }

  const pubsByTopic = new Map<string, string[]>()
  const subsByTopic = new Map<string, string[]>()
  const processLabels = new Map<string, string>()

  for (const n of topo.nodes) {
    if (n.kind === 'process') processLabels.set(n.id, n.label)
  }

  for (const e of topo.edges) {
    if (e.kind === 'publisher' && e.source.startsWith('node:')) {
      const list = pubsByTopic.get(e.topic) ?? []
      if (!list.includes(e.source)) list.push(e.source)
      pubsByTopic.set(e.topic, list)
      if (!processLabels.has(e.source)) {
        processLabels.set(e.source, e.source.replace(/^node:/, ''))
      }
    } else if (e.kind === 'subscriber' && e.target.startsWith('node:')) {
      const list = subsByTopic.get(e.topic) ?? []
      if (!list.includes(e.target)) list.push(e.target)
      subsByTopic.set(e.topic, list)
      if (!processLabels.has(e.target)) {
        processLabels.set(e.target, e.target.replace(/^node:/, ''))
      }
    }
  }

  const inputsByNode = new Map<string, Port[]>()
  const outputsByNode = new Map<string, Port[]>()

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

  const processes = [...processLabels.entries()]
    .sort((a, b) => a[1].localeCompare(b[1]))
    .map(([id, label]) => ({
      id,
      label,
      inputs: (inputsByNode.get(id) ?? []).sort((a, b) => a.topic.localeCompare(b.topic)),
      outputs: (outputsByNode.get(id) ?? []).sort((a, b) => a.topic.localeCompare(b.topic)),
    }))

  const wires: {
    id: string
    source: string
    target: string
    topic: string
    typeName?: string
    msgPerSec?: number
  }[] = []

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
          topic,
          ...meta,
        })
      }
    }
  }

  return { processes, wires }
}

function layoutProcessNodes(
  processes: { id: string; label: string; inputs: Port[]; outputs: Port[] }[],
  prev: Node[],
): Node[] {
  const prevPos = new Map(prev.map((n) => [n.id, n.position]))
  const cols = Math.max(1, Math.ceil(Math.sqrt(processes.length)))
  return processes.map((p, i) => {
    const col = i % cols
    const row = Math.floor(i / cols)
    return {
      id: p.id,
      type: 'process',
      position: prevPos.get(p.id) ?? { x: 40 + col * 350, y: 36 + row * 190 },
      data: {
        label: p.label,
        inputs: p.inputs,
        outputs: p.outputs,
      } satisfies ProcessNodeData,
      sourcePosition: Position.Right,
      targetPosition: Position.Left,
    }
  })
}

function buildWireEdges(
  wires: { id: string; source: string; target: string; topic: string; msgPerSec?: number }[],
): Edge[] {
  return wires.map((w) => {
    const hot = (w.msgPerSec ?? 0) > 80
    const color = hot ? '#f59e0b' : '#00d4ff'
    return {
      id: w.id,
      source: w.source,
      target: w.target,
      sourceHandle: portHandleId('out', w.topic),
      targetHandle: portHandleId('in', w.topic),
      label: shortTopic(w.topic),
      animated: (w.msgPerSec ?? 0) > 0,
      style: { stroke: color, strokeWidth: 1.5 },
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
  const { t } = useI18n()
  const d = data as ProcessNodeData
  const portCount = d.inputs.length + d.outputs.length

  return (
    <div
      className="relative w-[286px] overflow-hidden border border-bus-cyan/45 bg-[#12171b]/95 shadow-[0_10px_30px_rgba(0,0,0,0.35)] text-bus-text font-mono"
      style={{
        clipPath:
          'polygon(0 0, calc(100% - 12px) 0, 100% 12px, 100% 100%, 8px 100%, 0 calc(100% - 8px))',
      }}
    >
      <span className="pointer-events-none absolute left-0 top-0 z-10 h-[2px] w-14 bg-bus-cyan" />
      <span className="pointer-events-none absolute bottom-0 right-4 z-10 h-px w-9 bg-bus-cyan/70" />

      <div className="flex items-center gap-2 border-b border-bus-border bg-gradient-to-r from-bus-cyan/15 to-transparent px-2.5 py-2">
        <div className="flex h-7 w-7 shrink-0 items-center justify-center border border-bus-cyan/35 bg-bus-cyan/10 text-bus-cyan">
          <Cpu size={14} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="text-[8px] uppercase tracking-[0.2em] text-bus-cyan/65">
            {t('topologyNode')}
          </div>
          <div className="truncate text-[11px] font-semibold tracking-wide" title={d.label}>
            {d.label}
          </div>
        </div>
        <span className="flex h-5 min-w-5 items-center justify-center border border-bus-border bg-black/20 px-1 text-[9px] text-bus-muted">
          {portCount}
        </span>
      </div>

      {d.inputs.length > 0 && (
        <div className="border-b border-bus-border/60 py-1.5">
          <div className="px-2.5 pb-1 text-[8px] uppercase tracking-[0.18em] text-bus-green/70">
            {t('topologyInput')}
          </div>
          {d.inputs.map((p) => (
            <div
              key={`in-${p.topic}`}
              className="relative mx-1.5 grid min-h-[26px] grid-cols-[minmax(0,1fr)_96px_34px] items-center gap-2 border-l-2 border-bus-green/35 bg-white/[0.025] py-1 pl-2 pr-1.5"
            >
              <Handle
                type="target"
                position={Position.Left}
                id={portHandleId('in', p.topic)}
                className="!-left-[11px] !h-2 !w-2 !border !border-[#0e1012] !bg-bus-green"
                title={p.topic}
              />
              <div className="truncate text-[10px] font-medium text-bus-text" title={p.topic}>
                {shortTopic(p.topic)}
              </div>
              <div className="truncate text-[8px] text-[#647786]" title={p.typeName}>
                {p.typeName ?? '—'}
              </div>
              <span className="text-right text-[8px] tabular-nums text-bus-muted">
                {Math.round(p.msgPerSec ?? 0)}/s
              </span>
            </div>
          ))}
        </div>
      )}

      {d.outputs.length > 0 && (
        <div className="py-1.5">
          <div className="px-2.5 pb-1 text-right text-[8px] uppercase tracking-[0.18em] text-bus-cyan/70">
            {t('topologyOutput')}
          </div>
          {d.outputs.map((p) => (
            <div
              key={`out-${p.topic}`}
              className="relative mx-1.5 grid min-h-[26px] grid-cols-[minmax(0,1fr)_96px_34px] items-center gap-2 border-r-2 border-bus-cyan/35 bg-bus-cyan/[0.035] py-1 pl-1.5 pr-2"
            >
              <div className="truncate text-[10px] font-medium text-bus-cyan" title={p.topic}>
                {shortTopic(p.topic)}
              </div>
              <div className="truncate text-[8px] text-[#647786]" title={p.typeName}>
                {p.typeName ?? '—'}
              </div>
              <span className="text-right text-[8px] tabular-nums text-bus-muted">
                {Math.round(p.msgPerSec ?? 0)}/s
              </span>
              <Handle
                type="source"
                position={Position.Right}
                id={portHandleId('out', p.topic)}
                className="!-right-[11px] !h-2 !w-2 !border !border-[#0e1012] !bg-bus-cyan"
                title={p.topic}
              />
            </div>
          ))}
        </div>
      )}

      {d.inputs.length === 0 && d.outputs.length === 0 && (
        <div className="px-3 py-3 text-[10px] text-bus-muted">{t('topologyNoPorts')}</div>
      )}
    </div>
  )
})

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
        ]),
        wires: graph.wires.map((w) => [w.id, w.msgPerSec]),
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
      <div className="flex-1 min-h-0" style={{ height: 'calc(100vh - 160px)' }}>
        {empty ? (
          <div className="flex items-center justify-center h-full text-bus-muted font-mono text-xs">
            {t('topologyEmpty')}
          </div>
        ) : (
          <ReactFlow
            className="robot-bus-flow"
            nodes={nodes}
            edges={edges}
            nodeTypes={nodeTypes}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            fitView
            fitViewOptions={{ padding: 0.28, maxZoom: 0.72 }}
            minZoom={0.25}
            maxZoom={1.5}
            proOptions={{ hideAttribution: true }}
            nodesDraggable
            nodesConnectable={false}
            elementsSelectable
          >
            <Background color="#2a2f35" gap={20} />
            <Controls className="robot-bus-flow-controls" />
            <MiniMap
              className="robot-bus-flow-minimap"
              nodeColor="#007fa0"
              nodeStrokeColor="#00d4ff"
              nodeStrokeWidth={2}
              maskColor="rgba(10, 13, 15, 0.72)"
              pannable
              zoomable
            />
          </ReactFlow>
        )}
      </div>
    </section>
  )
}
