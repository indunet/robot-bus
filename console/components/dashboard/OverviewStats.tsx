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
import { Activity, Radio, Cpu, GitBranch, HardDrive, Layers, Hash, Clock } from 'lucide-react'
import { useI18n, type I18nValue } from '@/lib/i18n'
import type { ReactNode } from 'react'

interface Props {
  broker: BrokerInfo
  topics: TopicInfo[]
  services: ServiceInfo[]
  actions: ActionInfo[]
}

export default function OverviewStats({ broker, topics, services, actions }: Props) {
  const { t } = useI18n()

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm h-full flex flex-col min-h-0">
      <PanelHeader
        icon={<Layers size={14} />}
        title={t('statsTitle')}
        sub={t('statsSub')}
      />
      <div className="flex-1 p-3 grid grid-cols-2 sm:grid-cols-3 gap-2 min-h-0 [grid-auto-rows:minmax(0,1fr)]">
        <StatTile
          icon={<Activity size={12} />}
          label={t('statMsgS')}
          value={fmtNum(broker.msgPerSec)}
          accent="cyan"
        />
        <StatTile
          icon={<HardDrive size={12} />}
          label={t('statBw')}
          value={`${fmtBytes(broker.bytesPerSec)}/s`}
          accent="muted"
        />
        <StatTile
          icon={<Radio size={12} />}
          label={t('statTopics')}
          value={String(topics.length)}
          accent="cyan"
        />
        <StatTile
          icon={<Cpu size={12} />}
          label={t('statServices')}
          value={String(services.length)}
          accent="amber"
        />
        <StatTile
          icon={<Cpu size={12} />}
          label={t('statSvcS')}
          value={fmtNum(broker.svcCallsPerSec)}
          accent="amber"
        />
        <StatTile
          icon={<GitBranch size={12} />}
          label={t('statActions')}
          value={String(actions.length)}
          accent="green"
        />
        <StatTile
          icon={<GitBranch size={12} />}
          label={t('statActS')}
          value={fmtNum(broker.actRunsPerSec)}
          accent="green"
        />
        <StatTile
          icon={<Hash size={12} />}
          label={t('statTotalMsgs')}
          value={fmtNum(broker.totalMessages)}
          accent="muted"
        />
        <StatTile
          icon={<Clock size={12} />}
          label={t('statUptime')}
          value={formatShortUptime(broker.uptime, t)}
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
  accent,
}: {
  icon: ReactNode
  label: string
  value: string
  accent: Accent
}) {
  const { labelCase } = useI18n()
  const valueColor =
    accent === 'cyan'
      ? 'text-bus-cyan'
      : accent === 'amber'
        ? 'text-bus-amber'
        : accent === 'green'
          ? 'text-bus-green'
          : 'text-bus-text'

  return (
    <div className="border border-bus-border bg-bus-bg px-2.5 py-2 min-h-0 h-full flex flex-col justify-between gap-1">
      <div className="flex items-center gap-1.5 min-w-0">
        <span className="text-bus-muted shrink-0">{icon}</span>
        <span className={`font-mono text-[11px] leading-none text-bus-muted tracking-wider truncate ${labelCase}`}>
          {label}
        </span>
      </div>
      <span
        className={`mt-auto w-full min-w-0 truncate text-right font-mono text-[15px] font-medium tabular-nums leading-5 ${valueColor}`}
      >
        {value}
      </span>
    </div>
  )
}

function formatShortUptime(s: number, t: I18nValue['t']): string {
  const d = Math.floor(s / 86400)
  const h = Math.floor((s % 86400) / 3600)
  const m = Math.floor((s % 3600) / 60)
  if (d > 0) return t('uptimeShortDh', { d, h })
  if (h > 0) return t('uptimeShortHm', { h, m })
  return t('uptimeShortMs', { m, s: s % 60 })
}
