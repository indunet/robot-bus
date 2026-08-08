'use client'

import { useState, useRef, useEffect, useLayoutEffect } from 'react'
import { type LogEntry, type LogLevel } from '@/lib/mock-data'
import { Filter, Pause, Play, Trash2, Terminal } from 'lucide-react'
import { useI18n } from '@/lib/i18n'

interface Props {
  logs: LogEntry[]
}

const LEVEL_COLORS: Record<LogLevel, string> = {
  DEBUG: 'text-bus-muted',
  INFO:  'text-bus-cyan',
  WARN:  'text-bus-amber',
  ERROR: 'text-bus-red',
}
const LEVEL_BG: Record<LogLevel, string> = {
  DEBUG: '',
  INFO:  '',
  WARN:  'bg-bus-amber/5',
  ERROR: 'bg-bus-red/8',
}

/** Approx LogLine height (py-1 + 13px mono + border). */
const ROW_HEIGHT_PX = 28

export default function EventStream({ logs }: Props) {
  const { t, dateLocale } = useI18n()
  const [filter, setFilter] = useState<LogLevel | 'ALL'>('ALL')
  const [paused, setPaused] = useState(false)
  const [entries, setEntries] = useState<LogEntry[]>(logs)
  const [padRows, setPadRows] = useState(0)
  const bottomRef = useRef<HTMLDivElement>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!paused) setEntries(logs)
  }, [logs, paused])

  useEffect(() => {
    if (!paused && scrollContainerRef.current) {
      scrollContainerRef.current.scrollTop = scrollContainerRef.current.scrollHeight
    }
  }, [entries, paused])

  const visible = filter === 'ALL' ? entries : entries.filter((e) => e.level === filter)

  useLayoutEffect(() => {
    const el = scrollContainerRef.current
    if (!el) return

    const updatePad = () => {
      const capacity = Math.max(0, Math.floor(el.clientHeight / ROW_HEIGHT_PX))
      setPadRows(Math.max(0, capacity - visible.length))
    }

    updatePad()
    const ro = new ResizeObserver(updatePad)
    ro.observe(el)
    return () => ro.disconnect()
  }, [visible.length])

  const filterLabels = {
    ALL: t('filterAll'),
    DEBUG: 'DEBUG',
    INFO: 'INFO',
    WARN: 'WARN',
    ERROR: 'ERROR',
  } as const

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm flex flex-col h-full min-h-0">
      <div className="flex items-center justify-between px-3 h-9 border-b border-bus-border shrink-0">
        <div className="flex items-center gap-2">
          <Terminal size={14} className="text-bus-cyan" />
          <span className="font-mono text-xs font-bold text-bus-text tracking-widest uppercase">{t('eventsTitle')}</span>
          <span className="font-mono text-xs text-bus-muted">{t('eventsSub')}</span>
        </div>
        <div className="flex items-center gap-2">
          <Filter size={13} className="text-bus-muted" />
          {(['ALL', 'DEBUG', 'INFO', 'WARN', 'ERROR'] as const).map((lvl) => (
            <button
              key={lvl}
              onClick={() => setFilter(lvl)}
              className={`font-mono text-xs px-2 py-0.5 rounded border transition-colors ${
                filter === lvl
                  ? 'border-bus-cyan text-bus-cyan bg-bus-cyan/10'
                  : 'border-bus-border text-bus-muted hover:text-bus-text hover:border-bus-border2'
              }`}
            >
              {filterLabels[lvl]}
            </button>
          ))}
          <div className="w-px h-4 bg-bus-border" />
          <button
            onClick={() => setPaused((p) => !p)}
            className={`flex items-center gap-1 font-mono text-xs px-2 py-0.5 rounded border transition-colors ${
              paused
                ? 'border-bus-amber text-bus-amber bg-bus-amber/10'
                : 'border-bus-border text-bus-muted hover:text-bus-text'
            }`}
          >
            {paused ? <Play size={11} /> : <Pause size={11} />}
            {paused ? t('resume') : t('pause')}
          </button>
          <button
            onClick={() => setEntries([])}
            className="flex items-center gap-1 font-mono text-xs px-2 py-0.5 rounded border border-bus-border text-bus-muted hover:text-bus-red hover:border-bus-red transition-colors"
          >
            <Trash2 size={11} />
            {t('clear')}
          </button>
        </div>
      </div>

      <div ref={scrollContainerRef} className="relative flex-1 overflow-y-auto min-h-0 font-mono text-[13px]">
        {visible.map((entry) => (
          <LogLine key={entry.id} entry={entry} dateLocale={dateLocale} />
        ))}
        {Array.from({ length: padRows }, (_, i) => (
          <EmptyRow key={`pad-${i}`} />
        ))}
        {visible.length === 0 && (
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
            <span className="font-mono text-xs text-bus-muted">{t('eventsEmpty')}</span>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      <div className="h-7 border-t border-bus-border flex items-center px-3 gap-3 shrink-0 bg-bus-bg">
        <span className="font-mono text-xs text-bus-muted">{t('eventsCount', { n: visible.length })}</span>
        {paused && (
          <span className="font-mono text-xs text-bus-amber animate-pulse">● {t('eventsPaused')}</span>
        )}
        {!paused && (
          <span className="font-mono text-xs text-bus-green">● {t('eventsLive')}</span>
        )}
      </div>
    </section>
  )
}

function EmptyRow() {
  return (
    <div
      aria-hidden
      className="flex items-start gap-2.5 px-3 py-1 border-b border-bus-panel/70 min-h-[28px]"
    >
      <span className="tabular-nums shrink-0 w-[96px]" />
      <span className="shrink-0 w-11" />
      <span className="shrink-0 w-[110px]" />
      <span className="leading-relaxed" />
    </div>
  )
}

function LogLine({ entry, dateLocale }: { entry: LogEntry; dateLocale: string }) {
  const ts = new Date(entry.ts)
  const timeStr = ts.toLocaleTimeString(dateLocale, {
    hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
  }) + '.' + String(ts.getMilliseconds()).padStart(3, '0')

  return (
    <div className={`flex items-start gap-2.5 px-3 py-1 border-b border-bus-panel hover:bg-[#1f2428] min-h-[28px] ${LEVEL_BG[entry.level]}`}>
      <span className="text-bus-muted tabular-nums shrink-0 w-[96px]">{timeStr}</span>
      <span className={`${LEVEL_COLORS[entry.level]} font-bold shrink-0 w-11`}>{entry.level}</span>
      <span className="text-bus-cyan-dim shrink-0 w-[110px] truncate">{entry.source}</span>
      <span className="text-bus-text leading-relaxed">{entry.message}</span>
    </div>
  )
}
