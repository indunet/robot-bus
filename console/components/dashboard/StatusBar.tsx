'use client'

import { useState, useEffect } from 'react'
import { type BrokerInfo, fmtBytes, fmtNum, fmtRate } from '@/lib/mock-data'
import { Activity, Server, Zap, AlertTriangle, Clock } from 'lucide-react'
import { useI18n, type Locale } from '@/lib/i18n'

interface StatusBarProps {
  broker: BrokerInfo
}

export default function StatusBar({ broker }: StatusBarProps) {
  const { t, locale, setLocale, dateLocale } = useI18n()
  const connecting = broker.status === 'CONNECTING'
  const [dotPhase, setDotPhase] = useState(1)

  useEffect(() => {
    if (!connecting) return
    const id = setInterval(() => {
      setDotPhase((phase) => (phase % 3) + 1)
    }, 400)
    return () => clearInterval(id)
  }, [connecting])

  const statusColor =
    broker.status === 'ONLINE' ? 'text-bus-green' :
    broker.status === 'DEGRADED' ? 'text-bus-amber' :
    broker.status === 'CONNECTING' ? 'text-bus-cyan' :
    'text-bus-red'

  const statusPulse =
    broker.status === 'ONLINE' ? 'pulse-green bg-bus-green' :
    broker.status === 'DEGRADED' ? 'pulse-amber bg-bus-amber' :
    broker.status === 'CONNECTING' ? 'pulse-cyan bg-bus-cyan' :
    'pulse-red bg-bus-red'

  const statusLabel =
    broker.status === 'CONNECTING' ? t('connecting')
    : broker.status === 'ONLINE' ? t('statusOnline')
    : broker.status === 'DEGRADED' ? t('statusDegraded')
    : t('statusOffline')

  return (
    <header className="h-11 bg-[#0e1012] border-b border-bus-border flex items-center px-4 gap-6 shrink-0 overflow-x-auto">
      <div className="flex items-center gap-2.5 shrink-0">
        <Zap size={15} className="text-bus-cyan" strokeWidth={2.25} />
        <span className="font-brand text-[15px] font-semibold tracking-[0.2em] text-bus-cyan uppercase leading-none">
          ROBOT-BUS
        </span>
        <span className="font-mono text-xs text-bus-muted ml-0.5 inline-block min-w-[4.5rem] tabular-nums">
          {connecting ? 'v…' : `v${broker.version}`}
        </span>
      </div>

      <div className="w-px h-5 bg-bus-border shrink-0" />

      <div className="flex items-center gap-1.5 shrink-0">
        <span className={`w-2.5 h-2.5 rounded-full ${statusPulse}`} />
        <span className={`font-mono text-xs font-semibold min-w-[7.5rem] ${statusColor}`}>
          {statusLabel}
          {connecting && (
            <span className="connecting-dots inline-block w-4 text-left" data-phase={dotPhase} />
          )}
        </span>
      </div>

      <div className="w-px h-5 bg-bus-border shrink-0" />

      <div className="flex items-center gap-5 shrink-0">
        <Metric icon={<Activity size={13} className="text-bus-muted" />} label={t('metricMsgS')} value={fmtRate(broker.msgPerSec)} valueWidth="w-[6ch]" accent />
        <Metric icon={<Server size={13} className="text-bus-muted" />} label={t('metricBw')} value={`${fmtBytes(broker.bytesPerSec)}/s`} valueWidth="w-[10ch]" />
        <Metric label={t('metricSvcS')} value={fmtRate(broker.svcCallsPerSec)} valueWidth="w-[6ch]" />
        <Metric label={t('metricActS')} value={fmtRate(broker.actRunsPerSec)} valueWidth="w-[6ch]" />
        <Metric label={t('metricTotal')} value={fmtNum(broker.totalMessages)} valueWidth="w-[6ch]" />
        <Metric label={t('metricUptime')} value={fmtUptimeStable(broker.uptime)} valueWidth="w-[12ch]" />
        <Metric label={t('metricPid')} value={connecting ? '—' : String(broker.pid)} valueWidth="w-[7ch]" />
      </div>

      <div className="w-px h-5 bg-bus-border shrink-0" />
      <div
        className={`flex items-center gap-1.5 shrink-0 min-w-[4.75rem] ${
          broker.totalErrors > 0 ? '' : 'invisible'
        }`}
      >
        <AlertTriangle size={13} className="text-bus-red" />
        <span className="font-mono text-sm text-bus-red tabular-nums">
          {broker.totalErrors} {t('metricErr')}
        </span>
      </div>

      <div className="flex-1" />

      <LiveClock dateLocale={dateLocale} />
      <LangSwitch locale={locale} setLocale={setLocale} label={t('langSwitch')} />
    </header>
  )
}

function LangSwitch({
  locale,
  setLocale,
  label,
}: {
  locale: Locale
  setLocale: (locale: Locale) => void
  label: string
}) {
  return (
    <div
      className="flex items-center gap-0.5 shrink-0 border border-bus-border rounded overflow-hidden"
      title={label}
      role="group"
      aria-label={label}
    >
      <button
        type="button"
        onClick={() => setLocale('en')}
        className={`font-mono text-xs px-2 py-0.5 transition-colors ${
          locale === 'en'
            ? 'bg-bus-cyan/15 text-bus-cyan'
            : 'text-bus-muted hover:text-bus-text'
        }`}
      >
        EN
      </button>
      <button
        type="button"
        onClick={() => setLocale('zh')}
        className={`font-mono text-xs px-2 py-0.5 transition-colors ${
          locale === 'zh'
            ? 'bg-bus-cyan/15 text-bus-cyan'
            : 'text-bus-muted hover:text-bus-text'
        }`}
      >
        中文
      </button>
    </div>
  )
}

function Metric({
  icon, label, value, accent, valueWidth,
}: {
  icon?: React.ReactNode
  label: string
  value: string
  accent?: boolean
  /** Fixed slot so live counters do not shove neighbors. */
  valueWidth: string
}) {
  const { labelCase } = useI18n()
  return (
    <div className="flex items-center gap-1.5">
      {icon}
      <span className={`font-mono text-xs text-bus-muted shrink-0 ${labelCase}`}>{label}</span>
      <span
        className={`font-mono text-sm font-semibold tabular-nums text-right ${valueWidth} ${
          accent ? 'text-bus-cyan' : 'text-bus-text'
        }`}
      >
        {value}
      </span>
    </div>
  )
}

/** Fixed-width uptime (`00d 00:05:03`) so the header does not reflow every second. */
function fmtUptimeStable(s: number): string {
  const total = Math.max(0, Math.floor(s))
  const d = Math.floor(total / 86400)
  const h = Math.floor((total % 86400) / 3600)
  const m = Math.floor((total % 3600) / 60)
  const ss = total % 60
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(Math.min(d, 99))}d ${pad(h)}:${pad(m)}:${pad(ss)}`
}

function LiveClock({ dateLocale }: { dateLocale: string }) {
  // Empty until mount — avoids SSR/client clock skew hydration mismatch.
  const [time, setTime] = useState('')
  useEffect(() => {
    const tick = () => setTime(new Date().toLocaleTimeString(dateLocale, { hour12: false }))
    tick()
    const id = setInterval(tick, 1000)
    return () => clearInterval(id)
  }, [dateLocale])
  return (
    <span className="flex items-center gap-1.5 shrink-0">
      <Clock size={13} className="text-bus-muted" />
      <span className="font-mono text-xs text-bus-muted tabular-nums">{time || '--:--:--'}</span>
    </span>
  )
}
