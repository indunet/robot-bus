'use client'

import {
  type BrokerInfo,
  type TopicInfo,
  type ServiceInfo,
  type ActionInfo,
  fmtBytes,
  fmtNum,
} from '@/lib/mock-data'
import { PanelHeader } from './BrokerOverview'
import { Activity, Radio, Cpu, GitBranch, HardDrive, Layers } from 'lucide-react'
import { useI18n } from '@/lib/i18n'
import type { ReactNode } from 'react'

interface Props {
  broker: BrokerInfo
  topics: TopicInfo[]
  services: ServiceInfo[]
  actions: ActionInfo[]
}

export default function OverviewStats({ broker, topics, services, actions }: Props) {
  const { t } = useI18n()
  const activeTopics = topics.filter((x) => x.msgPerSec > 0).length
  const activeActions = actions.reduce((n, a) => n + a.active, 0)
  const svcWorkers = services.reduce((n, s) => n + s.workers, 0)

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm h-full flex flex-col min-h-0">
      <PanelHeader
        icon={<Layers size={14} />}
        title={t('statsTitle')}
        sub={t('statsSub')}
      />
      <div className="flex-1 p-3 grid grid-cols-2 sm:grid-cols-3 gap-2 content-start">
        <StatTile
          icon={<Activity size={13} />}
          label={t('statMsgS')}
          value={fmtNum(broker.msgPerSec)}
          accent="cyan"
        />
        <StatTile
          icon={<HardDrive size={13} />}
          label={t('statBw')}
          value={`${fmtBytes(broker.bytesPerSec)}/s`}
          accent="muted"
        />
        <StatTile
          icon={<Radio size={13} />}
          label={t('statTopics')}
          value={String(topics.length)}
          hint={t('statTopicsActive', { n: activeTopics })}
          accent="cyan"
        />
        <StatTile
          icon={<Cpu size={13} />}
          label={t('statServices')}
          value={String(services.length)}
          hint={t('statWorkers', { n: svcWorkers })}
          accent="amber"
        />
        <StatTile
          icon={<Cpu size={13} />}
          label={t('statSvcS')}
          value={fmtNum(broker.svcCallsPerSec)}
          accent="amber"
        />
        <StatTile
          icon={<GitBranch size={13} />}
          label={t('statActions')}
          value={String(actions.length)}
          hint={t('statActiveGoals', { n: activeActions })}
          accent="green"
        />
        <StatTile
          icon={<GitBranch size={13} />}
          label={t('statActS')}
          value={fmtNum(broker.actRunsPerSec)}
          accent="green"
        />
        <StatTile
          label={t('statTotalMsgs')}
          value={fmtNum(broker.totalMessages)}
          accent="muted"
        />
        <StatTile
          label={t('statUptime')}
          value={formatShortUptime(broker.uptime)}
          accent="muted"
        />
      </div>
    </section>
  )
}

type Accent = 'cyan' | 'amber' | 'green' | 'muted'

function StatTile({
  icon,
  label,
  value,
  hint,
  accent,
}: {
  icon?: ReactNode
  label: string
  value: string
  hint?: string
  accent: Accent
}) {
  const valueColor =
    accent === 'cyan'
      ? 'text-bus-cyan'
      : accent === 'amber'
        ? 'text-bus-amber'
        : accent === 'green'
          ? 'text-bus-green'
          : 'text-bus-text'

  return (
    <div className="border border-bus-border bg-bus-bg px-2.5 py-2 min-h-[3.5rem] flex flex-col justify-center gap-0.5">
      <div className="flex items-center gap-1.5">
        {icon ? <span className="text-bus-muted shrink-0">{icon}</span> : null}
        <span className="font-mono text-[10px] text-bus-muted tracking-wider uppercase truncate">
          {label}
        </span>
      </div>
      <span className={`font-mono text-lg font-semibold tabular-nums leading-none text-right ${valueColor}`}>
        {value}
      </span>
      {hint ? (
        <span className="font-mono text-[10px] text-bus-muted truncate text-right">{hint}</span>
      ) : null}
    </div>
  )
}

function formatShortUptime(s: number): string {
  const d = Math.floor(s / 86400)
  const h = Math.floor((s % 86400) / 3600)
  const m = Math.floor((s % 3600) / 60)
  if (d > 0) return `${d}d ${h}h`
  if (h > 0) return `${h}h ${m}m`
  return `${m}m ${s % 60}s`
}
