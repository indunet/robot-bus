'use client'

import { asRecord, num } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

type Xform = {
  parent: string
  child: string
  x: number
  y: number
  z: number
}

function collectTransforms(message: unknown): Xform[] {
  const rec = asRecord(message)
  if (!rec) return []

  const list =
    (Array.isArray(rec.transforms) && rec.transforms) ||
    (Array.isArray(rec.frameTransforms) && rec.frameTransforms) ||
    []

  return list.map((item) => {
    const t = asRecord(item) ?? {}
    const header = asRecord(t.header)
    const transform = asRecord(t.transform) ?? t
    const translation = asRecord(transform.translation) ?? asRecord(t.translation)
    return {
      parent:
        (typeof t.parentFrameId === 'string' && t.parentFrameId) ||
        (typeof header?.frameId === 'string' && header.frameId) ||
        (typeof t.parent === 'string' && t.parent) ||
        '?',
      child:
        (typeof t.childFrameId === 'string' && t.childFrameId) ||
        (typeof t.child === 'string' && t.child) ||
        '?',
      x: num(translation?.x),
      y: num(translation?.y),
      z: num(translation?.z),
    }
  })
}

export default function TfSummaryVisualizer({ message }: VisualizerProps) {
  const rows = collectTransforms(message)

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting…</div>
  }

  return (
    <div className="h-full overflow-auto">
      <table className="w-full text-left font-mono text-[11px]">
        <thead className="sticky top-0 bg-bus-panel border-b border-bus-border text-bus-muted">
          <tr>
            <th className="px-2 py-1.5 font-normal">Parent</th>
            <th className="px-2 py-1.5 font-normal">Child</th>
            <th className="px-2 py-1.5 font-normal text-right">x</th>
            <th className="px-2 py-1.5 font-normal text-right">y</th>
            <th className="px-2 py-1.5 font-normal text-right">z</th>
          </tr>
        </thead>
        <tbody>
          {rows.length === 0 ? (
            <tr>
              <td colSpan={5} className="px-2 py-4 text-center text-bus-muted">
                No transforms
              </td>
            </tr>
          ) : (
            rows.map((r, i) => (
              <tr key={`${r.parent}-${r.child}-${i}`} className="border-b border-[#1f2428]">
                <td className="px-2 py-1 text-bus-text truncate max-w-[8rem]">{r.parent}</td>
                <td className="px-2 py-1 text-bus-cyan truncate max-w-[8rem]">{r.child}</td>
                <td className="px-2 py-1 text-right tabular-nums text-bus-muted">{r.x.toFixed(3)}</td>
                <td className="px-2 py-1 text-right tabular-nums text-bus-muted">{r.y.toFixed(3)}</td>
                <td className="px-2 py-1 text-right tabular-nums text-bus-muted">{r.z.toFixed(3)}</td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  )
}
