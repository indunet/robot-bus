'use client'

import { useEffect, useState } from 'react'
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer, Legend, Tooltip } from 'recharts'
import { asRecord, num, pushPoint, type SeriesPoint } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

export default function JointStateVisualizer({ message }: VisualizerProps) {
  const [mode, setMode] = useState<'position' | 'velocity' | 'effort'>('position')
  const [history, setHistory] = useState<SeriesPoint[]>([])
  const [names, setNames] = useState<string[]>([])

  useEffect(() => {
    const rec = asRecord(message)
    if (!rec) return
    const nameList = Array.isArray(rec.name) ? (rec.name as string[]) : []
    const values = Array.isArray(rec[mode]) ? (rec[mode] as number[]) : []
    if (!nameList.length || !values.length) return
    setNames(nameList.slice(0, 8))
    const point: SeriesPoint = { t: Date.now() }
    nameList.slice(0, 8).forEach((n, i) => {
      point[n || `j${i}`] = num(values[i])
    })
    setHistory((h) => pushPoint(h, point))
  }, [message, mode])

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting…</div>
  }

  const colors = ['#00d4ff', '#22c55e', '#f59e0b', '#ef4444', '#a78bfa', '#38bdf8', '#fb7185', '#84cc16']

  return (
    <div className="h-full flex flex-col min-h-0 p-2 gap-2">
      <div className="flex gap-1 shrink-0">
        {(['position', 'velocity', 'effort'] as const).map((m) => (
          <button
            key={m}
            type="button"
            onClick={() => {
              setMode(m)
              setHistory([])
            }}
            className={`px-2 py-0.5 font-mono text-[10px] uppercase rounded-sm border ${
              mode === m
                ? 'border-bus-cyan text-bus-cyan bg-bus-cyan/10'
                : 'border-bus-border text-bus-muted hover:text-bus-text'
            }`}
          >
            {m}
          </button>
        ))}
      </div>
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={history}>
            <XAxis dataKey="t" hide />
            <YAxis width={40} tick={{ fill: '#5a6370', fontSize: 9 }} />
            <Tooltip contentStyle={{ background: '#1a1d20', border: '1px solid #2a2f35', fontSize: 11 }} labelFormatter={() => ''} />
            <Legend wrapperStyle={{ fontSize: 10 }} />
            {names.map((n, i) => (
              <Line
                key={n}
                type="monotone"
                dataKey={n || `j${i}`}
                stroke={colors[i % colors.length]}
                dot={false}
                strokeWidth={1.5}
                isAnimationActive={false}
              />
            ))}
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  )
}
