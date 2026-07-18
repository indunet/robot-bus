'use client'

import { useState, useEffect, useRef, useCallback } from 'react'
import {
  INITIAL_BROKER, INITIAL_TOPICS, INITIAL_SERVICES, INITIAL_ACTIONS,
  generateInitialLogs, type BrokerInfo, type TopicInfo, type LogEntry,
  type ServiceInfo, type ActionInfo,
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

// ─── 新消息模板 ──────────────────────────────────────────────────────────────
const LIVE_MESSAGES = [
  { level: 'INFO'  as const, source: 'msg-bus',        message: 'Topic /robot/base/odom: new subscriber connected' },
  { level: 'DEBUG' as const, source: 'msg-bus',        message: 'Frame dispatched to 3 subscribers in 0.12ms' },
  { level: 'INFO'  as const, source: 'svc-gateway',    message: 'ServiceGateway.Call /system/get_params latency=11ms' },
  { level: 'DEBUG' as const, source: 'broker',         message: 'Heartbeat tick — all buses alive' },
  { level: 'WARN'  as const, source: 'metrics',        message: '/map/costmap idle > 15s, no publisher detected' },
  { level: 'INFO'  as const, source: 'action-gateway', message: 'Action feedback /robot/navigate_to_pose: dist_remaining=0.42m' },
  { level: 'INFO'  as const, source: 'grpc-web',       message: 'MessageGateway.Subscribe /sensor/lidar/points: stream open' },
  { level: 'ERROR' as const, source: 'svc-gateway',    message: '/sensor/calibrate: worker returned error code 3' },
]

export default function Dashboard() {
  const [broker, setBroker]     = useState<BrokerInfo>(INITIAL_BROKER)
  const [topics, setTopics]     = useState<TopicInfo[]>(INITIAL_TOPICS)
  const [services]              = useState<ServiceInfo[]>(INITIAL_SERVICES)
  const [actions]               = useState<ActionInfo[]>(INITIAL_ACTIONS)
  const [logs, setLogs]         = useState<LogEntry[]>(() => generateInitialLogs(50))
  const [throughput, setThroughput] = useState(() => generateThroughputHistory(INITIAL_BROKER.msgPerSec))
  const [activeTab, setActiveTab] = useState<Tab>('overview')
  const logCountRef = useRef(50)

  // ─── 模拟实时数据更新 ──────────────────────────────────────────────────────
  const tick = useCallback(() => {
    // 更新 Broker 指标
    setBroker((prev) => ({
      ...prev,
      uptime: prev.uptime + 1,
      msgPerSec: Math.max(0, prev.msgPerSec + Math.round((Math.random() - 0.5) * 80)),
      bytesPerSec: Math.max(0, prev.bytesPerSec + Math.round((Math.random() - 0.5) * 30000)),
      totalMessages: prev.totalMessages + Math.round(prev.msgPerSec),
    }))

    // 更新 Topic 速率
    setTopics((prev) =>
      prev.map((t) => {
        const newRate = Math.max(0, t.msgPerSec + Math.round((Math.random() - 0.5) * (t.msgPerSec * 0.3 + 2)))
        const newSparkline = [...t.sparkline.slice(1), newRate]
        return {
          ...t,
          msgPerSec: newRate,
          bytesPerSec: newRate > 0 ? newRate * Math.round(t.bytesPerSec / (t.msgPerSec || 1)) : 0,
          totalMsgs: t.totalMsgs + newRate,
          lastSeen: newRate > 0 ? Date.now() : t.lastSeen,
          sparkline: newSparkline,
        }
      })
    )

    // 更新吞吐量图表
    setThroughput((prev) => {
      const t = new Date()
      const label = t.toLocaleTimeString('zh-CN', { minute: '2-digit', second: '2-digit', hour12: false })
      const newPoint = {
        t: label,
        msgs: Math.max(0, (prev[prev.length - 1]?.msgs ?? 1200) + Math.round((Math.random() - 0.5) * 150)),
        bytes: 0,
      }
      return [...prev.slice(1), newPoint]
    })

    // 随机注入新日志（约 40% 概率）
    if (Math.random() < 0.4) {
      const tmpl = LIVE_MESSAGES[Math.floor(Math.random() * LIVE_MESSAGES.length)]
      const newEntry: LogEntry = {
        ...tmpl,
        id: `live-${++logCountRef.current}`,
        ts: Date.now(),
      }
      setLogs((prev) => [...prev.slice(-200), newEntry])
    }
  }, [])

  useEffect(() => {
    const id = setInterval(tick, 1000)
    return () => clearInterval(id)
  }, [tick])

  return (
    <div className="flex flex-col h-screen bg-[#141618] overflow-hidden">
      <StatusBar broker={broker} />

      <div className="flex flex-1 min-h-0">
        <Sidebar active={activeTab} onSelect={setActiveTab} />

        <main className="flex-1 overflow-y-auto p-3">
          {activeTab === 'overview' && (
            <OverviewLayout broker={broker} topics={topics} services={services} actions={actions} logs={logs} throughput={throughput} />
          )}
          {activeTab === 'topics' && (
            <TopicTable topics={topics} />
          )}
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

// ─── Overview 布局：顶部指标 + 中部表格 + 底部事件 ────────────────────────────
function OverviewLayout({
  broker, topics, services, actions, logs, throughput,
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
      {/* 行 1：指标卡 + Broker 端点 + 吞吐图 */}
      <div className="grid grid-cols-[1fr_220px_300px] gap-3">
        <MetricCards broker={broker} />
        <BrokerOverview broker={broker} />
        <ThroughputChart data={throughput} />
      </div>

      {/* 行 2：Topic 表格（全宽） */}
      <TopicTable topics={topics} />

      {/* 行 3：Service + Action 表格 */}
      <ServiceActionTable services={services} actions={actions} />

      {/* 行 4：事件流（固定高度可滚动） */}
      <div style={{ height: '220px' }}>
        <EventStream logs={logs} />
      </div>
    </div>
  )
}
