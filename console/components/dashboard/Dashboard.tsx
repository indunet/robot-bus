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
  const [botWindows, setBotWindows] = useState({ sim: false, teleop: false })
  const seenLogIds = useRef(new Set<string>())
  const dateLocaleRef = useRef(dateLocale)
  dateLocaleRef.current = dateLocale

  useEffect(() => {
    return startConsoleBus({
      onStatus: (status) => {
        setBroker(status)
        const now = new Date()
        const label = now.toLocaleTimeString(dateLocaleRef.current, {
          minute: '2-digit',
          second: '2-digit',
          hour12: false,
        })
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
          onOpenBot={() => setBotWindows({ sim: true, teleop: true })}
        />

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
            />
          )}
          {activeTab === 'topology' && <TopologyView topology={topology} />}
          {activeTab === 'topics' && <TopicTable topics={topics} />}
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
      <BotWindows
        simOpen={botWindows.sim}
        teleopOpen={botWindows.teleop}
        onCloseSim={() => setBotWindows((current) => ({ ...current, sim: false }))}
        onCloseTeleop={() => setBotWindows((current) => ({ ...current, teleop: false }))}
      />
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
        <TopicTable topics={topics} maxBodyHeight={listMax} />
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
