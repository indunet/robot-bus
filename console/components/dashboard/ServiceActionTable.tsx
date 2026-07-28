'use client'

import { type ServiceInfo, type ActionInfo, fmtNum } from '@/lib/mock-data'
import { PanelHeader } from './BrokerOverview'
import { Cpu, GitBranch } from 'lucide-react'
import { fmtAgeLocalized, useI18n } from '@/lib/i18n'

interface Props {
  services: ServiceInfo[]
  actions: ActionInfo[]
  /** When set, only render that side (for dedicated tabs). */
  mode?: 'both' | 'services' | 'actions'
}

export default function ServiceActionTable({ services, actions, mode = 'both' }: Props) {
  const showServices = mode === 'both' || mode === 'services'
  const showActions = mode === 'both' || mode === 'actions'

  if (mode === 'services') {
    return <ServicesPanel services={services} />
  }
  if (mode === 'actions') {
    return <ActionsPanel actions={actions} />
  }

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
      {showServices && <ServicesPanel services={services} />}
      {showActions && <ActionsPanel actions={actions} />}
    </div>
  )
}

function ServicesPanel({ services }: { services: ServiceInfo[] }) {
  const { t } = useI18n()
  const sorted = [...services].sort((a, b) => b.calls - a.calls)

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm">
      <PanelHeader
        icon={<Cpu size={14} />}
        title={t('servicesTitle')}
        sub={t('servicesKnown', { n: services.length })}
      />
      <ColHeader
        cols={[t('colService'), t('colCalls'), t('colWorkers'), t('colErr'), t('colP99'), t('colLast')]}
        widths="grid-cols-[1fr_56px_64px_44px_64px_72px]"
      />
      {sorted.length === 0 ? (
        <EmptyRow text={t('servicesEmpty')} />
      ) : (
        sorted.map((s) => (
          <div
            key={s.name}
            className="grid grid-cols-[1fr_56px_64px_44px_64px_72px] items-center px-3 h-9 border-b border-[#1f2428] hover:bg-[#1f2428] transition-colors cursor-default"
          >
            <span className="font-mono text-[13px] text-bus-text truncate">{s.name}</span>
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
    </section>
  )
}

function ActionsPanel({ actions }: { actions: ActionInfo[] }) {
  const { t } = useI18n()
  const sorted = [...actions].sort((a, b) => b.runs - a.runs)

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm">
      <PanelHeader
        icon={<GitBranch size={14} />}
        title={t('actionsTitle')}
        sub={t('actionsKnown', { n: actions.length })}
      />
      <ColHeader
        cols={[t('colAction'), t('colRuns'), t('colActive'), t('colErr'), t('colAvgDur'), t('colLast')]}
        widths="grid-cols-[1fr_56px_64px_44px_64px_72px]"
      />
      {sorted.length === 0 ? (
        <EmptyRow text={t('actionsEmpty')} />
      ) : (
        sorted.map((a) => (
          <div
            key={a.name}
            className="grid grid-cols-[1fr_56px_64px_44px_64px_72px] items-center px-3 h-9 border-b border-[#1f2428] hover:bg-[#1f2428] transition-colors cursor-default"
          >
            <span className="font-mono text-[13px] text-bus-text truncate">{a.name}</span>
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
  return (
    <div className={`grid ${widths} items-center px-3 h-8 border-b border-bus-border bg-bus-bg`}>
      {cols.map((c, i) => (
        <span key={c} className={`font-mono text-xs text-bus-muted uppercase tracking-wider ${i > 0 ? 'text-right' : ''}`}>
          {c}
        </span>
      ))}
    </div>
  )
}
