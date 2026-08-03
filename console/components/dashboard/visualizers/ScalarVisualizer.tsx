'use client'

import { useEffect, useState } from 'react'
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer, Tooltip } from 'recharts'
import { asRecord, num, pushPoint, type SeriesPoint } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

/** Extract a primary numeric reading from common scalar sensor msgs. */
function extractScalar(message: unknown): { label: string; value: number } | null {
  const rec = asRecord(message)
  if (!rec) return null

  const candidates: [string, string][] = [
    ['temperature', 'temperature'],
    ['fluidPressure', 'pressure'],
    ['illuminance', 'illuminance'],
    ['relativeHumidity', 'humidity'],
    ['range', 'range'],
    ['percentage', 'battery %'],
    ['voltage', 'voltage'],
    ['data', 'data'],
  ]

  for (const [key, label] of candidates) {
    if (typeof rec[key] === 'number') return { label, value: num(rec[key]) }
  }

  if (typeof rec.data === 'boolean') {
    return { label: 'bool', value: rec.data ? 1 : 0 }
  }

  return null
}

export default function ScalarVisualizer({ message }: VisualizerProps) {
  const [history, setHistory] = useState<SeriesPoint[]>([])
  const current = extractScalar(message)

  useEffect(() => {
    if (!current) return
    setHistory((h) => pushPoint(h, { t: Date.now(), v: current.value }))
  }, [message]) // eslint-disable-line react-hooks/exhaustive-deps

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting…</div>
  }
  if (!current) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">No scalar field</div>
  }

  return (
    <div className="h-full flex flex-col p-3 gap-2 min-h-0">
      <div className="flex items-baseline gap-2 shrink-0">
        <span className="font-mono text-3xl text-bus-cyan tabular-nums">{current.value.toFixed(4)}</span>
        <span className="font-mono text-xs text-bus-muted uppercase">{current.label}</span>
      </div>
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={history}>
            <XAxis dataKey="t" hide />
            <YAxis width={40} tick={{ fill: '#5a6370', fontSize: 9 }} domain={['auto', 'auto']} />
            <Tooltip
              contentStyle={{ background: '#1a1d20', border: '1px solid #2a2f35', fontSize: 11 }}
              labelFormatter={() => ''}
            />
            <Line type="monotone" dataKey="v" stroke="#00d4ff" dot={false} strokeWidth={1.5} isAnimationActive={false} />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  )
}
