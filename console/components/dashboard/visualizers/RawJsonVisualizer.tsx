'use client'

import { hexPreview, jsonSafe } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

export default function RawJsonVisualizer({ message, raw }: VisualizerProps) {
  let body: string
  if (message != null) {
    try {
      body = JSON.stringify(jsonSafe(message), null, 2)
    } catch {
      body = String(message)
    }
  } else if (raw) {
    body = `bytes: ${raw.byteLength}\nhex: ${hexPreview(raw)}`
  } else {
    body = 'Waiting for messages…'
  }

  return (
    <pre className="h-full overflow-auto p-3 font-mono text-[11px] leading-relaxed text-bus-text whitespace-pre-wrap break-all bg-bus-bg/40">
      {body}
    </pre>
  )
}
