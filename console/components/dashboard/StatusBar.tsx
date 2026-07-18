'use client'

import { useState, useEffect } from 'react'
import { type BrokerInfo, fmtUptime, fmtBytes, fmtNum } from '@/lib/mock-data'
import { Activity, Server, Zap, AlertTriangle, Clock } from 'lucide-react'

interface StatusBarProps {
  broker: BrokerInfo
}

export default function StatusBar({ broker }: StatusBarProps) {
  const statusColor =
    broker.status === 'ONLINE' ? 'text-[#22c55e]' :
    broker.status === 'DEGRADED' ? 'text-[#f59e0b]' :
    'text-[#ef4444]'

  const statusPulse =
    broker.status === 'ONLINE' ? 'pulse-green bg-[#22c55e]' :
    broker.status === 'DEGRADED' ? 'pulse-amber bg-[#f59e0b]' :
    'pulse-red bg-[#ef4444]'

  return (
    <header className="h-9 bg-[#0e1012] border-b border-[#2a2f35] flex items-center px-4 gap-6 shrink-0 overflow-x-auto">
      {/* 品牌 */}
      <div className="flex items-center gap-2 shrink-0">
        <Zap size={14} className="text-[#00d4ff]" />
        <span className="font-mono text-xs font-bold tracking-widest text-[#00d4ff] uppercase">robot-bus</span>
        <span className="font-mono text-[10px] text-[#5a6370] ml-1">v{broker.version}</span>
      </div>

      <div className="w-px h-5 bg-[#2a2f35] shrink-0" />

      {/* Broker 状态 */}
      <div className="flex items-center gap-1.5 shrink-0">
        <span className={`w-2 h-2 rounded-full ${statusPulse}`} />
        <span className={`font-mono text-xs font-semibold ${statusColor}`}>{broker.status}</span>
      </div>

      <div className="w-px h-5 bg-[#2a2f35] shrink-0" />

      {/* 核心指标 */}
      <div className="flex items-center gap-5 shrink-0">
        <Metric icon={<Activity size={11} className="text-[#5a6370]" />} label="MSG/S" value={fmtNum(broker.msgPerSec)} accent />
        <Metric icon={<Server size={11} className="text-[#5a6370]" />} label="BW" value={`${fmtBytes(broker.bytesPerSec)}/s`} />
        <Metric label="TOTAL" value={fmtNum(broker.totalMessages)} />
        <Metric label="UPTIME" value={fmtUptime(broker.uptime)} />
        <Metric label="PID" value={String(broker.pid)} />
      </div>

      <div className="w-px h-5 bg-[#2a2f35] shrink-0" />

      {/* 地址 */}
      <div className="flex items-center gap-4 shrink-0">
        <AddrChip label="gRPC" addr={broker.grpcAddr} />
        <AddrChip label="Web" addr={broker.webAddr} />
      </div>

      {/* 错误计数 */}
      {broker.totalErrors > 0 && (
        <>
          <div className="w-px h-5 bg-[#2a2f35] shrink-0" />
          <div className="flex items-center gap-1.5 shrink-0">
            <AlertTriangle size={11} className="text-[#ef4444]" />
            <span className="font-mono text-xs text-[#ef4444]">{broker.totalErrors} ERR</span>
          </div>
        </>
      )}

      <div className="flex-1" />

      {/* 当前时间 */}
      <LiveClock />
    </header>
  )
}

function Metric({
  icon, label, value, accent,
}: {
  icon?: React.ReactNode
  label: string
  value: string
  accent?: boolean
}) {
  return (
    <div className="flex items-center gap-1">
      {icon}
      <span className="font-mono text-[10px] text-[#5a6370] uppercase">{label}</span>
      <span className={`font-mono text-xs font-semibold tabular-nums ${accent ? 'text-[#00d4ff]' : 'text-[#c8ced6]'}`}>
        {value}
      </span>
    </div>
  )
}

function AddrChip({ label, addr }: { label: string; addr: string }) {
  return (
    <div className="flex items-center gap-1">
      <span className="font-mono text-[10px] text-[#5a6370]">{label}:</span>
      <span className="font-mono text-[10px] text-[#c8ced6] bg-[#1a1d20] border border-[#2a2f35] px-1.5 py-px rounded">{addr}</span>
    </div>
  )
}

function LiveClock() {
  const [time, setTime] = useState(() => new Date().toLocaleTimeString('zh-CN', { hour12: false }))
  useEffect(() => {
    const id = setInterval(() => {
      setTime(new Date().toLocaleTimeString('zh-CN', { hour12: false }))
    }, 1000)
    return () => clearInterval(id)
  }, [])
  return (
    <span className="flex items-center gap-1 shrink-0">
      <Clock size={11} className="text-[#5a6370]" />
      <span className="font-mono text-[10px] text-[#5a6370] tabular-nums">{time}</span>
    </span>
  )
}
