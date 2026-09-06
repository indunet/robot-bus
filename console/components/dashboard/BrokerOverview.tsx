'use client'

import { formatWsRpcAddr, type BrokerInfo } from '@/lib/mock-data'
import { Server } from 'lucide-react'
import { useI18n } from '@/lib/i18n'

interface Props {
  broker: BrokerInfo
}

type BusEndpoint = { label: string; addr: string }

export default function BrokerOverview({ broker }: Props) {
  const { t } = useI18n()
  const connecting = broker.status === 'CONNECTING'
  const offline = broker.status === 'OFFLINE'

  const buses: BusEndpoint[] = [
    { label: t('brokerWs'), addr: formatWsRpcAddr(broker.grpcAddr) },
    { label: t('brokerMsgXSub'), addr: broker.msgBusXSub },
    { label: t('brokerMsgXPub'), addr: broker.msgBusXPub },
    { label: t('brokerSvcFE'), addr: broker.svcFE },
    { label: t('brokerSvcBE'), addr: broker.svcBE },
    { label: t('brokerActFE'), addr: broker.actFE },
    { label: t('brokerActBE'), addr: broker.actBE },
  ]

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm h-full">
      <PanelHeader icon={<Server size={14} />} title={t('brokerTitle')} sub={t('brokerSub')} />
      <div className="p-3 grid grid-cols-1 gap-1.5">
        {buses.map((b) => {
          const alive = !connecting && !offline
          return (
            <div key={b.label} className="flex items-center gap-2 h-7">
              <span
                className={`w-2 h-2 rounded-full shrink-0 ${
                  connecting
                    ? 'bg-bus-cyan pulse-cyan'
                    : alive
                      ? 'bg-bus-green pulse-green'
                      : 'bg-bus-red pulse-red'
                }`}
              />
              <span className="font-mono text-xs text-bus-muted w-24 shrink-0">{b.label}</span>
              {connecting ? (
                <span className="flex-1 h-3 rounded-sm status-shimmer" aria-hidden />
              ) : (
                <span className="font-mono text-xs text-bus-text flex-1 truncate">{b.addr}</span>
              )}
              <span
                className={`font-mono text-xs shrink-0 ${
                  connecting ? 'text-bus-cyan' : alive ? 'text-bus-green' : 'text-bus-red'
                }`}
              >
                {connecting ? t('loading') : alive ? t('listen') : t('down')}
              </span>
            </div>
          )
        })}
      </div>
    </section>
  )
}

export function PanelHeader({
  icon,
  title,
  sub,
  subClassName = 'text-bus-muted',
  trailing,
}: {
  icon?: React.ReactNode
  title: string
  sub?: string
  /** Accent for count / status subtitle (default muted). */
  subClassName?: string
  /** Optional controls on the right (e.g. window picker). */
  trailing?: React.ReactNode
}) {
  const { labelCase } = useI18n()
  return (
    <div className="flex items-center gap-2 px-3 h-9 border-b border-bus-border">
      <div className="flex items-center gap-2 min-w-0 flex-1">
        {icon ? (
          <span className="text-bus-cyan shrink-0">{icon}</span>
        ) : (
          <div className="w-0.5 h-4 bg-bus-cyan shrink-0" />
        )}
        <span className={`font-mono text-xs font-bold text-bus-text tracking-widest truncate ${labelCase}`}>
          {title}
        </span>
        {sub && (
          <span className={`font-mono text-xs font-medium tabular-nums shrink-0 ${subClassName}`}>
            {sub}
          </span>
        )}
      </div>
      {trailing ? <div className="flex items-center gap-2 shrink-0">{trailing}</div> : null}
    </div>
  )
}
