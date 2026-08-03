'use client'

import { asRecord, num } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

/** Duck-typed Joy visualizer (axes + buttons arrays). */
export default function JoyVisualizer({ message }: VisualizerProps) {
  const rec = asRecord(message)
  const axes = Array.isArray(rec?.axes) ? (rec!.axes as number[]) : []
  const buttons = Array.isArray(rec?.buttons) ? (rec!.buttons as number[]) : []

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting…</div>
  }
  if (!axes.length && !buttons.length) {
    return (
      <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted p-3 text-center">
        No axes/buttons (Joy stub may be missing — use Raw JSON)
      </div>
    )
  }

  return (
    <div className="h-full overflow-auto p-3 space-y-4">
      <section>
        <h3 className="font-mono text-[10px] text-bus-muted uppercase mb-2">Axes</h3>
        <div className="space-y-1.5">
          {axes.map((v, i) => (
            <div key={i} className="flex items-center gap-2">
              <span className="font-mono text-[10px] text-bus-muted w-6">{i}</span>
              <div className="flex-1 h-2 bg-bus-bg rounded-sm overflow-hidden relative">
                <div
                  className="absolute top-0 bottom-0 bg-bus-cyan/80"
                  style={{
                    left: `${((Math.min(1, Math.max(-1, v)) + 1) / 2) * 100}%`,
                    width: 4,
                    marginLeft: -2,
                  }}
                />
                <div className="absolute inset-y-0 left-1/2 w-px bg-bus-border" />
              </div>
              <span className="font-mono text-[11px] text-bus-text w-14 text-right tabular-nums">
                {num(v).toFixed(3)}
              </span>
            </div>
          ))}
        </div>
      </section>
      <section>
        <h3 className="font-mono text-[10px] text-bus-muted uppercase mb-2">Buttons</h3>
        <div className="flex flex-wrap gap-1.5">
          {buttons.map((v, i) => (
            <span
              key={i}
              className={`w-8 h-8 flex items-center justify-center font-mono text-[11px] rounded-sm border ${
                v
                  ? 'border-bus-cyan bg-bus-cyan/20 text-bus-cyan'
                  : 'border-bus-border text-bus-muted'
              }`}
            >
              {i}
            </span>
          ))}
        </div>
      </section>
    </div>
  )
}
