'use client'

import { AreaChart, Area, XAxis, YAxis, ResponsiveContainer, Tooltip } from 'recharts'
import { PanelHeader } from './BrokerOverview'
import { BarChart2, Cpu, GitBranch } from 'lucide-react'
import { useI18n } from '@/lib/i18n'
import { formatHms } from '@/lib/utils'
import type { ReactNode } from 'react'

export interface RatePoint {
  t: string
  value: number
}

interface RateChartProps {
  title: string
  sub: string
  seriesLabel: string
  unit: string
  stroke: string
  gradientId: string
  icon: ReactNode
  data: RatePoint[]
}

function RateChart({ title, sub, seriesLabel, unit, stroke, gradientId, icon, data }: RateChartProps) {
  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm h-full flex flex-col min-h-0">
      <PanelHeader icon={icon} title={title} sub={sub} />
      <div className="flex-1 min-h-[7rem] px-2 pb-1 pt-1">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data} margin={{ top: 4, right: 8, left: -16, bottom: 2 }}>
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
              angle={-35}
              textAnchor="end"
              height={36}
              tickMargin={4}
            />
            <YAxis
              tick={{ fill: '#5a6370', fontSize: 11, fontFamily: 'monospace' }}
              tickLine={false}
              axisLine={false}
              width={40}
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
              formatter={(v) => [`${Number(v ?? 0)} ${unit}`, seriesLabel]}
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
export default function ThroughputChart({ data }: { data: RatePoint[] }) {
  const { t } = useI18n()
  return (
    <RateChart
      title={t('throughputTitle')}
      sub={t('throughputSub')}
      seriesLabel={t('throughputSeries')}
      unit="msg/s"
      stroke="#00d4ff"
      gradientId="gradCyan"
      icon={<BarChart2 size={14} />}
      data={data}
    />
  )
}

/** Aggregate service call rate (calls/s). */
export function ServiceRateChart({ data }: { data: RatePoint[] }) {
  const { t } = useI18n()
  return (
    <RateChart
      title={t('svcRateTitle')}
      sub={t('svcRateSub')}
      seriesLabel={t('svcRateSeries')}
      unit="calls/s"
      stroke="#f5a623"
      gradientId="gradAmber"
      icon={<Cpu size={14} />}
      data={data}
    />
  )
}

/** Aggregate action run rate (runs/s). */
export function ActionRateChart({ data }: { data: RatePoint[] }) {
  const { t } = useI18n()
  return (
    <RateChart
      title={t('actRateTitle')}
      sub={t('actRateSub')}
      seriesLabel={t('actRateSeries')}
      unit="runs/s"
      stroke="#3dd68c"
      gradientId="gradGreen"
      icon={<GitBranch size={14} />}
      data={data}
    />
  )
}

/** Seed empty rate history (HH:MM:SS labels; live poll overwrites). */
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

/** @deprecated Prefer generateRateHistory */
export function generateThroughputHistory(base: number, len = 60): RatePoint[] {
  return generateRateHistory(base, len)
}
