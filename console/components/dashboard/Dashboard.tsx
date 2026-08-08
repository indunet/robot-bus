'use client'

import { useState, useEffect, useRef } from 'react'
import {
  EMPTY_BROKER,
  mergeTopics,
  type BrokerInfo,
  type TopicInfo,
  type LogEntry,
  type ServiceInfo,
  type ActionInfo,
  type TopologyInfo,
} from '@/lib/mock-data'
import { startConsoleBus } from '@/lib/console-bus'
import StatusBar from './StatusBar'
import Sidebar, { type Tab } from './Sidebar'
import BrokerOverview from './BrokerOverview'
import OverviewStats from './OverviewStats'
import TopicTable from './TopicTable'
import ServiceActionTable from './ServiceActionTable'
import EventStream from './EventStream'
import TopologyView from './TopologyView'
import BotWindows from './BotWindows'
import ThroughputChart, {
  ServiceRateChart,
  ActionRateChart,
  generateRateHistory,
  type RatePoint,
} from './ThroughputChart'
import { formatHms } from '@/lib/utils'

function appendRatePoint(prev: RatePoint[], label: string, value: number): RatePoint[] {
  return [...prev.slice(1), { t: label, value }]
}

export default function Dashboard() {
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
  const [botOpen, setBotOpen] = useState(false)
  const seenLogIds = useRef(new Set<string>())

  useEffect(() => {
    return startConsoleBus({
      onStatus: (status) => {
        setBroker(status)
        const label = formatHms()
        setThroughput((prev) => appendRatePoint(prev, label, status.msgPerSec))
        setSvcRate((prev) => appendRatePoint(prev, label, status.svcCallsPerSec ?? 0))
        setActRate((prev) => appendRatePoint(prev, label, status.actRunsPerSec ?? 0))
      },
      onTopics: (topicList) => {
        setTopics((prev) => mergeTopics(prev, topicList))
      },
      onServices: setServices,
      onActions: setActions,
      onTopology: setTopology,
      onEvent: (entry) => {
        if (seenLogIds.current.has(entry.id)) return
        seenLogIds.current.add(entry.id)
        setLogs((prev) => [...prev.slice(-200), entry])
      },
      onOffline: () => {
        setBroker((prev) => ({ ...prev, status: 'OFFLINE' }))
      },
    })
  }, [])

  return (
    <div className="flex flex-col h-screen bg-bus-bg overflow-hidden">
      <StatusBar broker={broker} />

      <div className="flex flex-1 min-h-0">
        <Sidebar
          active={activeTab}
          onSelect={setActiveTab}
          onOpenBot={() => setBotOpen(true)}
        />

        <main className="flex-1 min-h-0 overflow-y-auto p-3 flex flex-col">
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
            />
          )}
          {activeTab === 'topology' && <TopologyView topology={topology} />}
          {activeTab === 'topics' && (
            <div className="flex flex-col gap-3 flex-1 min-h-0">
              <div className="h-[16rem] shrink-0">
                <ThroughputChart data={throughput} />
              </div>
              <div className="flex-1 min-h-[12rem]">
                <TopicTable topics={topics} />
              </div>
            </div>
          )}
          {activeTab === 'services' && (
            <div className="flex flex-col gap-3 flex-1 min-h-0">
              <div className="h-[16rem] shrink-0">
                <ServiceRateChart data={svcRate} />
              </div>
              <div className="flex-1 min-h-[12rem]">
                <ServiceActionTable services={services} actions={actions} mode="services" />
              </div>
            </div>
          )}
          {activeTab === 'actions' && (
            <div className="flex flex-col gap-3 flex-1 min-h-0">
              <div className="h-[16rem] shrink-0">
                <ActionRateChart data={actRate} />
              </div>
              <div className="flex-1 min-h-[12rem]">
                <ServiceActionTable services={services} actions={actions} mode="actions" />
              </div>
            </div>
          )}
          {activeTab === 'logs' && (
            <div className="flex-1 min-h-0">
              <EventStream logs={logs} />
            </div>
          )}
        </main>
      </div>
      <BotWindows open={botOpen} onClose={() => setBotOpen(false)} />
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
}: {
  broker: BrokerInfo
  topics: TopicInfo[]
  services: ServiceInfo[]
  actions: ActionInfo[]
  logs: LogEntry[]
  throughput: RatePoint[]
  svcRate: RatePoint[]
  actRate: RatePoint[]
}) {
  const listMax = '14rem'

  return (
    <div className="flex flex-col gap-3 min-h-full flex-1">
      <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-3 items-stretch shrink-0">
        <BrokerOverview broker={broker} />
        <OverviewStats
          broker={broker}
          topics={topics}
          services={services}
          actions={actions}
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-3 items-stretch shrink-0" style={{ minHeight: '16rem' }}>
        <TopicTable topics={topics} maxBodyHeight={listMax} />
        <ThroughputChart data={throughput} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-3 items-stretch shrink-0" style={{ minHeight: '16rem' }}>
        <ServiceActionTable
          services={services}
          actions={actions}
          mode="services"
          maxBodyHeight={listMax}
        />
        <ServiceRateChart data={svcRate} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-3 items-stretch shrink-0" style={{ minHeight: '16rem' }}>
        <ServiceActionTable
          services={services}
          actions={actions}
          mode="actions"
          maxBodyHeight={listMax}
        />
        <ActionRateChart data={actRate} />
      </div>

      <div className="flex-1 min-h-[14rem] flex flex-col">
        <EventStream logs={logs} />
      </div>
    </div>
  )
}
