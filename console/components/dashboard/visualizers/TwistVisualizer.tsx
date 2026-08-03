'use client'

import { useEffect, useState } from 'react'
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Legend,
  Tooltip,
} from 'recharts'
import { asRecord, num, pushPoint, type SeriesPoint } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

function twistFrom(message: unknown) {
  const rec = asRecord(message)
  if (!rec) return null
  const twist = asRecord(rec.twist) ?? rec
  const linear = asRecord(twist?.linear)
  const angular = asRecord(twist?.angular)
  if (!linear && !angular) return null
  return { linear, angular }
}

export default function TwistVisualizer({ message }: VisualizerProps) {
  const [lin, setLin] = useState<SeriesPoint[]>([])
  const [ang, setAng] = useState<SeriesPoint[]>([])

  useEffect(() => {
    const t = twistFrom(message)
    if (!t) return
    const now = Date.now()
    if (t.linear) {
      setLin((h) =>
        pushPoint(h, { t: now, x: num(t.linear!.x), y: num(t.linear!.y), z: num(t.linear!.z) }),
      )
    }
    if (t.angular) {
      setAng((h) =>
        pushPoint(h, { t: now, x: num(t.angular!.x), y: num(t.angular!.y), z: num(t.angular!.z) }),
      )
    }
  }, [message])

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting…</div>
  }

  return (
    <div className="h-full grid grid-rows-2 gap-1 p-1 min-h-0">
      <Mini title="linear" data={lin} />
      <Mini title="angular" data={ang} />
    </div>
  )
}

function Mini({ title, data }: { title: string; data: SeriesPoint[] }) {
  return (
    <div className="min-h-0 flex flex-col border border-bus-border/60 rounded-sm">
      <div className="px-2 py-0.5 font-mono text-[10px] text-bus-muted uppercase">{title}</div>
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data}>
            <XAxis dataKey="t" hide />
            <YAxis width={36} tick={{ fill: '#5a6370', fontSize: 9 }} />
            <Tooltip contentStyle={{ background: '#1a1d20', border: '1px solid #2a2f35', fontSize: 11 }} labelFormatter={() => ''} />
            <Legend wrapperStyle={{ fontSize: 10 }} />
            <Line type="monotone" dataKey="x" stroke="#00d4ff" dot={false} isAnimationActive={false} />
            <Line type="monotone" dataKey="y" stroke="#22c55e" dot={false} isAnimationActive={false} />
            <Line type="monotone" dataKey="z" stroke="#f59e0b" dot={false} isAnimationActive={false} />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  )
}
