'use client'

import { useMemo, useState, type ReactNode } from 'react'
import { AreaChart, Area, XAxis, YAxis, ResponsiveContainer, Tooltip } from 'recharts'
import { PanelHeader } from './BrokerOverview'
import { BarChart2, Cpu, GitBranch } from 'lucide-react'
import { useI18n } from '@/lib/i18n'
import { formatHms, formatMsTick } from '@/lib/utils'

/** Max ring-buffer length — 10 min @ ~1 Hz console status. */
export const RATE_HISTORY_MAX = 600

export const RATE_WINDOWS = [1, 3, 5, 10] as const
export type RateWindowMinutes = (typeof RATE_WINDOWS)[number]

export interface RatePoint {
  t: string
  value: number
}

interface RateChartProps {
  title: string
  /** i18n key factory: receives selected window minutes. */
  subForWindow: (minutes: RateWindowMinutes) => string
  seriesLabel: string
  unit: string
  stroke: string
  gradientId: string
  icon: ReactNode
  data: RatePoint[]
  /** Show 1m/3m/5m/10m picker (topics / services / actions charts). */
  showWindowPicker?: boolean
}

function WindowPicker({
  value,
  onChange,
  accent,
}: {
  value: RateWindowMinutes
  onChange: (next: RateWindowMinutes) => void
  accent: string
}) {
  return (
    <div
      role="radiogroup"
      aria-label="chart window"
      className="flex items-center gap-0.5 rounded-md border border-[#3a8fa3]/40 bg-[#0c1014]/90 p-0.5 shadow-[inset_0_1px_0_rgb(255_255_255_/06%)]"
    >
      {RATE_WINDOWS.map((mins) => {
        const active = value === mins
        return (
          <button
            key={mins}
            type="button"
            role="radio"
            aria-checked={active}
            onClick={() => onChange(mins)}
            className={`relative h-6 min-w-[1.85rem] rounded-[4px] px-1.5 font-mono text-[10px] tabular-nums tracking-wide transition-all ${
              active
                ? 'text-[#0a0d0f] font-semibold'
                : 'text-bus-muted hover:text-bus-text hover:bg-white/[0.05]'
            }`}
            style={
              active
                ? {
                    backgroundColor: accent,
                    boxShadow: `0 0 0 1px ${accent}55, 0 0 10px ${accent}33`,
                  }
                : undefined
            }
          >
            {mins}m
          </button>
        )
      })}
    </div>
  )
}

function RateChart({
  title,
  subForWindow,
  seriesLabel,
  unit,
  stroke,
  gradientId,
  icon,
  data,
  showWindowPicker = true,
}: RateChartProps) {
  const [windowMin, setWindowMin] = useState<RateWindowMinutes>(1)
  const view = useMemo(() => {
    const n = windowMin * 60
    return data.length <= n ? data : data.slice(data.length - n)
  }, [data, windowMin])

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm h-full flex flex-col min-h-0 min-w-0 w-full">
      <PanelHeader
        icon={icon}
        title={title}
        sub={subForWindow(windowMin)}
        trailing={
          showWindowPicker ? (
            <WindowPicker value={windowMin} onChange={setWindowMin} accent={stroke} />
          ) : undefined
        }
      />
      <div className="flex-1 min-h-[7rem] px-3 pb-2 pt-2">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={view} margin={{ top: 10, right: 8, left: 4, bottom: 4 }}>
            <defs>
              <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor={stroke} stopOpacity={0.25} />
                <stop offset="100%" stopColor={stroke} stopOpacity={0.02} />
              </linearGradient>
            </defs>
            <XAxis
              dataKey="t"
              tick={{ fill: '#5a6370', fontSize: 10, fontFamily: 'monospace' }}
              tickLine={false}
              axisLine={false}
              interval="preserveStartEnd"
              minTickGap={28}
              height={36}
              tickMargin={8}
              angle={-28}
              textAnchor="end"
              tickFormatter={formatMsTick}
            />
            <YAxis
              tick={{ fill: '#5a6370', fontSize: 11, fontFamily: 'monospace' }}
              tickLine={false}
              axisLine={false}
              width={44}
              allowDecimals={false}
              tickFormatter={(v) => String(Math.round(Number(v)))}
            />
            <Tooltip
              contentStyle={{
                background: '#1a1d20',
                border: '1px solid #2a2f35',
                borderRadius: 2,
                fontSize: 12,
                fontFamily: 'monospace',
                color: '#c8ced6',
              }}
              labelStyle={{ color: '#5a6370' }}
              formatter={(v) => [`${Math.round(Number(v ?? 0))} ${unit}`, seriesLabel]}
            />
            <Area
              type="monotone"
              dataKey="value"
              stroke={stroke}
              strokeWidth={1.5}
              fill={`url(#${gradientId})`}
              dot={false}
              isAnimationActive={false}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </section>
  )
}

/** Topic message throughput (msg/s). */
export default function ThroughputChart({
  data,
  showWindowPicker = true,
}: {
  data: RatePoint[]
  showWindowPicker?: boolean
}) {
  const { t } = useI18n()
  return (
    <RateChart
      title={t('throughputTitle')}
      subForWindow={(m) => t('throughputSub', { n: m })}
      seriesLabel={t('throughputSeries')}
      unit="msg/s"
      stroke="#00d4ff"
      gradientId="gradCyan"
      icon={<BarChart2 size={14} />}
      data={data}
      showWindowPicker={showWindowPicker}
    />
  )
}

/** Aggregate service call rate (calls/s). */
export function ServiceRateChart({
  data,
  showWindowPicker = true,
}: {
  data: RatePoint[]
  showWindowPicker?: boolean
}) {
  const { t } = useI18n()
  return (
    <RateChart
      title={t('svcRateTitle')}
      subForWindow={(m) => t('svcRateSub', { n: m })}
      seriesLabel={t('svcRateSeries')}
      unit="calls/s"
      stroke="#f5a623"
      gradientId="gradAmber"
      icon={<Cpu size={14} />}
      data={data}
      showWindowPicker={showWindowPicker}
    />
  )
}

/** Aggregate action run rate (runs/s). */
export function ActionRateChart({
  data,
  showWindowPicker = true,
}: {
  data: RatePoint[]
  showWindowPicker?: boolean
}) {
  const { t } = useI18n()
  return (
    <RateChart
      title={t('actRateTitle')}
      subForWindow={(m) => t('actRateSub', { n: m })}
      seriesLabel={t('actRateSeries')}
      unit="runs/s"
      stroke="#3dd68c"
      gradientId="gradGreen"
      icon={<GitBranch size={14} />}
      data={data}
      showWindowPicker={showWindowPicker}
    />
  )
}

/** Seed empty rate history (HH:MM:SS labels for tooltips; axis shows MM:SS). */
export function generateRateHistory(base = 0, len = 60): RatePoint[] {
  let v = base
  return Array.from({ length: len }, (_, i) => {
    v = Math.max(0, v + (Math.random() - 0.5) * (base > 0 ? base * 0.2 : 0))
    const t = new Date(Date.now() - (len - i) * 1000)
    return {
      t: formatHms(t),
      value: Math.round(v),
    }
  })
}

/** Append one sample; keep at most `maxLen` points (default 5 min @ 1 Hz). */
export function appendRatePoint(
  prev: RatePoint[],
  label: string,
  value: number,
  maxLen = RATE_HISTORY_MAX,
): RatePoint[] {
  const next = [...prev, { t: label, value }]
  return next.length > maxLen ? next.slice(next.length - maxLen) : next
}

/** @deprecated Prefer generateRateHistory */
export function generateThroughputHistory(base: number, len = 60): RatePoint[] {
  return generateRateHistory(base, len)
}
