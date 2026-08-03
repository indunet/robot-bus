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

function quatToEuler(x: number, y: number, z: number, w: number) {
  const sinr = 2 * (w * x + y * z)
  const cosr = 1 - 2 * (x * x + y * y)
  const roll = Math.atan2(sinr, cosr)
  const sinp = 2 * (w * y - z * x)
  const pitch = Math.abs(sinp) >= 1 ? (Math.sign(sinp) * Math.PI) / 2 : Math.asin(sinp)
  const siny = 2 * (w * z + x * y)
  const cosy = 1 - 2 * (y * y + z * z)
  const yaw = Math.atan2(siny, cosy)
  return { roll, pitch, yaw }
}

export default function ImuVisualizer({ message }: VisualizerProps) {
  const [accel, setAccel] = useState<SeriesPoint[]>([])
  const [gyro, setGyro] = useState<SeriesPoint[]>([])
  const [orient, setOrient] = useState<SeriesPoint[]>([])

  useEffect(() => {
    const rec = asRecord(message)
    if (!rec) return
    const t = Date.now()
    const la = asRecord(rec.linearAcceleration) ?? asRecord(rec.linear_acceleration)
    const av = asRecord(rec.angularVelocity) ?? asRecord(rec.angular_velocity)
    const o = asRecord(rec.orientation)

    if (la) {
      setAccel((h) =>
        pushPoint(h, { t, x: num(la.x), y: num(la.y), z: num(la.z) }),
      )
    }
    if (av) {
      setGyro((h) =>
        pushPoint(h, { t, x: num(av.x), y: num(av.y), z: num(av.z) }),
      )
    }
    if (o) {
      const e = quatToEuler(num(o.x), num(o.y), num(o.z), num(o.w, 1))
      setOrient((h) =>
        pushPoint(h, { t, roll: e.roll, pitch: e.pitch, yaw: e.yaw }),
      )
    }
  }, [message])

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting for IMU…</div>
  }

  return (
    <div className="h-full grid grid-rows-3 gap-1 p-1 min-h-0">
      <ChartBlock title="linear accel (m/s²)" data={accel} keys={['x', 'y', 'z']} colors={['#00d4ff', '#22c55e', '#f59e0b']} />
      <ChartBlock title="angular vel (rad/s)" data={gyro} keys={['x', 'y', 'z']} colors={['#00d4ff', '#22c55e', '#f59e0b']} />
      <ChartBlock title="orientation (rad)" data={orient} keys={['roll', 'pitch', 'yaw']} colors={['#ef4444', '#a78bfa', '#38bdf8']} />
    </div>
  )
}

function ChartBlock({
  title,
  data,
  keys,
  colors,
}: {
  title: string
  data: SeriesPoint[]
  keys: string[]
  colors: string[]
}) {
  return (
    <div className="min-h-0 flex flex-col border border-bus-border/60 rounded-sm">
      <div className="px-2 py-0.5 font-mono text-[10px] text-bus-muted uppercase tracking-wider shrink-0">
        {title}
      </div>
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data}>
            <XAxis dataKey="t" hide />
            <YAxis width={36} tick={{ fill: '#5a6370', fontSize: 9 }} />
            <Tooltip
              contentStyle={{ background: '#1a1d20', border: '1px solid #2a2f35', fontSize: 11 }}
              labelFormatter={() => ''}
            />
            <Legend wrapperStyle={{ fontSize: 10 }} />
            {keys.map((k, i) => (
              <Line
                key={k}
                type="monotone"
                dataKey={k}
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
