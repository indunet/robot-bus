'use client'

import { useMemo, useState } from 'react'
import { type BridgeInfo, type BridgeRouteInfo, fmtNum } from '@/lib/mock-data'
import { PanelHeader } from './BrokerOverview'
import NameFilter, { nameMatches } from './NameFilter'
import TruncateTip from './TruncateTip'
import { ArrowLeftRight } from 'lucide-react'
import { useI18n } from '@/lib/i18n'

interface Props {
  bridges: BridgeInfo[]
  maxBodyHeight?: string
}

const COLS =
  'grid-cols-[72px_88px_minmax(0,1fr)_minmax(0,1fr)_minmax(0,0.9fr)_minmax(0,1.1fr)_72px_88px_64px]'

export default function BridgeTable({ bridges, maxBodyHeight }: Props) {
  const { t, labelCase } = useI18n()
  const [query, setQuery] = useState('')
  const rows = useMemo(() => {
    const out: { bridge: string; route: BridgeRouteInfo; key: string }[] = []
    for (const bridge of bridges) {
      for (const route of bridge.routes) {
        if (
          !nameMatches(
            query,
            route.rosName,
            route.busName,
            route.typeName,
            bridge.bridgeName,
          )
        ) {
          continue
        }
        out.push({
          bridge: bridge.bridgeName,
          route,
          key: `${bridge.bridgeId}:${route.kind}:${route.direction}:${route.rosName}:${route.busName}`,
        })
      }
    }
    return out
  }, [bridges, query])
  const filtering = query.trim().length > 0
  const headers = [
    t('colKind'),
    t('colDirection'),
    t('colRos'),
    t('colBus'),
    t('colType'),
    t('colQos'),
    t('colRxTx'),
    t('colDrops'),
    t('colIdle'),
  ]

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm flex flex-col min-h-0 min-w-0 h-full">
      <PanelHeader
        icon={<ArrowLeftRight size={14} />}
        title={t('bridgesTitle')}
        sub={
          filtering
            ? t('nameFilterHits', { n: rows.length, total: bridges.reduce((n, b) => n + b.routes.length, 0) })
            : t('bridgesActive', { n: bridges.length, r: rows.length })
        }
        subClassName="text-bus-cyan"
        trailing={<NameFilter value={query} onChange={setQuery} />}
      />

      <div className="overflow-x-auto bus-scroll flex-1 min-h-0">
        <div className="min-w-[1080px] h-full flex flex-col">
          <div className={`grid ${COLS} items-center px-3 h-8 border-b border-bus-border bg-bus-bg shrink-0`}>
            {headers.map((h, i) => (
              <span
                key={`${h}-${i}`}
                className={`font-mono text-xs text-bus-muted tracking-wider ${labelCase} ${i > 5 ? 'text-right' : ''}`}
              >
                {h}
              </span>
            ))}
          </div>
          <div
            className="overflow-y-auto bus-scroll"
            style={maxBodyHeight ? { maxHeight: maxBodyHeight } : undefined}
          >
            {rows.length === 0 ? (
              <div className="flex items-center justify-center h-14 text-bus-muted font-mono text-xs">
                {filtering ? t('nameFilterEmpty') : t('bridgesEmpty')}
              </div>
            ) : (
              rows.map((row) => <BridgeRow key={row.key} bridge={row.bridge} route={row.route} />)
            )}
          </div>
        </div>
      </div>
    </section>
  )
}

function BridgeRow({ bridge, route }: { bridge: string; route: BridgeRouteInfo }) {
  const { t } = useI18n()
  const drops = route.convertFail + route.decodeFail + route.publishFail
  const alert = route.idle || drops > 0
  const rowClass = alert ? 'bg-bus-amber/5' : ''

  return (
    <div
      className={`grid ${COLS} items-center px-3 h-9 border-b border-[#1f2428] transition-colors cursor-default hover:bg-[#1f2428] ${rowClass}`}
    >
      <span className="font-mono text-[12px] text-bus-muted truncate">{route.kind}</span>
      <span className="font-mono text-[12px] text-bus-cyan truncate">{route.direction}</span>
      <TruncateTip text={route.rosName} className="font-mono text-[13px] text-bus-text" />
      <TruncateTip text={route.busName} className="font-mono text-[13px] text-bus-text" />
      <TruncateTip
        text={route.typeName || undefined}
        className="font-mono text-[12px] text-[#6b8294]"
      />
      <TruncateTip
        text={`ros=${route.rosQos} bus=${route.busQos}`}
        className="font-mono text-[11px] text-bus-muted"
      />
      <span className="font-mono text-[13px] text-bus-text tabular-nums text-right">
        {fmtNum(route.rx)}/{fmtNum(route.tx)}
      </span>
      <span
        className={`font-mono text-[13px] tabular-nums text-right ${drops > 0 ? 'text-bus-amber' : 'text-bus-muted'}`}
      >
        {drops > 0 ? fmtNum(drops) : '—'}
      </span>
      <span
        className={`font-mono text-[11px] text-right ${route.idle ? 'text-bus-amber' : 'text-bus-muted'}`}
        title={bridge}
      >
        {route.idle ? t('bridgeIdle') : route.lazy && !route.enabled ? t('bridgeLazy') : '—'}
      </span>
    </div>
  )
}
