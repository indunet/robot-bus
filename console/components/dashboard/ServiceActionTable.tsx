'use client'

import { useMemo, useState } from 'react'
import { type ServiceInfo, type ActionInfo, fmtNum, fmtRate } from '@/lib/mock-data'
import { PanelHeader } from './BrokerOverview'
import NameFilter, { nameMatches } from './NameFilter'
import { Cpu, GitBranch } from 'lucide-react'
import { fmtAgeLocalized, useI18n } from '@/lib/i18n'

interface Props {
  services: ServiceInfo[]
  actions: ActionInfo[]
  /** When set, only render that side (for dedicated tabs). */
  mode?: 'both' | 'services' | 'actions'
  maxBodyHeight?: string
}

export default function ServiceActionTable({
  services,
  actions,
  mode = 'both',
  maxBodyHeight,
}: Props) {
  const showServices = mode === 'both' || mode === 'services'
  const showActions = mode === 'both' || mode === 'actions'

  if (mode === 'services') {
    return <ServicesPanel services={services} maxBodyHeight={maxBodyHeight} />
  }
  if (mode === 'actions') {
    return <ActionsPanel actions={actions} maxBodyHeight={maxBodyHeight} />
  }

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
      {showServices && <ServicesPanel services={services} maxBodyHeight={maxBodyHeight} />}
      {showActions && <ActionsPanel actions={actions} maxBodyHeight={maxBodyHeight} />}
    </div>
  )
}

function ServicesPanel({
  services,
  maxBodyHeight,
}: {
  services: ServiceInfo[]
  maxBodyHeight?: string
}) {
  const { t } = useI18n()
  const [query, setQuery] = useState('')
  const sorted = useMemo(
    () =>
      [...services]
        .filter((s) => nameMatches(query, s.name))
        .sort((a, b) => b.callsPerSec - a.callsPerSec || b.calls - a.calls),
    [services, query],
  )
  const filtering = query.trim().length > 0
  const grid = 'grid-cols-[1fr_72px_64px_64px_48px_64px_72px]'

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm h-full flex flex-col min-h-0">
      <PanelHeader
        icon={<Cpu size={14} />}
        title={t('servicesTitle')}
        sub={
          filtering
            ? t('nameFilterHits', { n: sorted.length, total: services.length })
            : t('servicesKnown', { n: services.length })
        }
        subClassName="text-bus-amber"
        trailing={<NameFilter value={query} onChange={setQuery} />}
      />
      <ColHeader
        cols={[t('colService'), t('colCallsS'), t('colCalls'), t('colWorkers'), t('colErr'), t('colP99'), t('colLast')]}
        widths={grid}
      />
      <div className="overflow-y-auto bus-scroll" style={maxBodyHeight ? { maxHeight: maxBodyHeight } : undefined}>
        {sorted.length === 0 ? (
          <EmptyRow text={filtering ? t('nameFilterEmpty') : t('servicesEmpty')} />
        ) : (
          sorted.map((s) => (
            <div
              key={s.name}
              className={`grid ${grid} items-center px-3 h-9 border-b border-[#1f2428] hover:bg-[#1f2428] transition-colors cursor-default`}
            >
              <span className="font-mono text-[13px] text-bus-text truncate">{s.name}</span>
              <span className={`font-mono text-[13px] tabular-nums text-right ${s.callsPerSec > 0 ? 'text-bus-amber' : 'text-bus-muted'}`}>
                {s.callsPerSec > 0 ? fmtRate(s.callsPerSec) : '—'}
              </span>
              <span className="font-mono text-[13px] text-bus-muted tabular-nums text-right">{fmtNum(s.calls)}</span>
              <span className="font-mono text-[13px] text-bus-muted tabular-nums text-right">{s.workers}</span>
              <span className={`font-mono text-[13px] tabular-nums text-right ${s.errors + s.timeouts > 0 ? 'text-bus-red' : 'text-bus-muted'}`}>
                {s.errors + s.timeouts > 0 ? s.errors + s.timeouts : '—'}
              </span>
              <span className="font-mono text-[13px] text-bus-cyan tabular-nums text-right">
                {s.calls > 0 ? s.avgLatencyMs : '—'}
              </span>
              <span className="font-mono text-[13px] text-bus-muted tabular-nums text-right">
                <LastSeen ts={s.lastCallAt} />
              </span>
            </div>
          ))
        )}
      </div>
    </section>
  )
}

function ActionsPanel({
  actions,
  maxBodyHeight,
}: {
  actions: ActionInfo[]
  maxBodyHeight?: string
}) {
  const { t } = useI18n()
  const [query, setQuery] = useState('')
  const sorted = useMemo(
    () =>
      [...actions]
        .filter((a) => nameMatches(query, a.name))
        .sort((a, b) => b.runsPerSec - a.runsPerSec || b.runs - a.runs),
    [actions, query],
  )
  const filtering = query.trim().length > 0
  const grid = 'grid-cols-[1fr_72px_56px_64px_48px_64px_72px]'

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm h-full flex flex-col min-h-0">
      <PanelHeader
        icon={<GitBranch size={14} />}
        title={t('actionsTitle')}
        sub={
          filtering
            ? t('nameFilterHits', { n: sorted.length, total: actions.length })
            : t('actionsKnown', { n: actions.length })
        }
        subClassName="text-bus-green"
        trailing={<NameFilter value={query} onChange={setQuery} />}
      />
      <ColHeader
        cols={[t('colAction'), t('colRunsS'), t('colRuns'), t('colActive'), t('colErr'), t('colAvgDur'), t('colLast')]}
        widths={grid}
      />
      <div className="overflow-y-auto bus-scroll" style={maxBodyHeight ? { maxHeight: maxBodyHeight } : undefined}>
        {sorted.length === 0 ? (
          <EmptyRow text={filtering ? t('nameFilterEmpty') : t('actionsEmpty')} />
        ) : (
          sorted.map((a) => (
            <div
              key={a.name}
              className={`grid ${grid} items-center px-3 h-9 border-b border-[#1f2428] hover:bg-[#1f2428] transition-colors cursor-default`}
            >
              <span className="font-mono text-[13px] text-bus-text truncate">{a.name}</span>
              <span className={`font-mono text-[13px] tabular-nums text-right ${a.runsPerSec > 0 ? 'text-bus-green' : 'text-bus-muted'}`}>
                {a.runsPerSec > 0 ? fmtRate(a.runsPerSec) : '—'}
              </span>
              <span className="font-mono text-[13px] text-bus-muted tabular-nums text-right">{fmtNum(a.runs)}</span>
              <span className={`font-mono text-[13px] tabular-nums text-right ${a.active > 0 ? 'text-bus-green' : 'text-bus-muted'}`}>
                {a.active > 0 ? a.active : '—'}
              </span>
              <span className={`font-mono text-[13px] tabular-nums text-right ${a.errors > 0 ? 'text-bus-red' : 'text-bus-muted'}`}>
                {a.errors > 0 ? a.errors : '—'}
              </span>
              <span className="font-mono text-[13px] text-bus-cyan tabular-nums text-right">
                {a.runs > 0 ? a.avgDurationMs : '—'}
              </span>
              <span className="font-mono text-[13px] text-bus-muted tabular-nums text-right">
                <LastSeen ts={a.lastRunAt} />
              </span>
            </div>
          ))
        )}
      </div>
    </section>
  )
}

function LastSeen({ ts }: { ts: number }) {
  const { t } = useI18n()
  return <>{fmtAgeLocalized(ts, t)}</>
}

function EmptyRow({ text }: { text: string }) {
  return (
    <div className="flex items-center justify-center h-14 text-bus-muted font-mono text-xs">
      {text}
    </div>
  )
}

function ColHeader({ cols, widths }: { cols: string[]; widths: string }) {
  const { labelCase } = useI18n()
  return (
    <div className={`grid ${widths} items-center px-3 h-8 border-b border-bus-border bg-bus-bg`}>
      {cols.map((c, i) => (
        <span key={`${c}-${i}`} className={`font-mono text-xs text-bus-muted tracking-wider ${labelCase} ${i > 0 ? 'text-right' : ''}`}>
          {c}
        </span>
      ))}
    </div>
  )
}
