'use client'

import { asRecord, num } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

export default function CameraInfoVisualizer({ message }: VisualizerProps) {
  const rec = asRecord(message)
  if (!message || !rec) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting…</div>
  }

  const width = num(rec.width)
  const height = num(rec.height)
  const model = typeof rec.distortionModel === 'string' ? rec.distortionModel : typeof rec.distortion_model === 'string' ? rec.distortion_model : '—'
  const k = (Array.isArray(rec.k) ? rec.k : Array.isArray(rec.K) ? rec.K : []) as number[]
  const d = (Array.isArray(rec.d) ? rec.d : Array.isArray(rec.D) ? rec.D : []) as number[]

  const rows: [string, string][] = [
    ['Size', `${width} × ${height}`],
    ['Distortion', model],
    ['fx', k[0] != null ? String(k[0]) : '—'],
    ['fy', k[4] != null ? String(k[4]) : '—'],
    ['cx', k[2] != null ? String(k[2]) : '—'],
    ['cy', k[5] != null ? String(k[5]) : '—'],
    ['D', d.length ? d.map((v) => Number(v).toFixed(4)).join(', ') : '—'],
  ]

  return (
    <div className="h-full overflow-auto p-3">
      <dl className="space-y-2 font-mono text-[12px]">
        {rows.map(([k, v]) => (
          <div key={k} className="grid grid-cols-[7rem_1fr] gap-2">
            <dt className="text-bus-muted">{k}</dt>
            <dd className="text-bus-text break-all">{v}</dd>
          </div>
        ))}
      </dl>
    </div>
  )
}
