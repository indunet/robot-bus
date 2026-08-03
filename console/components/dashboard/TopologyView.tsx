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
import { Network } from 'lucide-react'
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
      position: prevPos.get(p.id) ?? { x: 48 + col * 320, y: 40 + row * 220 },
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
  const d = data as ProcessNodeData
  return (
    <div className="min-w-[200px] max-w-[260px] rounded-sm border border-bus-cyan/70 bg-[#1a2228] shadow-md text-bus-text font-mono">
      <div className="px-3 py-1.5 border-b border-bus-border bg-bus-cyan/10 text-[12px] font-semibold tracking-wide truncate">
        {d.label}
      </div>

      {d.inputs.length > 0 && (
        <div className="border-b border-bus-border/60 py-1">
          <div className="px-2 pb-0.5 text-[9px] uppercase tracking-wider text-bus-muted">in</div>
          {d.inputs.map((p) => (
            <div key={`in-${p.topic}`} className="relative flex items-center min-h-[22px] pr-2 pl-3">
              <Handle
                type="target"
                position={Position.Left}
                id={portHandleId('in', p.topic)}
                className="!w-2.5 !h-2.5 !bg-bus-green !border !border-[#0e1012] !-left-[5px]"
                title={p.topic}
              />
              <div className="min-w-0 flex-1">
                <div className="text-[11px] text-bus-text truncate" title={p.topic}>
                  {shortTopic(p.topic)}
                </div>
                {p.typeName && (
                  <div className="text-[9px] text-bus-muted truncate" title={p.typeName}>
                    {p.typeName}
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {d.outputs.length > 0 && (
        <div className="py-1">
          <div className="px-2 pb-0.5 text-[9px] uppercase tracking-wider text-bus-muted text-right">
            out
          </div>
          {d.outputs.map((p) => (
            <div key={`out-${p.topic}`} className="relative flex items-center min-h-[22px] pl-2 pr-3 justify-end">
              <div className="min-w-0 text-right">
                <div className="text-[11px] text-bus-cyan truncate" title={p.topic}>
                  {shortTopic(p.topic)}
                </div>
                {p.typeName && (
                  <div className="text-[9px] text-bus-muted truncate" title={p.typeName}>
                    {p.typeName}
                  </div>
                )}
              </div>
              <Handle
                type="source"
                position={Position.Right}
                id={portHandleId('out', p.topic)}
                className="!w-2.5 !h-2.5 !bg-bus-cyan !border !border-[#0e1012] !-right-[5px]"
                title={p.topic}
              />
            </div>
          ))}
        </div>
      )}

      {d.inputs.length === 0 && d.outputs.length === 0 && (
        <div className="px-3 py-2 text-[11px] text-bus-muted">no ports</div>
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
      <p className="px-3 pb-2 font-mono text-[11px] text-bus-muted">{t('topologyHint')}</p>
      <div className="flex-1 min-h-0" style={{ height: 'calc(100vh - 160px)' }}>
        {empty ? (
          <div className="flex items-center justify-center h-full text-bus-muted font-mono text-xs">
            {t('topologyEmpty')}
          </div>
        ) : (
          <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={nodeTypes}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            fitView
            proOptions={{ hideAttribution: true }}
            nodesDraggable
            nodesConnectable={false}
            elementsSelectable
          >
            <Background color="#2a2f35" gap={20} />
            <Controls />
            <MiniMap nodeColor="#00d4ff" maskColor="rgba(0,0,0,0.6)" />
          </ReactFlow>
        )}
      </div>
    </section>
  )
}
