'use client'

import { type BrokerInfo } from '@/lib/mock-data'
import { Server } from 'lucide-react'
import { useI18n } from '@/lib/i18n'

interface Props {
  broker: BrokerInfo
}

type BusEndpoint = { label: string; addr: string; alive: boolean }

export default function BrokerOverview({ broker }: Props) {
  const { t } = useI18n()

  const buses: BusEndpoint[] = [
    { label: 'MSG XSUB', addr: broker.msgBusXSub, alive: broker.status !== 'OFFLINE' },
    { label: 'MSG XPUB', addr: broker.msgBusXPub, alive: broker.status !== 'OFFLINE' },
    { label: 'SVC  FE',  addr: broker.svcFE,      alive: broker.status !== 'OFFLINE' },
    { label: 'SVC  BE',  addr: broker.svcBE,      alive: broker.status !== 'OFFLINE' },
    { label: 'ACT  FE',  addr: broker.actFE,       alive: broker.status !== 'OFFLINE' },
    { label: 'ACT  BE',  addr: broker.actBE,       alive: broker.status !== 'OFFLINE' },
  ]

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm h-full">
      <PanelHeader icon={<Server size={14} />} title={t('brokerTitle')} sub={t('brokerSub')} />
      <div className="p-3 grid grid-cols-1 gap-1.5">
        {buses.map((b) => (
          <div key={b.label} className="flex items-center gap-2 h-7">
            <span className={`w-2 h-2 rounded-full shrink-0 ${b.alive ? 'bg-bus-green pulse-green' : 'bg-bus-red pulse-red'}`} />
            <span className="font-mono text-xs text-bus-muted w-[4.5rem] shrink-0">{b.label}</span>
            <span className="font-mono text-xs text-bus-text flex-1 truncate">{b.addr}</span>
            <span className={`font-mono text-xs ${b.alive ? 'text-bus-green' : 'text-bus-red'}`}>
              {b.alive ? t('listen') : t('down')}
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
    <div className="flex items-center justify-between px-3 h-9 border-b border-bus-border">
      <div className="flex items-center gap-2">
        {icon ? (
          <span className="text-bus-cyan">{icon}</span>
        ) : (
          <div className="w-0.5 h-4 bg-bus-cyan" />
        )}
        <span className="font-mono text-xs font-bold text-bus-text tracking-widest uppercase">{title}</span>
      </div>
      {sub && <span className="font-mono text-xs text-bus-muted">{sub}</span>}
    </div>
  )
}
