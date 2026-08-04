'use client'

import { useState, useEffect, useRef, useCallback } from 'react'
import {
  EMPTY_BROKER,
  fetchStatus,
  fetchTopics,
  fetchServices,
  fetchActions,
  fetchTopology,
  mergeTopics,
  type BrokerInfo,
  type TopicInfo,
  type LogEntry,
  type ServiceInfo,
  type ActionInfo,
  type TopologyInfo,
} from '@/lib/mock-data'
import StatusBar from './StatusBar'
import Sidebar, { type Tab } from './Sidebar'
import BrokerOverview from './BrokerOverview'
import OverviewStats from './OverviewStats'
import TopicTable from './TopicTable'
import ServiceActionTable from './ServiceActionTable'
import EventStream from './EventStream'
import TopologyView from './TopologyView'
import FlowEditor from './FlowEditor'
import VisualizeWorkspace from './VisualizeWorkspace'
import LiveView from './LiveView'
import ThroughputChart, {
  ServiceRateChart,
  ActionRateChart,
  generateRateHistory,
  type RatePoint,
} from './ThroughputChart'
import { useI18n } from '@/lib/i18n'

function appendRatePoint(prev: RatePoint[], label: string, value: number): RatePoint[] {
  return [...prev.slice(1), { t: label, value }]
}

export default function Dashboard() {
  const { dateLocale } = useI18n()
  const [broker, setBroker] = useState<BrokerInfo>(EMPTY_BROKER)
  const [topics, setTopics] = useState<TopicInfo[]>([])
  const [services, setServices] = useState<ServiceInfo[]>([])
  const [actions, setActions] = useState<ActionInfo[]>([])
  const [topology, setTopology] = useState<TopologyInfo>({ nodes: [], edges: [] })
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [throughput, setThroughput] = useState(() => generateRateHistory(0))
  const [svcRate, setSvcRate] = useState(() => generateRateHistory(0))
  const [actRate, setActRate] = useState(() => generateRateHistory(0))
  const [activeTab, setActiveTab] = useState<Tab>('overview')
  const [vizTopic, setVizTopic] = useState<string | null>(null)
  const seenLogIds = useRef(new Set<string>())

  const openVisualize = useCallback((topic: string) => {
    setVizTopic(topic)
    setActiveTab('visualize')
  }, [])

  const poll = useCallback(async () => {
    try {
      const [status, topicList, serviceList, actionList, topo] = await Promise.all([
        fetchStatus(),
        fetchTopics(),
        fetchServices(),
        fetchActions(),
        fetchTopology(),
      ])
      setBroker(status)
      setTopics((prev) => mergeTopics(prev, topicList))
      setServices(serviceList)
      setActions(actionList)
      setTopology(topo)
      const now = new Date()
      const label = now.toLocaleTimeString(dateLocale, {
        minute: '2-digit',
        second: '2-digit',
        hour12: false,
      })
      setThroughput((prev) => appendRatePoint(prev, label, status.msgPerSec))
      setSvcRate((prev) => appendRatePoint(prev, label, status.svcCallsPerSec ?? 0))
      setActRate((prev) => appendRatePoint(prev, label, status.actRunsPerSec ?? 0))
    } catch {
      setBroker((prev) => ({ ...prev, status: 'OFFLINE' }))
    }
  }, [dateLocale])

  useEffect(() => {
    void poll()
    const id = setInterval(() => void poll(), 1000)
    return () => clearInterval(id)
  }, [poll])

  useEffect(() => {
    const es = new EventSource('/api/v1/events')
    es.addEventListener('log', (ev) => {
      try {
        const entry = JSON.parse((ev as MessageEvent).data) as LogEntry
        if (seenLogIds.current.has(entry.id)) return
        seenLogIds.current.add(entry.id)
        setLogs((prev) => [...prev.slice(-200), entry])
      } catch {
        /* ignore malformed */
      }
    })
    es.onerror = () => {
      // Browser will retry; mark degraded if status polls also fail.
    }
    return () => es.close()
  }, [])

  return (
    <div className="flex flex-col h-screen bg-bus-bg overflow-hidden">
      <StatusBar broker={broker} />

      <div className="flex flex-1 min-h-0">
        <Sidebar active={activeTab} onSelect={setActiveTab} />

        <main className="flex-1 overflow-y-auto p-3">
          {activeTab === 'overview' && (
            <OverviewLayout
              broker={broker}
              topics={topics}
              services={services}
              actions={actions}
              logs={logs}
              throughput={throughput}
              svcRate={svcRate}
              actRate={actRate}
              onVisualize={openVisualize}
            />
          )}
          {activeTab === 'topology' && <TopologyView topology={topology} />}
          {activeTab === 'flow' && <FlowEditor topology={topology} />}
          {activeTab === 'visualize' && (
            <VisualizeWorkspace
              topics={topics}
              grpcAddr={broker.grpcAddr}
              initialTopic={vizTopic}
              onConsumedInitial={() => setVizTopic(null)}
            />
          )}
          {activeTab === 'live' && <LiveView />}
          {activeTab === 'topics' && <TopicTable topics={topics} onVisualize={openVisualize} />}
          {activeTab === 'services' && (
            <ServiceActionTable services={services} actions={actions} mode="services" />
          )}
          {activeTab === 'actions' && (
            <ServiceActionTable services={services} actions={actions} mode="actions" />
          )}
          {activeTab === 'logs' && (
            <div style={{ height: 'calc(100vh - 88px)' }}>
              <EventStream logs={logs} />
            </div>
          )}
        </main>
      </div>
    </div>
  )
}

function OverviewLayout({
  broker,
  topics,
  services,
  actions,
  logs,
  throughput,
  svcRate,
  actRate,
  onVisualize,
}: {
  broker: BrokerInfo
  topics: TopicInfo[]
  services: ServiceInfo[]
  actions: ActionInfo[]
  logs: LogEntry[]
  throughput: RatePoint[]
  svcRate: RatePoint[]
  actRate: RatePoint[]
  onVisualize?: (topic: string) => void
}) {
  const listMax = '14rem'

  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-3 items-stretch">
        <BrokerOverview broker={broker} />
        <OverviewStats
          broker={broker}
          topics={topics}
          services={services}
          actions={actions}
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-3 items-stretch" style={{ minHeight: '16rem' }}>
        <TopicTable topics={topics} maxBodyHeight={listMax} onVisualize={onVisualize} />
        <ThroughputChart data={throughput} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-3 items-stretch" style={{ minHeight: '16rem' }}>
        <ServiceActionTable
          services={services}
          actions={actions}
          mode="services"
          maxBodyHeight={listMax}
        />
        <ServiceRateChart data={svcRate} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-3 items-stretch" style={{ minHeight: '16rem' }}>
        <ServiceActionTable
          services={services}
          actions={actions}
          mode="actions"
          maxBodyHeight={listMax}
        />
        <ActionRateChart data={actRate} />
      </div>

      <div style={{ height: '260px' }}>
        <EventStream logs={logs} />
      </div>
    </div>
  )
}
