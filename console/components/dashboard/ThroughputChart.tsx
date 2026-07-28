'use client'

import { AreaChart, Area, XAxis, YAxis, ResponsiveContainer, Tooltip } from 'recharts'
import { PanelHeader } from './BrokerOverview'
import { BarChart2 } from 'lucide-react'
import { useI18n } from '@/lib/i18n'

interface DataPoint {
  t: string
  msgs: number
  bytes: number
}

interface Props {
  data: DataPoint[]
}

export default function ThroughputChart({ data }: Props) {
  const { t } = useI18n()

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm">
      <PanelHeader icon={<BarChart2 size={14} />} title={t('throughputTitle')} sub={t('throughputSub')} />
      <div className="h-28 px-2 pb-2 pt-1">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data} margin={{ top: 2, right: 4, left: -30, bottom: 0 }}>
            <defs>
              <linearGradient id="gradCyan" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#00d4ff" stopOpacity={0.25} />
                <stop offset="100%" stopColor="#00d4ff" stopOpacity={0.02} />
              </linearGradient>
            </defs>
            <XAxis
              dataKey="t"
              tick={{ fill: '#5a6370', fontSize: 11, fontFamily: 'monospace' }}
              tickLine={false}
              axisLine={false}
              interval="preserveStartEnd"
            />
            <YAxis
              tick={{ fill: '#5a6370', fontSize: 11, fontFamily: 'monospace' }}
              tickLine={false}
              axisLine={false}
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
              formatter={(v: number) => [`${v} msg/s`, t('throughputSeries')]}
            />
            <Area
              type="monotone"
              dataKey="msgs"
              stroke="#00d4ff"
              strokeWidth={1.5}
              fill="url(#gradCyan)"
              dot={false}
              isAnimationActive={false}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </section>
  )
}

/** Seed empty throughput history (labels use en-US; live poll overwrites). */
export function generateThroughputHistory(base: number, len = 60): DataPoint[] {
  let v = base
  return Array.from({ length: len }, (_, i) => {
    v = Math.max(0, v + (Math.random() - 0.5) * 200)
    const t = new Date(Date.now() - (len - i) * 1000)
    return {
      t: t.toLocaleTimeString('en-US', { minute: '2-digit', second: '2-digit', hour12: false }),
      msgs: Math.round(v),
      bytes: Math.round(v * 250),
    }
  })
}
