'use client'

import {
  memo,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type DragEvent,
  type ReactNode,
} from 'react'
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
  type Connection,
  type OnConnect,
  type NodeChange,
  type EdgeChange,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import {
  Workflow,
  Upload,
  Download,
  Copy,
  Plus,
  Trash2,
  Settings2,
  X,
  Terminal,
} from 'lucide-react'
import { PanelHeader } from './BrokerOverview'
import { useI18n } from '@/lib/i18n'
import type { TopologyInfo } from '@/lib/mock-data'
import { FLOW_NODE_TYPES, getNodeType, shortTopic } from '@/lib/flow-catalog'
import {
  CAMERA_PIPELINE_EXAMPLE,
  addNodeToFlow,
  applyEdgeTopic,
  bridgeConfigFromNode,
  connectPorts,
  emptyFlow,
  exportFlowYaml,
  parseFlowYaml,
  portsForNode,
  removeEdge,
  removeNode,
  toBridgeYaml,
  toLaunchCommands,
  toParamsYaml,
  updateNode,
  validateFlow,
  type ActionRoute,
  type FlowConfig,
  type FlowNode,
  type ServiceRoute,
  type TopicRoute,
} from '@/lib/flow-yaml'
import {
  ACTION_TYPES,
  SERVICE_TYPES,
  SRV_ACT_DIRECTIONS,
  TOPIC_DIRECTIONS,
  TOPIC_TYPES,
  addActionRoute,
  addServiceRoute,
  addTopicRoute,
} from '@/lib/bridge-yaml'

interface Props {
  topology: TopologyInfo
}

type Selected =
  | { kind: 'node'; id: string }
  | { kind: 'edge'; id: string }
  | { kind: 'broker' }
  | null

type PlumbingNodeData = {
  flowNode: FlowNode
  live: 'live' | 'missing' | 'external'
  selected: boolean
  external?: boolean
}

const inputCls =
  'w-full font-mono text-[11px] bg-bus-bg border border-bus-border rounded-sm px-2 py-1.5 text-bus-text focus:outline-none focus:border-bus-cyan'

function dirLabel(direction: string): string {
  if (direction === 'ros_to_bus') return 'ROS → Bus'
  if (direction === 'bus_to_ros') return 'Bus → ROS'
  if (direction === 'both') return 'ROS ↔ Bus'
  return direction
}

function liveProcessNames(topo: TopologyInfo): Set<string> {
  const names = new Set<string>()
  for (const n of topo.nodes) {
    if (n.kind === 'process') {
      names.add(n.label)
      // id is often `node:<name>`
      if (n.id.startsWith('node:')) names.add(n.id.slice(5))
    }
  }
  for (const e of topo.edges) {
    if (e.source.startsWith('node:')) names.add(e.source.slice(5))
    if (e.target.startsWith('node:')) names.add(e.target.slice(5))
  }
  return names
}

function statusColor(live: 'live' | 'missing' | 'external'): string {
  if (live === 'live') return 'border-bus-green/70 bg-bus-green/10 text-bus-green'
  if (live === 'external') return 'border-bus-muted/50 bg-[#1a1e22] text-bus-muted'
  return 'border-bus-amber/70 bg-bus-amber/10 text-bus-amber'
}

const PlumbingNode = memo(function PlumbingNode({ data }: NodeProps) {
  const d = data as PlumbingNodeData
  const n = d.flowNode
  const def = getNodeType(n.type)
  const ports = portsForNode(n)
  const inputs = ports.filter((p) => p.direction === 'in')
  const outputs = ports.filter((p) => p.direction === 'out')
  const border = d.external
    ? 'border-bus-muted/50'
    : d.selected
      ? 'border-bus-cyan'
      : 'border-bus-cyan/50'

  return (
    <div
      className={`min-w-[200px] max-w-[280px] rounded-sm border ${border} bg-[#1a2228] shadow-md text-bus-text font-mono ${
        d.external ? 'opacity-70' : ''
      }`}
    >
      <div className="px-3 py-1.5 border-b border-bus-border flex items-center gap-2">
        <div className="min-w-0 flex-1">
          <div className="text-[12px] font-semibold tracking-wide truncate">{n.name}</div>
          <div className="text-[9px] text-bus-muted truncate">
            {def?.label ?? n.type}
            {d.external ? ' · external' : ''}
          </div>
        </div>
        <span className={`text-[9px] uppercase px-1.5 py-0.5 rounded-sm border ${statusColor(d.live)}`}>
          {d.live}
        </span>
      </div>

      {inputs.length > 0 && (
        <div className="border-b border-bus-border/60 py-1">
          <div className="px-2 pb-0.5 text-[9px] uppercase tracking-wider text-bus-muted">in</div>
          {inputs.map((p) => (
            <div key={`in-${p.id}`} className="relative flex items-center min-h-[22px] pr-2 pl-3">
              <Handle
                type="target"
                position={Position.Left}
                id={p.id}
                className="!w-2.5 !h-2.5 !bg-bus-green !border !border-[#0e1012] !-left-[5px]"
                isConnectable={!d.external}
                title={p.topic || p.id}
              />
              <div className="min-w-0 flex-1">
                <div className="text-[11px] truncate" title={p.topic || p.id}>
                  {shortTopic(p.topic || p.label || p.id)}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {outputs.length > 0 && (
        <div className="py-1">
          <div className="px-2 pb-0.5 text-[9px] uppercase tracking-wider text-bus-muted text-right">
            out
          </div>
          {outputs.map((p) => (
            <div
              key={`out-${p.id}`}
              className="relative flex items-center min-h-[22px] pl-2 pr-3 justify-end"
            >
              <div className="min-w-0 text-right">
                <div className="text-[11px] text-bus-cyan truncate" title={p.topic || p.id}>
                  {shortTopic(p.topic || p.label || p.id)}
                </div>
              </div>
              <Handle
                type="source"
                position={Position.Right}
                id={p.id}
                className="!w-2.5 !h-2.5 !bg-bus-cyan !border !border-[#0e1012] !-right-[5px]"
                isConnectable={!d.external}
                title={p.topic || p.id}
              />
            </div>
          ))}
        </div>
      )}

      {inputs.length === 0 && outputs.length === 0 && (
        <div className="px-3 py-2 text-[11px] text-bus-muted">
          {n.type === 'ros2_bridge' ? 'add topic routes' : 'no ports'}
        </div>
      )}
    </div>
  )
})

const nodeTypes = { plumbing: PlumbingNode }

export default function FlowEditor({ topology }: Props) {
  const { t } = useI18n()
  const [flow, setFlow] = useState<FlowConfig>(() => emptyFlow())
  const [selected, setSelected] = useState<Selected>(null)
  const [status, setStatus] = useState('')
  const [showYaml, setShowYaml] = useState(false)
  const [showLaunch, setShowLaunch] = useState(false)
  const [pasteOpen, setPasteOpen] = useState(false)
  const [pasteText, setPasteText] = useState('')
  const fileRef = useRef<HTMLInputElement>(null)
  const [rfNodes, setRfNodes, onNodesChangeBase] = useNodesState<Node>([])
  const [rfEdges, setRfEdges, onEdgesChangeBase] = useEdgesState<Edge>([])

  const errors = useMemo(() => validateFlow(flow), [flow])
  const liveNames = useMemo(() => liveProcessNames(topology), [topology])

  const selectedNode = useMemo(() => {
    if (selected?.kind !== 'node') return null
    return flow.nodes.find((n) => n.id === selected.id) ?? null
  }, [selected, flow.nodes])

  const selectedEdge = useMemo(() => {
    if (selected?.kind !== 'edge') return null
    return flow.edges.find((e) => e.id === selected.id) ?? null
  }, [selected, flow.edges])

  const yamlPreview = useMemo(() => exportFlowYaml(flow), [flow])
  const launchPreview = useMemo(() => toLaunchCommands(flow).join('\n'), [flow])

  // Rebuild React Flow graph from flow + topology (preserve positions)
  useEffect(() => {
    const flowNameSet = new Set(flow.nodes.map((n) => n.name))
    const selectedId = selected?.kind === 'node' ? selected.id : null

    setRfNodes((prev) => {
      const prevPos = new Map(prev.map((n) => [n.id, n.position]))
      const nodes: Node[] = flow.nodes.map((n, i) => {
        const live: 'live' | 'missing' = liveNames.has(n.name) ? 'live' : 'missing'
        const fallback = { x: 48 + (i % 3) * 300, y: 40 + Math.floor(i / 3) * 220 }
        return {
          id: n.id,
          type: 'plumbing',
          position: prevPos.get(n.id) ?? n.position ?? fallback,
          data: {
            flowNode: n,
            live,
            selected: n.id === selectedId,
          } satisfies PlumbingNodeData,
          sourcePosition: Position.Right,
          targetPosition: Position.Left,
          draggable: true,
          connectable: true,
        }
      })

      let ghostIdx = 0
      for (const name of liveNames) {
        if (flowNameSet.has(name)) continue
        const id = `external:${name}`
        const fallback = {
          x: 48 + (ghostIdx % 3) * 300,
          y: 420 + Math.floor(ghostIdx / 3) * 160,
        }
        nodes.push({
          id,
          type: 'plumbing',
          position: prevPos.get(id) ?? fallback,
          data: {
            flowNode: {
              id,
              type: 'external',
              name,
              params: {},
            },
            live: 'external',
            selected: false,
            external: true,
          } satisfies PlumbingNodeData,
          draggable: true,
          connectable: false,
          selectable: false,
        })
        ghostIdx++
      }
      return nodes
    })

    const edges: Edge[] = flow.edges.map((e) => {
      const live =
        liveNames.has(flow.nodes.find((n) => n.id === e.from.node)?.name ?? '') &&
        liveNames.has(flow.nodes.find((n) => n.id === e.to.node)?.name ?? '')
      const color = live ? '#22c55e' : '#00d4ff'
      return {
        id: e.id,
        source: e.from.node,
        target: e.to.node,
        sourceHandle: e.from.port,
        targetHandle: e.to.port,
        label: shortTopic(e.topic),
        animated: live,
        style: {
          stroke: color,
          strokeWidth: selected?.kind === 'edge' && selected.id === e.id ? 2.5 : 1.5,
        },
        labelStyle: { fill: '#9ca3af', fontSize: 10, fontFamily: 'ui-monospace, monospace' },
        labelBgStyle: { fill: '#0e1012', fillOpacity: 0.85 },
        markerEnd: { type: MarkerType.ArrowClosed, color, width: 16, height: 16 },
      }
    })
    setRfEdges(edges)
  }, [flow, liveNames, selected, setRfNodes, setRfEdges])

  const onNodesChange = useCallback(
    (changes: NodeChange[]) => {
      onNodesChangeBase(changes)
      for (const ch of changes) {
        if (ch.type === 'position' && ch.position && !ch.dragging) {
          const id = ch.id
          if (id.startsWith('external:')) continue
          setFlow((prev) => updateNode(prev, id, { position: ch.position }))
        }
      }
    },
    [onNodesChangeBase],
  )

  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      onEdgesChangeBase(changes)
      for (const ch of changes) {
        if (ch.type === 'remove') {
          setFlow((prev) => removeEdge(prev, ch.id))
          setSelected((s) => (s?.kind === 'edge' && s.id === ch.id ? null : s))
        }
      }
    },
    [onEdgesChangeBase],
  )

  const onConnect: OnConnect = useCallback((conn: Connection) => {
    if (!conn.source || !conn.target || !conn.sourceHandle || !conn.targetHandle) return
    if (conn.source.startsWith('external:') || conn.target.startsWith('external:')) return
    setFlow((prev) =>
      connectPorts(prev, conn.source!, conn.sourceHandle!, conn.target!, conn.targetHandle!),
    )
  }, [])

  const onDrop = useCallback((e: DragEvent) => {
    e.preventDefault()
    const type = e.dataTransfer.getData('application/robot-bus-flow-type')
    if (!type) return
    const bounds = (e.target as HTMLElement).closest('.react-flow')?.getBoundingClientRect()
    const x = bounds ? e.clientX - bounds.left - 100 : 80
    const y = bounds ? e.clientY - bounds.top - 40 : 80
    setFlow((prev) => addNodeToFlow(prev, type, { x, y }))
  }, [])

  const onDragOver = useCallback((e: DragEvent) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'
  }, [])

  const onImportFile = async (file: File) => {
    const text = await file.text()
    const { flow: next, errors: errs, upgraded } = parseFlowYaml(text)
    setFlow(next)
    setSelected(null)
    setStatus(
      errs.length
        ? errs[0]
        : upgraded
          ? t('flowUpgraded')
          : t('flowImported'),
    )
  }

  const onExport = () => {
    const blob = new Blob([exportFlowYaml(flow)], { type: 'text/yaml' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'flow.yaml'
    a.click()
    URL.revokeObjectURL(url)
    setStatus(t('flowExported'))
  }

  const onCopy = async () => {
    try {
      await navigator.clipboard.writeText(exportFlowYaml(flow))
      setStatus(t('flowCopied'))
    } catch {
      setStatus(t('flowCopyFailed'))
    }
  }

  const onCopyLaunch = async () => {
    try {
      await navigator.clipboard.writeText(launchPreview)
      setStatus(t('flowLaunchCopied'))
    } catch {
      setStatus(t('flowCopyFailed'))
    }
  }

  const patchSelectedNode = (patch: Partial<FlowNode>) => {
    if (!selectedNode) return
    setFlow((prev) => updateNode(prev, selectedNode.id, patch))
  }

  const patchBridgeRoutes = (routes: TopicRoute[]) => {
    if (!selectedNode || selectedNode.type !== 'ros2_bridge') return
    patchSelectedNode({ params: { ...selectedNode.params, routes } })
  }

  const patchBridgeServices = (services: ServiceRoute[]) => {
    if (!selectedNode || selectedNode.type !== 'ros2_bridge') return
    patchSelectedNode({ params: { ...selectedNode.params, services } })
  }

  const patchBridgeActions = (actions: ActionRoute[]) => {
    if (!selectedNode || selectedNode.type !== 'ros2_bridge') return
    patchSelectedNode({ params: { ...selectedNode.params, actions } })
  }

  const deleteSelected = () => {
    if (selected?.kind === 'node') {
      setFlow((prev) => removeNode(prev, selected.id))
      setSelected(null)
    } else if (selected?.kind === 'edge') {
      setFlow((prev) => removeEdge(prev, selected.id))
      setSelected(null)
    }
  }

  const bridgeCfg = selectedNode?.type === 'ros2_bridge'
    ? bridgeConfigFromNode(selectedNode, flow.robot_bus)
    : null

  const inspectorOpen = selected !== null

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm flex flex-col h-full min-h-0">
      <PanelHeader
        icon={<Workflow size={14} />}
        title={t('flowTitle')}
        sub={t('flowSub', {
          n: flow.nodes.length,
          e: flow.edges.length,
        })}
      />
      <p className="px-3 pb-2 font-mono text-[11px] text-bus-muted">{t('flowHint')}</p>

      <div className="px-3 pb-2 flex flex-wrap gap-2 items-center">
        <ToolBtn onClick={() => fileRef.current?.click()}>
          <Upload size={12} /> {t('flowImport')}
        </ToolBtn>
        <input
          ref={fileRef}
          type="file"
          accept=".yaml,.yml,text/yaml"
          className="hidden"
          onChange={(e) => {
            const f = e.target.files?.[0]
            if (f) void onImportFile(f)
            e.target.value = ''
          }}
        />
        <ToolBtn onClick={() => setPasteOpen((v) => !v)}>{t('flowPaste')}</ToolBtn>
        <ToolBtn onClick={onExport}>
          <Download size={12} /> {t('flowExport')}
        </ToolBtn>
        <ToolBtn onClick={() => void onCopy()}>
          <Copy size={12} /> {t('flowCopy')}
        </ToolBtn>
        <ToolBtn onClick={() => setShowYaml((v) => !v)}>{t('flowToggleYaml')}</ToolBtn>
        <ToolBtn onClick={() => setShowLaunch((v) => !v)}>
          <Terminal size={12} /> {t('flowLaunch')}
        </ToolBtn>
        <ToolBtn
          onClick={() => {
            setFlow(parseFlowYaml(CAMERA_PIPELINE_EXAMPLE).flow)
            setSelected(null)
            setStatus(t('flowExampleLoaded'))
          }}
        >
          {t('flowExample')}
        </ToolBtn>
        <ToolBtn onClick={() => setSelected({ kind: 'broker' })}>
          <Settings2 size={12} /> {t('flowBroker')}
        </ToolBtn>
        {selected && selected.kind !== 'broker' && (
          <ToolBtn className="text-bus-amber" onClick={deleteSelected}>
            <Trash2 size={12} /> {t('flowRemove')}
          </ToolBtn>
        )}
      </div>

      {pasteOpen && (
        <div className="px-3 pb-2 flex flex-col gap-2">
          <textarea
            className={`${inputCls} h-28`}
            value={pasteText}
            onChange={(e) => setPasteText(e.target.value)}
            placeholder="paste flow.yaml or Ros2Bridge YAML…"
          />
          <ToolBtn
            onClick={() => {
              const { flow: next, errors: errs, upgraded } = parseFlowYaml(pasteText)
              setFlow(next)
              setSelected(null)
              setPasteOpen(false)
              setStatus(
                errs.length ? errs[0] : upgraded ? t('flowUpgraded') : t('flowImported'),
              )
            }}
          >
            {t('flowApplyPaste')}
          </ToolBtn>
        </div>
      )}

      {(status || errors.length > 0) && (
        <div className="px-3 pb-2 font-mono text-[11px]">
          {errors.length > 0 ? (
            <span className="text-bus-amber">{errors.slice(0, 3).join(' · ')}</span>
          ) : (
            <span className="text-bus-cyan">{status}</span>
          )}
        </div>
      )}

      <div className="relative flex-1 min-h-0 border-t border-bus-border flex">
        {/* Palette */}
        <aside className="w-40 shrink-0 border-r border-bus-border bg-[#12161a] p-2 overflow-y-auto">
          <div className="font-mono text-[9px] uppercase tracking-wider text-bus-muted mb-2 px-1">
            {t('flowPalette')}
          </div>
          <div className="flex flex-col gap-1.5">
            {FLOW_NODE_TYPES.map((nt) => (
              <div
                key={nt.type}
                draggable
                onDragStart={(e) => {
                  e.dataTransfer.setData('application/robot-bus-flow-type', nt.type)
                  e.dataTransfer.effectAllowed = 'move'
                }}
                className="cursor-grab active:cursor-grabbing font-mono text-[11px] px-2 py-1.5 border border-bus-border rounded-sm bg-bus-bg text-bus-text hover:border-bus-cyan hover:text-bus-cyan"
                title={nt.binary ?? 'Ros2Bridge::from_yaml'}
              >
                <div className="flex items-center gap-1">
                  <Plus size={10} className="shrink-0 opacity-60" />
                  <span className="truncate">{nt.label}</span>
                </div>
              </div>
            ))}
          </div>
        </aside>

        <div className="relative flex-1 min-h-0 flex flex-col">
          <div
            className="flex-1 min-h-0"
            style={{ height: showYaml || showLaunch ? 'calc(100vh - 420px)' : 'calc(100vh - 260px)' }}
            onDrop={onDrop}
            onDragOver={onDragOver}
          >
            <ReactFlow
              nodes={rfNodes}
              edges={rfEdges}
              nodeTypes={nodeTypes}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onConnect={onConnect}
              onNodeClick={(_, node) => {
                if (node.id.startsWith('external:')) return
                setSelected({ kind: 'node', id: node.id })
              }}
              onEdgeClick={(_, edge) => setSelected({ kind: 'edge', id: edge.id })}
              onPaneClick={() => setSelected(null)}
              fitView
              proOptions={{ hideAttribution: true }}
              nodesConnectable
              nodesDraggable
              elementsSelectable
              deleteKeyCode={['Backspace', 'Delete']}
            >
              <Background color="#2a2f35" gap={20} />
              <Controls />
              <MiniMap nodeColor="#00d4ff" maskColor="rgba(0,0,0,0.6)" />
            </ReactFlow>
          </div>

          {inspectorOpen && (
            <aside className="absolute top-2 right-2 bottom-2 w-80 max-w-[calc(100%-1rem)] border border-bus-border bg-[#12161a]/95 backdrop-blur-sm rounded-sm shadow-lg overflow-y-auto p-3 z-10">
              <div className="flex items-center justify-between mb-3">
                <div className="font-mono text-xs text-bus-muted uppercase tracking-wider">
                  {selected?.kind === 'broker'
                    ? t('flowBroker')
                    : selected?.kind === 'edge'
                      ? t('flowEditEdge')
                      : t('flowInspector')}
                </div>
                <button
                  type="button"
                  className="text-bus-muted hover:text-bus-text p-1"
                  onClick={() => setSelected(null)}
                  aria-label="close"
                >
                  <X size={14} />
                </button>
              </div>

              {selected?.kind === 'broker' && (
                <div className="space-y-3">
                  <Field label={t('routesFieldTransport')}>
                    <select
                      className={inputCls}
                      value={flow.robot_bus.transport}
                      onChange={(e) =>
                        setFlow((f) => ({
                          ...f,
                          robot_bus: { ...f.robot_bus, transport: e.target.value },
                        }))
                      }
                    >
                      <option value="tcp">tcp</option>
                      <option value="ipc">ipc</option>
                      <option value="discover">discover</option>
                    </select>
                  </Field>
                  <Field label={t('routesFieldHost')}>
                    <input
                      className={inputCls}
                      value={flow.robot_bus.host ?? ''}
                      onChange={(e) =>
                        setFlow((f) => ({
                          ...f,
                          robot_bus: { ...f.robot_bus, host: e.target.value },
                        }))
                      }
                    />
                  </Field>
                </div>
              )}

              {selected?.kind === 'edge' && selectedEdge && (
                <div className="space-y-3">
                  <Field label={t('flowFieldTopic')}>
                    <input
                      className={inputCls}
                      value={selectedEdge.topic}
                      onChange={(e) => {
                        const topic = e.target.value
                        setFlow((prev) =>
                          applyEdgeTopic(prev, { ...selectedEdge, topic }),
                        )
                      }}
                    />
                  </Field>
                </div>
              )}

              {selectedNode && selectedNode.type !== 'ros2_bridge' && (
                <div className="space-y-3">
                  <Field label={t('flowFieldName')}>
                    <input
                      className={inputCls}
                      value={selectedNode.name}
                      onChange={(e) => patchSelectedNode({ name: e.target.value })}
                    />
                  </Field>
                  <div className="font-mono text-[10px] text-bus-muted">{selectedNode.type}</div>
                  {(getNodeType(selectedNode.type)?.fields ?? []).map((field) => (
                    <Field key={field.key} label={field.label}>
                      <input
                        className={inputCls}
                        type={field.type === 'number' ? 'number' : 'text'}
                        value={String(selectedNode.params[field.key] ?? '')}
                        onChange={(e) => {
                          const raw = e.target.value
                          const val =
                            field.type === 'number'
                              ? raw === ''
                                ? 0
                                : Number(raw)
                              : raw
                          patchSelectedNode({
                            params: { ...selectedNode.params, [field.key]: val },
                          })
                        }}
                      />
                    </Field>
                  ))}
                  {getNodeType(selectedNode.type)?.binary && (
                    <ToolBtn
                      onClick={() => {
                        const y = toParamsYaml(selectedNode)
                        if (!y) return
                        const blob = new Blob([y], { type: 'text/yaml' })
                        const url = URL.createObjectURL(blob)
                        const a = document.createElement('a')
                        a.href = url
                        a.download = `${selectedNode.name}.yaml`
                        a.click()
                        URL.revokeObjectURL(url)
                        setStatus(t('flowParamsExported'))
                      }}
                    >
                      <Download size={12} /> {t('flowExportParams')}
                    </ToolBtn>
                  )}
                </div>
              )}

              {selectedNode?.type === 'ros2_bridge' && bridgeCfg && (
                <div className="space-y-3">
                  <Field label={t('flowFieldName')}>
                    <input
                      className={inputCls}
                      value={selectedNode.name}
                      onChange={(e) => patchSelectedNode({ name: e.target.value })}
                    />
                  </Field>
                  <div className="flex flex-wrap gap-1">
                    <ToolBtn
                      onClick={() => {
                        const next = addTopicRoute(bridgeCfg)
                        patchBridgeRoutes(next.routes)
                      }}
                    >
                      <Plus size={12} /> {t('routesAddTopic')}
                    </ToolBtn>
                    <ToolBtn
                      onClick={() => {
                        const next = addServiceRoute(bridgeCfg)
                        patchBridgeServices(next.services)
                      }}
                    >
                      <Plus size={12} /> {t('routesAddService')}
                    </ToolBtn>
                    <ToolBtn
                      onClick={() => {
                        const next = addActionRoute(bridgeCfg)
                        patchBridgeActions(next.actions)
                      }}
                    >
                      <Plus size={12} /> {t('routesAddAction')}
                    </ToolBtn>
                  </div>

                  {bridgeCfg.routes.map((r) => (
                    <div key={r.id} className="border border-bus-border rounded-sm p-2 space-y-2">
                      <div className="flex justify-between items-center">
                        <span className="font-mono text-[10px] text-bus-cyan">{t('routesTopics')}</span>
                        <button
                          type="button"
                          className="text-bus-amber text-[10px]"
                          onClick={() =>
                            patchBridgeRoutes(bridgeCfg.routes.filter((x) => x.id !== r.id))
                          }
                        >
                          {t('flowRemove')}
                        </button>
                      </div>
                      <Field label={t('routesFieldRosTopic')}>
                        <input
                          className={inputCls}
                          value={r.ros_topic}
                          onChange={(e) =>
                            patchBridgeRoutes(
                              bridgeCfg.routes.map((x) =>
                                x.id === r.id ? { ...x, ros_topic: e.target.value } : x,
                              ),
                            )
                          }
                        />
                      </Field>
                      <Field label={t('routesFieldBusTopic')}>
                        <input
                          className={inputCls}
                          value={r.bus_topic}
                          onChange={(e) =>
                            patchBridgeRoutes(
                              bridgeCfg.routes.map((x) =>
                                x.id === r.id ? { ...x, bus_topic: e.target.value } : x,
                              ),
                            )
                          }
                        />
                      </Field>
                      <Field label={t('routesFieldType')}>
                        <select
                          className={inputCls}
                          value={r.type}
                          onChange={(e) =>
                            patchBridgeRoutes(
                              bridgeCfg.routes.map((x) =>
                                x.id === r.id ? { ...x, type: e.target.value } : x,
                              ),
                            )
                          }
                        >
                          {TOPIC_TYPES.map((ty) => (
                            <option key={ty} value={ty}>
                              {ty}
                            </option>
                          ))}
                        </select>
                      </Field>
                      <Field label={t('routesFieldDirection')}>
                        <select
                          className={inputCls}
                          value={r.direction}
                          onChange={(e) =>
                            patchBridgeRoutes(
                              bridgeCfg.routes.map((x) =>
                                x.id === r.id
                                  ? { ...x, direction: e.target.value as TopicRoute['direction'] }
                                  : x,
                              ),
                            )
                          }
                        >
                          {TOPIC_DIRECTIONS.map((d) => (
                            <option key={d} value={d}>
                              {dirLabel(d)}
                            </option>
                          ))}
                        </select>
                      </Field>
                    </div>
                  ))}

                  {bridgeCfg.services.map((s) => (
                    <div key={s.id} className="border border-bus-border rounded-sm p-2 space-y-2">
                      <div className="flex justify-between">
                        <span className="font-mono text-[10px] text-bus-cyan">{t('routesServices')}</span>
                        <button
                          type="button"
                          className="text-bus-amber text-[10px]"
                          onClick={() =>
                            patchBridgeServices(bridgeCfg.services.filter((x) => x.id !== s.id))
                          }
                        >
                          {t('flowRemove')}
                        </button>
                      </div>
                      <Field label={t('routesFieldRosService')}>
                        <input
                          className={inputCls}
                          value={s.ros_service}
                          onChange={(e) =>
                            patchBridgeServices(
                              bridgeCfg.services.map((x) =>
                                x.id === s.id ? { ...x, ros_service: e.target.value } : x,
                              ),
                            )
                          }
                        />
                      </Field>
                      <Field label={t('routesFieldBusService')}>
                        <input
                          className={inputCls}
                          value={s.bus_service}
                          onChange={(e) =>
                            patchBridgeServices(
                              bridgeCfg.services.map((x) =>
                                x.id === s.id ? { ...x, bus_service: e.target.value } : x,
                              ),
                            )
                          }
                        />
                      </Field>
                      <Field label={t('routesFieldType')}>
                        <select
                          className={inputCls}
                          value={s.type}
                          onChange={(e) =>
                            patchBridgeServices(
                              bridgeCfg.services.map((x) =>
                                x.id === s.id ? { ...x, type: e.target.value } : x,
                              ),
                            )
                          }
                        >
                          {SERVICE_TYPES.map((ty) => (
                            <option key={ty} value={ty}>
                              {ty}
                            </option>
                          ))}
                        </select>
                      </Field>
                      <Field label={t('routesFieldDirection')}>
                        <select
                          className={inputCls}
                          value={s.direction}
                          onChange={(e) =>
                            patchBridgeServices(
                              bridgeCfg.services.map((x) =>
                                x.id === s.id
                                  ? {
                                      ...x,
                                      direction: e.target.value as ServiceRoute['direction'],
                                    }
                                  : x,
                              ),
                            )
                          }
                        >
                          {SRV_ACT_DIRECTIONS.map((d) => (
                            <option key={d} value={d}>
                              {dirLabel(d)}
                            </option>
                          ))}
                        </select>
                      </Field>
                    </div>
                  ))}

                  {bridgeCfg.actions.map((a) => (
                    <div key={a.id} className="border border-bus-border rounded-sm p-2 space-y-2">
                      <div className="flex justify-between">
                        <span className="font-mono text-[10px] text-bus-cyan">{t('routesActions')}</span>
                        <button
                          type="button"
                          className="text-bus-amber text-[10px]"
                          onClick={() =>
                            patchBridgeActions(bridgeCfg.actions.filter((x) => x.id !== a.id))
                          }
                        >
                          {t('flowRemove')}
                        </button>
                      </div>
                      <Field label={t('routesFieldRosAction')}>
                        <input
                          className={inputCls}
                          value={a.ros_action}
                          onChange={(e) =>
                            patchBridgeActions(
                              bridgeCfg.actions.map((x) =>
                                x.id === a.id ? { ...x, ros_action: e.target.value } : x,
                              ),
                            )
                          }
                        />
                      </Field>
                      <Field label={t('routesFieldBusAction')}>
                        <input
                          className={inputCls}
                          value={a.bus_action}
                          onChange={(e) =>
                            patchBridgeActions(
                              bridgeCfg.actions.map((x) =>
                                x.id === a.id ? { ...x, bus_action: e.target.value } : x,
                              ),
                            )
                          }
                        />
                      </Field>
                      <Field label={t('routesFieldType')}>
                        <select
                          className={inputCls}
                          value={a.type}
                          onChange={(e) =>
                            patchBridgeActions(
                              bridgeCfg.actions.map((x) =>
                                x.id === a.id ? { ...x, type: e.target.value } : x,
                              ),
                            )
                          }
                        >
                          {ACTION_TYPES.map((ty) => (
                            <option key={ty} value={ty}>
                              {ty}
                            </option>
                          ))}
                        </select>
                      </Field>
                      <Field label={t('routesFieldDirection')}>
                        <select
                          className={inputCls}
                          value={a.direction}
                          onChange={(e) =>
                            patchBridgeActions(
                              bridgeCfg.actions.map((x) =>
                                x.id === a.id
                                  ? {
                                      ...x,
                                      direction: e.target.value as ActionRoute['direction'],
                                    }
                                  : x,
                              ),
                            )
                          }
                        >
                          {SRV_ACT_DIRECTIONS.map((d) => (
                            <option key={d} value={d}>
                              {dirLabel(d)}
                            </option>
                          ))}
                        </select>
                      </Field>
                    </div>
                  ))}

                  <ToolBtn
                    onClick={() => {
                      const y = toBridgeYaml(selectedNode, flow.robot_bus)
                      if (!y) return
                      const blob = new Blob([y], { type: 'text/yaml' })
                      const url = URL.createObjectURL(blob)
                      const a = document.createElement('a')
                      a.href = url
                      a.download = `${selectedNode.name}.yaml`
                      a.click()
                      URL.revokeObjectURL(url)
                      setStatus(t('flowBridgeExported'))
                    }}
                  >
                    <Download size={12} /> {t('flowExportBridge')}
                  </ToolBtn>
                </div>
              )}
            </aside>
          )}

          {(showYaml || showLaunch) && (
            <div className="border-t border-bus-border bg-bus-bg p-3 max-h-40 overflow-auto flex flex-col gap-2">
              {showLaunch && (
                <div>
                  <div className="flex items-center justify-between mb-1">
                    <span className="font-mono text-[10px] text-bus-muted uppercase">
                      {t('flowLaunch')}
                    </span>
                    <ToolBtn onClick={() => void onCopyLaunch()}>
                      <Copy size={12} /> {t('flowCopy')}
                    </ToolBtn>
                  </div>
                  <pre className="font-mono text-[11px] text-bus-muted whitespace-pre-wrap">
                    {launchPreview}
                  </pre>
                </div>
              )}
              {showYaml && (
                <pre className="font-mono text-[11px] text-bus-muted whitespace-pre-wrap">
                  {yamlPreview}
                </pre>
              )}
            </div>
          )}
        </div>
      </div>
    </section>
  )
}

function ToolBtn({
  children,
  onClick,
  className = '',
}: {
  children: ReactNode
  onClick: () => void
  className?: string
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`inline-flex items-center gap-1 font-mono text-[11px] px-2 py-1 border border-bus-border rounded-sm bg-bus-bg text-bus-text hover:border-bus-cyan hover:text-bus-cyan ${className}`}
    >
      {children}
    </button>
  )
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div>
      <label className="block font-mono text-[11px] text-bus-muted mb-1">{label}</label>
      {children}
    </div>
  )
}
