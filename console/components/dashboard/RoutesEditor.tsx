'use client'

import { memo, useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import {
  ReactFlow,
  Background,
  Controls,
  Handle,
  MarkerType,
  Position,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
  type NodeProps,
  type EdgeMouseHandler,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { GitBranch, Download, Upload, Plus, Trash2, Copy, Settings2, X } from 'lucide-react'
import { PanelHeader } from './BrokerOverview'
import { useI18n } from '@/lib/i18n'
import {
  ACTION_TYPES,
  CAMERA_H264_EXAMPLE,
  SERVICE_TYPES,
  SRV_ACT_DIRECTIONS,
  TOPIC_DIRECTIONS,
  TOPIC_TYPES,
  addActionRoute,
  addServiceRoute,
  addTopicRoute,
  emptyConfig,
  exportBridgeYaml,
  parseBridgeYaml,
  validateConfig,
  type ActionRoute,
  type BridgeConfig,
  type ServiceRoute,
  type TopicRoute,
} from '@/lib/bridge-yaml'

type Selected =
  | { kind: 'route'; id: string }
  | { kind: 'service'; id: string }
  | { kind: 'action'; id: string }
  | { kind: 'broker' }
  | null

type PortKind = 'topic' | 'service' | 'action'

type PortRow = {
  id: string
  kind: PortKind
  name: string
  typeName: string
  direction: string
  selected: boolean
}

type SystemNodeData = {
  title: string
  subtitle: string
  accent: 'ros' | 'bus'
  side: 'ros' | 'bus'
  ports: PortRow[]
  onSelectPort: (kind: PortKind, id: string) => void
}

function shortName(name: string): string {
  if (name.length <= 26) return name
  const parts = name.split('/').filter(Boolean)
  if (parts.length === 0) return name.slice(0, 24) + '…'
  return `…/${parts[parts.length - 1]}`
}

function dirLabel(direction: string): string {
  if (direction === 'ros_to_bus') return 'ROS → Bus'
  if (direction === 'bus_to_ros') return 'Bus → ROS'
  if (direction === 'both') return 'ROS ↔ Bus'
  return direction
}

function dirColor(direction: string): string {
  if (direction === 'ros_to_bus') return '#a78bfa'
  if (direction === 'bus_to_ros') return '#22c55e'
  return '#00d4ff'
}

function buildPorts(cfg: BridgeConfig, selected: Selected, side: 'ros' | 'bus'): PortRow[] {
  const ports: PortRow[] = []
  for (const r of cfg.routes) {
    ports.push({
      id: r.id,
      kind: 'topic',
      name: side === 'ros' ? r.ros_topic : r.bus_topic,
      typeName: r.type,
      direction: r.direction,
      selected: selected?.kind === 'route' && selected.id === r.id,
    })
  }
  for (const s of cfg.services) {
    ports.push({
      id: s.id,
      kind: 'service',
      name: side === 'ros' ? s.ros_service : s.bus_service,
      typeName: s.type,
      direction: s.direction,
      selected: selected?.kind === 'service' && selected.id === s.id,
    })
  }
  for (const a of cfg.actions) {
    ports.push({
      id: a.id,
      kind: 'action',
      name: side === 'ros' ? a.ros_action : a.bus_action,
      typeName: a.type,
      direction: a.direction,
      selected: selected?.kind === 'action' && selected.id === a.id,
    })
  }
  return ports
}

function handleId(side: 'ros' | 'bus', kind: PortKind, id: string): string {
  return `${side}:${kind}:${id}`
}

function edgeId(kind: PortKind, id: string): string {
  return `edge:${kind}:${id}`
}

function parseEdgeId(id: string): Selected {
  const m = /^edge:(route|service|action):(.+)$/.exec(id)
  if (!m) return null
  if (m[1] === 'route') return { kind: 'route', id: m[2] }
  if (m[1] === 'service') return { kind: 'service', id: m[2] }
  return { kind: 'action', id: m[2] }
}

const SystemNode = memo(function SystemNode({ data }: NodeProps) {
  const d = data as SystemNodeData
  const isRos = d.side === 'ros'
  const border = isRos ? 'border-purple-400/70' : 'border-bus-green/70'
  const header = isRos ? 'bg-purple-500/10 text-purple-200' : 'bg-bus-green/10 text-emerald-200'
  const handleColor = isRos ? '!bg-purple-400' : '!bg-bus-green'

  return (
    <div className={`min-w-[240px] max-w-[280px] rounded-sm border ${border} bg-[#1a2228] shadow-md font-mono`}>
      <div className={`px-3 py-2 border-b border-bus-border ${header}`}>
        <div className="text-[13px] font-semibold tracking-wide">{d.title}</div>
        <div className="text-[10px] opacity-70 mt-0.5">{d.subtitle}</div>
      </div>

      {d.ports.length === 0 ? (
        <div className="px-3 py-4 text-[11px] text-bus-muted">No routes yet — add one above</div>
      ) : (
        <div className="py-1">
          {d.ports.map((p) => (
            <button
              key={`${p.kind}-${p.id}`}
              type="button"
              onClick={(e) => {
                e.stopPropagation()
                d.onSelectPort(p.kind, p.id)
              }}
              className={`relative w-full text-left min-h-[40px] px-3 py-1.5 transition-colors ${
                p.selected ? 'bg-bus-cyan/15' : 'hover:bg-[#1f2428]'
              }`}
            >
              {isRos ? (
                <Handle
                  type="source"
                  position={Position.Right}
                  id={handleId('ros', p.kind, p.id)}
                  className={`!w-2.5 !h-2.5 ${handleColor} !border !border-[#0e1012] !-right-[5px]`}
                />
              ) : (
                <Handle
                  type="target"
                  position={Position.Left}
                  id={handleId('bus', p.kind, p.id)}
                  className={`!w-2.5 !h-2.5 ${handleColor} !border !border-[#0e1012] !-left-[5px]`}
                />
              )}
              <div className="flex items-center gap-2">
                <span
                  className={`shrink-0 text-[9px] uppercase tracking-wider px-1 py-0.5 rounded-sm border ${
                    p.kind === 'topic'
                      ? 'border-bus-cyan/40 text-bus-cyan'
                      : p.kind === 'service'
                        ? 'border-sky-400/40 text-sky-300'
                        : 'border-pink-400/40 text-pink-300'
                  }`}
                >
                  {p.kind === 'topic' ? 'topic' : p.kind === 'service' ? 'svc' : 'act'}
                </span>
                <div className="min-w-0 flex-1">
                  <div className={`text-[12px] truncate ${p.selected ? 'text-bus-cyan' : 'text-bus-text'}`} title={p.name}>
                    {shortName(p.name || '(empty)')}
                  </div>
                  <div className="text-[9px] text-bus-muted truncate" title={p.typeName}>
                    {dirLabel(p.direction)} · {p.typeName.split('/').pop()}
                  </div>
                </div>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  )
})

const nodeTypes = { system: SystemNode }

const inputCls =
  'w-full bg-bus-bg border border-bus-border rounded-sm px-2 py-1.5 font-mono text-xs text-bus-text focus:outline-none focus:border-bus-cyan'

export default function RoutesEditor() {
  const { t } = useI18n()
  const fileRef = useRef<HTMLInputElement>(null)
  const [cfg, setCfg] = useState<BridgeConfig>(() => parseBridgeYaml(CAMERA_H264_EXAMPLE).config)
  const [selected, setSelected] = useState<Selected>(null)
  const [pasteOpen, setPasteOpen] = useState(false)
  const [pasteText, setPasteText] = useState('')
  const [status, setStatus] = useState<string | null>(null)
  const [showYaml, setShowYaml] = useState(false)

  const errors = useMemo(() => validateConfig(cfg), [cfg])
  const yamlPreview = useMemo(() => exportBridgeYaml(cfg), [cfg])

  const selectPort = useCallback((kind: PortKind, id: string) => {
    if (kind === 'topic') setSelected({ kind: 'route', id })
    else if (kind === 'service') setSelected({ kind: 'service', id })
    else setSelected({ kind: 'action', id })
  }, [])

  const graphSignature = useMemo(
    () =>
      JSON.stringify({
        routes: cfg.routes,
        services: cfg.services,
        actions: cfg.actions,
        selected,
        transport: cfg.robot_bus.transport,
        host: cfg.robot_bus.host,
      }),
    [cfg, selected],
  )

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([])
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([])

  useEffect(() => {
    setNodes((prev) => {
      const prevPos = new Map(prev.map((n) => [n.id, n.position]))
      const rosPorts = buildPorts(cfg, selected, 'ros')
      const busPorts = buildPorts(cfg, selected, 'bus')
      return [
        {
          id: 'ros',
          type: 'system',
          position: prevPos.get('ros') ?? { x: 60, y: 40 },
          data: {
            title: 'ROS 2',
            subtitle: t('routesRosSide'),
            accent: 'ros',
            side: 'ros',
            ports: rosPorts,
            onSelectPort: selectPort,
          } satisfies SystemNodeData,
        },
        {
          id: 'bus',
          type: 'system',
          position: prevPos.get('bus') ?? { x: 520, y: 40 },
          data: {
            title: 'robot-bus',
            subtitle: `${cfg.robot_bus.transport}${cfg.robot_bus.host ? ` · ${cfg.robot_bus.host}` : ''}`,
            accent: 'bus',
            side: 'bus',
            ports: busPorts,
            onSelectPort: selectPort,
          } satisfies SystemNodeData,
        },
      ]
    })

    const nextEdges: Edge[] = []
    for (const r of cfg.routes) {
      const color = dirColor(r.direction)
      const selectedEdge = selected?.kind === 'route' && selected.id === r.id
      nextEdges.push({
        id: edgeId('topic', r.id),
        source: 'ros',
        target: 'bus',
        sourceHandle: handleId('ros', 'topic', r.id),
        targetHandle: handleId('bus', 'topic', r.id),
        label: dirLabel(r.direction),
        animated: selectedEdge || r.direction === 'both',
        style: {
          stroke: color,
          strokeWidth: selectedEdge ? 2.5 : 1.5,
        },
        labelStyle: { fill: '#9ca3af', fontSize: 10, fontFamily: 'ui-monospace, monospace' },
        labelBgStyle: { fill: '#0e1012', fillOpacity: 0.9 },
        markerEnd: { type: MarkerType.ArrowClosed, color, width: 16, height: 16 },
        markerStart:
          r.direction === 'both'
            ? { type: MarkerType.ArrowClosed, color, width: 16, height: 16 }
            : undefined,
      })
    }
    for (const s of cfg.services) {
      const color = '#38bdf8'
      const selectedEdge = selected?.kind === 'service' && selected.id === s.id
      nextEdges.push({
        id: edgeId('service', s.id),
        source: 'ros',
        target: 'bus',
        sourceHandle: handleId('ros', 'service', s.id),
        targetHandle: handleId('bus', 'service', s.id),
        label: `svc · ${dirLabel(s.direction)}`,
        style: { stroke: color, strokeWidth: selectedEdge ? 2.5 : 1.5, strokeDasharray: '5 3' },
        labelStyle: { fill: '#9ca3af', fontSize: 10, fontFamily: 'ui-monospace, monospace' },
        labelBgStyle: { fill: '#0e1012', fillOpacity: 0.9 },
        markerEnd: { type: MarkerType.ArrowClosed, color },
      })
    }
    for (const a of cfg.actions) {
      const color = '#f472b6'
      const selectedEdge = selected?.kind === 'action' && selected.id === a.id
      nextEdges.push({
        id: edgeId('action', a.id),
        source: 'ros',
        target: 'bus',
        sourceHandle: handleId('ros', 'action', a.id),
        targetHandle: handleId('bus', 'action', a.id),
        label: `act · ${dirLabel(a.direction)}`,
        style: { stroke: color, strokeWidth: selectedEdge ? 2.5 : 1.5, strokeDasharray: '5 3' },
        labelStyle: { fill: '#9ca3af', fontSize: 10, fontFamily: 'ui-monospace, monospace' },
        labelBgStyle: { fill: '#0e1012', fillOpacity: 0.9 },
        markerEnd: { type: MarkerType.ArrowClosed, color },
      })
    }
    setEdges(nextEdges)
  }, [graphSignature, cfg, selected, selectPort, setNodes, setEdges, t])

  const selectedRoute =
    selected?.kind === 'route' ? cfg.routes.find((r) => r.id === selected.id) : undefined
  const selectedService =
    selected?.kind === 'service' ? cfg.services.find((s) => s.id === selected.id) : undefined
  const selectedAction =
    selected?.kind === 'action' ? cfg.actions.find((a) => a.id === selected.id) : undefined

  const onImportFile = useCallback(
    (file: File) => {
      const reader = new FileReader()
      reader.onload = () => {
        const { config, errors: errs } = parseBridgeYaml(String(reader.result ?? ''))
        setCfg(config)
        setSelected(null)
        setStatus(errs.length ? errs[0] : t('routesImported'))
      }
      reader.readAsText(file)
    },
    [t],
  )

  const onExport = useCallback(() => {
    const blob = new Blob([exportBridgeYaml(cfg)], { type: 'text/yaml;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'ros2_bridge.yaml'
    a.click()
    URL.revokeObjectURL(url)
    setStatus(t('routesExported'))
  }, [cfg, t])

  const onCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(exportBridgeYaml(cfg))
      setStatus(t('routesCopied'))
    } catch {
      setStatus(t('routesCopyFailed'))
    }
  }, [cfg, t])

  const updateRoute = (id: string, patch: Partial<TopicRoute>) => {
    setCfg((prev) => ({
      ...prev,
      routes: prev.routes.map((r) => (r.id === id ? { ...r, ...patch } : r)),
    }))
  }
  const updateService = (id: string, patch: Partial<ServiceRoute>) => {
    setCfg((prev) => ({
      ...prev,
      services: prev.services.map((s) => (s.id === id ? { ...s, ...patch } : s)),
    }))
  }
  const updateAction = (id: string, patch: Partial<ActionRoute>) => {
    setCfg((prev) => ({
      ...prev,
      actions: prev.actions.map((a) => (a.id === id ? { ...a, ...patch } : a)),
    }))
  }

  const removeSelected = () => {
    if (!selected || selected.kind === 'broker') return
    setCfg((prev) => {
      if (selected.kind === 'route') {
        return { ...prev, routes: prev.routes.filter((r) => r.id !== selected.id) }
      }
      if (selected.kind === 'service') {
        return { ...prev, services: prev.services.filter((s) => s.id !== selected.id) }
      }
      return { ...prev, actions: prev.actions.filter((a) => a.id !== selected.id) }
    })
    setSelected(null)
  }

  const addAndSelectTopic = () => {
    setCfg((c) => {
      const next = addTopicRoute(c)
      const last = next.routes[next.routes.length - 1]
      setSelected({ kind: 'route', id: last.id })
      return next
    })
  }
  const addAndSelectService = () => {
    setCfg((c) => {
      const next = addServiceRoute(c)
      const last = next.services[next.services.length - 1]
      setSelected({ kind: 'service', id: last.id })
      return next
    })
  }
  const addAndSelectAction = () => {
    setCfg((c) => {
      const next = addActionRoute(c)
      const last = next.actions[next.actions.length - 1]
      setSelected({ kind: 'action', id: last.id })
      return next
    })
  }

  const onEdgeClick: EdgeMouseHandler = useCallback((_, edge) => {
    setSelected(parseEdgeId(edge.id))
  }, [])

  const inspectorOpen = selected !== null

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm flex flex-col h-full min-h-0">
      <PanelHeader
        icon={<GitBranch size={14} />}
        title={t('routesTitle')}
        sub={t('routesSub', {
          r: cfg.routes.length,
          s: cfg.services.length,
          a: cfg.actions.length,
        })}
      />
      <p className="px-3 pb-2 font-mono text-[11px] text-bus-muted">{t('routesHint')}</p>

      <div className="px-3 pb-2 flex flex-wrap gap-2 items-center">
        <ToolBtn onClick={() => fileRef.current?.click()}>
          <Upload size={12} /> {t('routesImport')}
        </ToolBtn>
        <input
          ref={fileRef}
          type="file"
          accept=".yaml,.yml,text/yaml"
          className="hidden"
          onChange={(e) => {
            const f = e.target.files?.[0]
            if (f) onImportFile(f)
            e.target.value = ''
          }}
        />
        <ToolBtn onClick={() => setPasteOpen((v) => !v)}>{t('routesPaste')}</ToolBtn>
        <ToolBtn onClick={onExport}>
          <Download size={12} /> {t('routesExport')}
        </ToolBtn>
        <ToolBtn onClick={onCopy}>
          <Copy size={12} /> {t('routesCopy')}
        </ToolBtn>
        <ToolBtn onClick={() => setShowYaml((v) => !v)}>{t('routesToggleYaml')}</ToolBtn>
        <ToolBtn
          onClick={() => {
            setCfg(parseBridgeYaml(CAMERA_H264_EXAMPLE).config)
            setSelected(null)
            setStatus(t('routesExampleLoaded'))
          }}
        >
          {t('routesExample')}
        </ToolBtn>
        <span className="mx-1 w-px h-4 bg-bus-border" />
        <ToolBtn onClick={addAndSelectTopic}>
          <Plus size={12} /> {t('routesAddTopic')}
        </ToolBtn>
        <ToolBtn onClick={addAndSelectService}>
          <Plus size={12} /> {t('routesAddService')}
        </ToolBtn>
        <ToolBtn onClick={addAndSelectAction}>
          <Plus size={12} /> {t('routesAddAction')}
        </ToolBtn>
        <ToolBtn onClick={() => setSelected({ kind: 'broker' })}>
          <Settings2 size={12} /> {t('routesBroker')}
        </ToolBtn>
        {selected && selected.kind !== 'broker' && (
          <ToolBtn className="text-bus-amber" onClick={removeSelected}>
            <Trash2 size={12} /> {t('routesRemove')}
          </ToolBtn>
        )}
      </div>

      {pasteOpen && (
        <div className="px-3 pb-2 flex flex-col gap-2">
          <textarea
            className={`${inputCls} h-28`}
            value={pasteText}
            onChange={(e) => setPasteText(e.target.value)}
            placeholder="paste Ros2Bridge YAML…"
          />
          <ToolBtn
            onClick={() => {
              const { config, errors: errs } = parseBridgeYaml(pasteText)
              setCfg(config)
              setSelected(null)
              setPasteOpen(false)
              setStatus(errs.length ? errs[0] : t('routesImported'))
            }}
          >
            {t('routesApplyPaste')}
          </ToolBtn>
        </div>
      )}

      {(status || errors.length > 0) && (
        <div className="px-3 pb-2 font-mono text-[11px]">
          {errors.length > 0 ? (
            <span className="text-bus-amber">{errors.join(' · ')}</span>
          ) : (
            <span className="text-bus-cyan">{status}</span>
          )}
        </div>
      )}

      <div className="relative flex-1 min-h-0 border-t border-bus-border flex flex-col">
        <div className="flex-1 min-h-0" style={{ height: showYaml ? 'calc(100vh - 420px)' : 'calc(100vh - 260px)' }}>
          <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={nodeTypes}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onEdgeClick={onEdgeClick}
            onPaneClick={() => setSelected(null)}
            fitView
            proOptions={{ hideAttribution: true }}
            nodesConnectable={false}
            nodesDraggable
            elementsSelectable
          >
            <Background color="#2a2f35" gap={20} />
            <Controls />
          </ReactFlow>
        </div>

        {inspectorOpen && (
          <aside className="absolute top-2 right-2 bottom-2 w-80 max-w-[calc(100%-1rem)] border border-bus-border bg-[#12161a]/95 backdrop-blur-sm rounded-sm shadow-lg overflow-y-auto p-3 z-10">
            <div className="flex items-center justify-between mb-3">
              <div className="font-mono text-xs text-bus-muted uppercase tracking-wider">
                {selected?.kind === 'broker'
                  ? t('routesBroker')
                  : selected?.kind === 'route'
                    ? t('routesEditTopic')
                    : selected?.kind === 'service'
                      ? t('routesEditService')
                      : t('routesEditAction')}
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
                <FriendlyField label={t('routesFieldTransport')}>
                  <select
                    className={inputCls}
                    value={cfg.robot_bus.transport}
                    onChange={(e) =>
                      setCfg((c) => ({
                        ...c,
                        robot_bus: { ...c.robot_bus, transport: e.target.value },
                      }))
                    }
                  >
                    <option value="tcp">tcp</option>
                    <option value="ipc">ipc</option>
                    <option value="discover">discover</option>
                  </select>
                </FriendlyField>
                <FriendlyField label={t('routesFieldHost')}>
                  <input
                    className={inputCls}
                    value={cfg.robot_bus.host ?? ''}
                    onChange={(e) =>
                      setCfg((c) => ({
                        ...c,
                        robot_bus: { ...c.robot_bus, host: e.target.value },
                      }))
                    }
                  />
                </FriendlyField>
              </div>
            )}

            {selectedRoute && (
              <div className="space-y-3">
                <FriendlyField label={t('routesFieldRosTopic')}>
                  <input
                    className={inputCls}
                    value={selectedRoute.ros_topic}
                    onChange={(e) => updateRoute(selectedRoute.id, { ros_topic: e.target.value })}
                  />
                </FriendlyField>
                <FriendlyField label={t('routesFieldBusTopic')}>
                  <input
                    className={inputCls}
                    value={selectedRoute.bus_topic}
                    onChange={(e) => updateRoute(selectedRoute.id, { bus_topic: e.target.value })}
                  />
                </FriendlyField>
                <FriendlyField label={t('routesFieldType')}>
                  <select
                    className={inputCls}
                    value={selectedRoute.type}
                    onChange={(e) => updateRoute(selectedRoute.id, { type: e.target.value })}
                  >
                    {TOPIC_TYPES.map((ty) => (
                      <option key={ty} value={ty}>
                        {ty}
                      </option>
                    ))}
                  </select>
                </FriendlyField>
                <FriendlyField label={t('routesFieldDirection')}>
                  <div className="grid grid-cols-1 gap-1.5">
                    {TOPIC_DIRECTIONS.map((d) => (
                      <button
                        key={d}
                        type="button"
                        onClick={() => updateRoute(selectedRoute.id, { direction: d })}
                        className={`text-left font-mono text-[11px] px-2 py-1.5 rounded-sm border ${
                          selectedRoute.direction === d
                            ? 'border-bus-cyan bg-bus-cyan/15 text-bus-cyan'
                            : 'border-bus-border text-bus-text hover:border-bus-cyan/50'
                        }`}
                      >
                        {dirLabel(d)}
                      </button>
                    ))}
                  </div>
                </FriendlyField>
              </div>
            )}

            {selectedService && (
              <div className="space-y-3">
                <FriendlyField label={t('routesFieldRosService')}>
                  <input
                    className={inputCls}
                    value={selectedService.ros_service}
                    onChange={(e) => updateService(selectedService.id, { ros_service: e.target.value })}
                  />
                </FriendlyField>
                <FriendlyField label={t('routesFieldBusService')}>
                  <input
                    className={inputCls}
                    value={selectedService.bus_service}
                    onChange={(e) => updateService(selectedService.id, { bus_service: e.target.value })}
                  />
                </FriendlyField>
                <FriendlyField label={t('routesFieldType')}>
                  <select
                    className={inputCls}
                    value={selectedService.type}
                    onChange={(e) => updateService(selectedService.id, { type: e.target.value })}
                  >
                    {SERVICE_TYPES.map((ty) => (
                      <option key={ty} value={ty}>
                        {ty}
                      </option>
                    ))}
                  </select>
                </FriendlyField>
                <FriendlyField label={t('routesFieldDirection')}>
                  <div className="grid grid-cols-1 gap-1.5">
                    {SRV_ACT_DIRECTIONS.map((d) => (
                      <button
                        key={d}
                        type="button"
                        onClick={() => updateService(selectedService.id, { direction: d })}
                        className={`text-left font-mono text-[11px] px-2 py-1.5 rounded-sm border ${
                          selectedService.direction === d
                            ? 'border-bus-cyan bg-bus-cyan/15 text-bus-cyan'
                            : 'border-bus-border text-bus-text hover:border-bus-cyan/50'
                        }`}
                      >
                        {dirLabel(d)}
                      </button>
                    ))}
                  </div>
                </FriendlyField>
              </div>
            )}

            {selectedAction && (
              <div className="space-y-3">
                <FriendlyField label={t('routesFieldRosAction')}>
                  <input
                    className={inputCls}
                    value={selectedAction.ros_action}
                    onChange={(e) => updateAction(selectedAction.id, { ros_action: e.target.value })}
                  />
                </FriendlyField>
                <FriendlyField label={t('routesFieldBusAction')}>
                  <input
                    className={inputCls}
                    value={selectedAction.bus_action}
                    onChange={(e) => updateAction(selectedAction.id, { bus_action: e.target.value })}
                  />
                </FriendlyField>
                <FriendlyField label={t('routesFieldType')}>
                  <select
                    className={inputCls}
                    value={selectedAction.type}
                    onChange={(e) => updateAction(selectedAction.id, { type: e.target.value })}
                  >
                    {ACTION_TYPES.map((ty) => (
                      <option key={ty} value={ty}>
                        {ty}
                      </option>
                    ))}
                  </select>
                </FriendlyField>
                <FriendlyField label={t('routesFieldDirection')}>
                  <div className="grid grid-cols-1 gap-1.5">
                    {SRV_ACT_DIRECTIONS.map((d) => (
                      <button
                        key={d}
                        type="button"
                        onClick={() => updateAction(selectedAction.id, { direction: d })}
                        className={`text-left font-mono text-[11px] px-2 py-1.5 rounded-sm border ${
                          selectedAction.direction === d
                            ? 'border-bus-cyan bg-bus-cyan/15 text-bus-cyan'
                            : 'border-bus-border text-bus-text hover:border-bus-cyan/50'
                        }`}
                      >
                        {dirLabel(d)}
                      </button>
                    ))}
                  </div>
                </FriendlyField>
              </div>
            )}
          </aside>
        )}

        {showYaml && (
          <div className="border-t border-bus-border bg-bus-bg p-3 max-h-40 overflow-auto">
            <pre className="font-mono text-[11px] text-bus-muted whitespace-pre-wrap">{yamlPreview}</pre>
          </div>
        )}
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

function FriendlyField({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div>
      <label className="block font-mono text-[11px] text-bus-muted mb-1">{label}</label>
      {children}
    </div>
  )
}
