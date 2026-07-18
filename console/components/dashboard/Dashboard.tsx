'use client'

import { useState, useEffect, useRef, useCallback } from 'react'
import {
  EMPTY_BROKER,
  fetchStatus,
  fetchTopics,
  mergeTopics,
  type BrokerInfo,
  type TopicInfo,
  type LogEntry,
  type ServiceInfo,
  type ActionInfo,
} from '@/lib/mock-data'
import StatusBar from './StatusBar'
import Sidebar from './Sidebar'
import MetricCards from './MetricCards'
import BrokerOverview from './BrokerOverview'
import TopicTable from './TopicTable'
import ServiceActionTable from './ServiceActionTable'
import EventStream from './EventStream'
import ThroughputChart, { generateThroughputHistory } from './ThroughputChart'

type Tab = 'overview' | 'topics' | 'services' | 'actions' | 'logs'

export default function Dashboard() {
  const [broker, setBroker] = useState<BrokerInfo>(EMPTY_BROKER)
  const [topics, setTopics] = useState<TopicInfo[]>([])
  const [services] = useState<ServiceInfo[]>([])
  const [actions] = useState<ActionInfo[]>([])
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [throughput, setThroughput] = useState(() => generateThroughputHistory(0))
  const [activeTab, setActiveTab] = useState<Tab>('overview')
  const seenLogIds = useRef(new Set<string>())

  const poll = useCallback(async () => {
    try {
      const [status, topicList] = await Promise.all([fetchStatus(), fetchTopics()])
      setBroker(status)
      setTopics((prev) => mergeTopics(prev, topicList))
      setThroughput((prev) => {
        const t = new Date()
        const label = t.toLocaleTimeString('zh-CN', {
          minute: '2-digit',
          second: '2-digit',
          hour12: false,
        })
        return [...prev.slice(1), { t: label, msgs: status.msgPerSec, bytes: status.bytesPerSec }]
      })
    } catch {
      setBroker((prev) => ({ ...prev, status: 'OFFLINE' }))
    }
  }, [])

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
    <div className="flex flex-col h-screen bg-[#141618] overflow-hidden">
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
            />
          )}
          {activeTab === 'topics' && <TopicTable topics={topics} />}
          {activeTab === 'services' && (
            <ServiceActionTable services={services} actions={actions} />
          )}
          {activeTab === 'actions' && (
            <ServiceActionTable services={services} actions={actions} />
          )}
          {activeTab === 'logs' && (
            <div style={{ height: 'calc(100vh - 76px)' }}>
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
}: {
  broker: BrokerInfo
  topics: TopicInfo[]
  services: ServiceInfo[]
  actions: ActionInfo[]
  logs: LogEntry[]
  throughput: ReturnType<typeof generateThroughputHistory>
}) {
  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-[1fr_220px_300px] gap-3">
        <MetricCards broker={broker} />
        <BrokerOverview broker={broker} />
        <ThroughputChart data={throughput} />
      </div>

      <TopicTable topics={topics} />

      <ServiceActionTable services={services} actions={actions} />

      <div style={{ height: '220px' }}>
        <EventStream logs={logs} />
      </div>
    </div>
  )
}
