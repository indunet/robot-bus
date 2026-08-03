'use client'

import { useEffect, useMemo, useState } from 'react'
import { asRecord } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

function mimeFromFormat(format: string): string {
  const f = format.toLowerCase()
  if (f.includes('png')) return 'image/png'
  if (f.includes('webp')) return 'image/webp'
  if (f.includes('avif')) return 'image/avif'
  if (f.includes('jpeg') || f.includes('jpg')) return 'image/jpeg'
  return 'image/jpeg'
}

export default function CompressedImageVisualizer({ message }: VisualizerProps) {
  const rec = asRecord(message)
  const data = rec?.data
  const format = typeof rec?.format === 'string' ? rec.format : 'jpeg'
  const [url, setUrl] = useState<string | null>(null)
  const [err, setErr] = useState<string | null>(null)

  const bytes = useMemo(() => {
    if (data instanceof Uint8Array) return data
    return null
  }, [data])

  useEffect(() => {
    if (!bytes || bytes.byteLength === 0) {
      setUrl(null)
      return
    }
              const blob = new Blob([bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)], {
                type: mimeFromFormat(format),
              })
    const next = URL.createObjectURL(blob)
    setUrl(next)
    setErr(null)
    return () => URL.revokeObjectURL(next)
  }, [bytes, format])

  if (!message) {
    return <Empty label="Waiting for image…" />
  }
  if (!bytes) {
    return <Empty label="No image data field" />
  }
  if (err) {
    return <Empty label={err} />
  }

  return (
    <div className="h-full flex flex-col items-center justify-center gap-2 p-2 bg-black/40">
      {url ? (
        // eslint-disable-next-line @next/next/no-img-element
        <img
          src={url}
          alt="compressed"
          className="max-w-full max-h-full object-contain"
          onError={() => setErr(`Failed to decode ${format}`)}
        />
      ) : (
        <Empty label="Decoding…" />
      )}
      <span className="font-mono text-[10px] text-bus-muted">
        {format} · {bytes.byteLength} B
      </span>
    </div>
  )
}

function Empty({ label }: { label: string }) {
  return (
    <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">{label}</div>
  )
}
