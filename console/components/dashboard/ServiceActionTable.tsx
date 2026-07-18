'use client'

import { type ServiceInfo, type ActionInfo, fmtAge, fmtNum } from '@/lib/mock-data'
import { PanelHeader } from './BrokerOverview'
import { Cpu, GitBranch } from 'lucide-react'

interface Props {
  services: ServiceInfo[]
  actions: ActionInfo[]
}

export default function ServiceActionTable({ services, actions }: Props) {
  return (
    <div className="grid grid-cols-2 gap-3">
      {/* Services */}
      <section className="border border-[#2a2f35] bg-[#1a1d20] rounded-sm">
        <PanelHeader icon={<Cpu size={12} />} title="SERVICES" sub={`${services.length} 已知`} />
        <ColHeader cols={['SERVICE', 'CALLS', 'ERR', 'P99 ms', 'LAST']} widths="grid-cols-[1fr_50px_40px_55px_70px]" />
        {services.map((s) => (
          <div
            key={s.name}
            className="grid grid-cols-[1fr_50px_40px_55px_70px] items-center px-3 h-8 border-b border-[#1f2428] hover:bg-[#1f2428] transition-colors cursor-default"
          >
            <span className="font-mono text-[11px] text-[#c8ced6] truncate">{s.name}</span>
            <span className="font-mono text-[11px] text-[#5a6370] tabular-nums text-right">{fmtNum(s.calls)}</span>
            <span className={`font-mono text-[11px] tabular-nums text-right ${s.errors + s.timeouts > 0 ? 'text-[#ef4444]' : 'text-[#5a6370]'}`}>
              {s.errors + s.timeouts > 0 ? s.errors + s.timeouts : '—'}
            </span>
            <span className="font-mono text-[11px] text-[#00d4ff] tabular-nums text-right">{s.avgLatencyMs}</span>
            <span className="font-mono text-[11px] text-[#5a6370] tabular-nums text-right">{fmtAge(s.lastCallAt)}</span>
          </div>
        ))}
      </section>

      {/* Actions */}
      <section className="border border-[#2a2f35] bg-[#1a1d20] rounded-sm">
        <PanelHeader icon={<GitBranch size={12} />} title="ACTIONS" sub={`${actions.length} 已知`} />
        <ColHeader cols={['ACTION', 'RUNS', 'ACTIVE', 'ERR', 'LAST']} widths="grid-cols-[1fr_45px_55px_40px_70px]" />
        {actions.map((a) => (
          <div
            key={a.name}
            className="grid grid-cols-[1fr_45px_55px_40px_70px] items-center px-3 h-8 border-b border-[#1f2428] hover:bg-[#1f2428] transition-colors cursor-default"
          >
            <span className="font-mono text-[11px] text-[#c8ced6] truncate">{a.name}</span>
            <span className="font-mono text-[11px] text-[#5a6370] tabular-nums text-right">{fmtNum(a.runs)}</span>
            <span className={`font-mono text-[11px] tabular-nums text-right ${a.active > 0 ? 'text-[#22c55e]' : 'text-[#5a6370]'}`}>
              {a.active > 0 ? `${a.active} RUN` : '—'}
            </span>
            <span className={`font-mono text-[11px] tabular-nums text-right ${a.errors > 0 ? 'text-[#ef4444]' : 'text-[#5a6370]'}`}>
              {a.errors > 0 ? a.errors : '—'}
            </span>
            <span className="font-mono text-[11px] text-[#5a6370] tabular-nums text-right">{fmtAge(a.lastRunAt)}</span>
          </div>
        ))}
      </section>
    </div>
  )
}

function ColHeader({ cols, widths }: { cols: string[]; widths: string }) {
  return (
    <div className={`grid ${widths} items-center px-3 h-7 border-b border-[#2a2f35] bg-[#141618]`}>
      {cols.map((c, i) => (
        <span key={c} className={`font-mono text-[10px] text-[#5a6370] uppercase tracking-wider ${i > 0 ? 'text-right' : ''}`}>
          {c}
        </span>
      ))}
    </div>
  )
}
