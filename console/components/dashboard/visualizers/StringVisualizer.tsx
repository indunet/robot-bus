'use client'

import { asRecord } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

export default function StringVisualizer({ message }: VisualizerProps) {
  const rec = asRecord(message)
  const text =
    typeof rec?.data === 'string'
      ? rec.data
      : typeof message === 'string'
        ? message
        : message == null
          ? null
          : JSON.stringify(rec?.data ?? message)

  if (text == null) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting…</div>
  }

  return (
    <div className="h-full overflow-auto p-4">
      <p className="font-mono text-sm text-bus-text whitespace-pre-wrap break-words">{text}</p>
    </div>
  )
}
