'use client'

import { useEffect, useRef } from 'react'
import { asRecord, num } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

export default function OccupancyGridVisualizer({ message }: VisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    const rec = asRecord(message)
    if (!canvas || !rec) return
    const info = asRecord(rec.info)
    const width = num(info?.width)
    const height = num(info?.height)
    const data = Array.isArray(rec.data) ? (rec.data as number[]) : null
    if (!data || width <= 0 || height <= 0) return

    canvas.width = width
    canvas.height = height
    const ctx = canvas.getContext('2d')
    if (!ctx) return
    const img = ctx.createImageData(width, height)
    for (let i = 0; i < width * height; i++) {
      const v = data[i] ?? -1
      const di = i * 4
      if (v < 0) {
        img.data[di] = 40
        img.data[di + 1] = 44
        img.data[di + 2] = 52
      } else {
        const g = Math.round(255 * (1 - Math.min(100, v) / 100))
        img.data[di] = g
        img.data[di + 1] = g
        img.data[di + 2] = g
      }
      img.data[di + 3] = 255
    }
    ctx.putImageData(img, 0, 0)
  }, [message])

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting…</div>
  }

  const info = asRecord(asRecord(message)?.info)
  const res = num(info?.resolution)

  return (
    <div className="h-full flex flex-col items-center justify-center gap-2 p-2 bg-black/40">
      <canvas ref={canvasRef} className="max-w-full max-h-[90%] image-pixelated" style={{ imageRendering: 'pixelated' }} />
      <span className="font-mono text-[10px] text-bus-muted">
        {num(info?.width)}×{num(info?.height)} · res {res}
      </span>
    </div>
  )
}
