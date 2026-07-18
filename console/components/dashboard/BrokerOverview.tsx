'use client'

import { type BrokerInfo } from '@/lib/mock-data'
import { Server } from 'lucide-react'

interface Props {
  broker: BrokerInfo
}

type BusEndpoint = { label: string; addr: string; alive: boolean }

export default function BrokerOverview({ broker }: Props) {
  const buses: BusEndpoint[] = [
    { label: 'MSG XSUB', addr: broker.msgBusXSub, alive: broker.status !== 'OFFLINE' },
    { label: 'MSG XPUB', addr: broker.msgBusXPub, alive: broker.status !== 'OFFLINE' },
    { label: 'SVC  FE',  addr: broker.svcFE,      alive: broker.status !== 'OFFLINE' },
    { label: 'SVC  BE',  addr: broker.svcBE,      alive: broker.status !== 'OFFLINE' },
    { label: 'ACT  FE',  addr: broker.actFE,       alive: broker.status !== 'OFFLINE' },
    { label: 'ACT  BE',  addr: broker.actBE,       alive: broker.status !== 'OFFLINE' },
  ]

  return (
    <section className="border border-[#2a2f35] bg-[#1a1d20] rounded-sm">
      <PanelHeader icon={<Server size={12} />} title="BROKER" sub="端点健康" />
      <div className="p-3 grid grid-cols-1 gap-1">
        {buses.map((b) => (
          <div key={b.label} className="flex items-center gap-2 h-6">
            <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${b.alive ? 'bg-[#22c55e] pulse-green' : 'bg-[#ef4444] pulse-red'}`} />
            <span className="font-mono text-[10px] text-[#5a6370] w-16 shrink-0">{b.label}</span>
            <span className="font-mono text-[10px] text-[#c8ced6] flex-1 truncate">{b.addr}</span>
            <span className={`font-mono text-[10px] ${b.alive ? 'text-[#22c55e]' : 'text-[#ef4444]'}`}>
              {b.alive ? 'LISTEN' : 'DOWN'}
            </span>
          </div>
        ))}
      </div>
    </section>
  )
}

export function PanelHeader({
  icon,
  title,
  sub,
}: {
  icon?: React.ReactNode
  title: string
  sub?: string
}) {
  return (
    <div className="flex items-center justify-between px-3 h-8 border-b border-[#2a2f35]">
      <div className="flex items-center gap-2">
        {icon ? (
          <span className="text-[#00d4ff]">{icon}</span>
        ) : (
          <div className="w-0.5 h-3.5 bg-[#00d4ff]" />
        )}
        <span className="font-mono text-[11px] font-bold text-[#c8ced6] tracking-widest uppercase">{title}</span>
      </div>
      {sub && <span className="font-mono text-[10px] text-[#5a6370]">{sub}</span>}
    </div>
  )
}
