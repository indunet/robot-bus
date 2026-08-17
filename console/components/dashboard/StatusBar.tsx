'use client'

import { useState, useEffect } from 'react'
import { type BrokerInfo, fmtBytes, fmtNum } from '@/lib/mock-data'
import { Activity, Server, Zap, AlertTriangle, Clock } from 'lucide-react'
import { fmtUptimeLocalized, useI18n, type Locale } from '@/lib/i18n'

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
    broker.status === 'CONNECTING' ? t('connecting') : broker.status

  return (
    <header className="h-11 bg-[#0e1012] border-b border-bus-border flex items-center px-4 gap-6 shrink-0 overflow-x-auto">
      <div className="flex items-center gap-2.5 shrink-0">
        <Zap size={15} className="text-bus-cyan" strokeWidth={2.25} />
        <span className="font-brand text-[15px] font-semibold tracking-[0.2em] text-bus-cyan uppercase leading-none">
          ROBOT-BUS
        </span>
        <span className="font-mono text-xs text-bus-muted ml-0.5">
          {connecting ? 'v…' : `v${broker.version}`}
        </span>
      </div>

      <div className="w-px h-5 bg-bus-border shrink-0" />

      <div className="flex items-center gap-1.5 shrink-0">
        <span className={`w-2.5 h-2.5 rounded-full ${statusPulse}`} />
        <span className={`font-mono text-sm font-semibold ${statusColor}`}>
          {statusLabel}
          {connecting && (
            <span className="connecting-dots inline-block w-4 text-left" data-phase={dotPhase} />
          )}
        </span>
      </div>

      <div className="w-px h-5 bg-bus-border shrink-0" />

      <div className="flex items-center gap-5 shrink-0">
        <Metric icon={<Activity size={13} className="text-bus-muted" />} label={t('metricMsgS')} value={fmtNum(broker.msgPerSec)} accent />
        <Metric icon={<Server size={13} className="text-bus-muted" />} label={t('metricBw')} value={`${fmtBytes(broker.bytesPerSec)}/s`} />
        <Metric label={t('metricSvcS')} value={fmtNum(broker.svcCallsPerSec)} />
        <Metric label={t('metricActS')} value={fmtNum(broker.actRunsPerSec)} />
        <Metric label={t('metricTotal')} value={fmtNum(broker.totalMessages)} />
        <Metric label={t('metricUptime')} value={fmtUptimeLocalized(broker.uptime, t)} />
        <Metric label={t('metricPid')} value={String(broker.pid)} />
      </div>

      {broker.totalErrors > 0 && (
        <>
          <div className="w-px h-5 bg-bus-border shrink-0" />
          <div className="flex items-center gap-1.5 shrink-0">
            <AlertTriangle size={13} className="text-bus-red" />
            <span className="font-mono text-sm text-bus-red">{broker.totalErrors} {t('metricErr')}</span>
          </div>
        </>
      )}

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
  icon, label, value, accent,
}: {
  icon?: React.ReactNode
  label: string
  value: string
  accent?: boolean
}) {
  return (
    <div className="flex items-center gap-1.5">
      {icon}
      <span className="font-mono text-xs text-bus-muted uppercase">{label}</span>
      <span className={`font-mono text-sm font-semibold tabular-nums ${accent ? 'text-bus-cyan' : 'text-bus-text'}`}>
        {value}
      </span>
    </div>
  )
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
