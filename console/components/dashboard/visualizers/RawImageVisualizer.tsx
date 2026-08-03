'use client'

import { useEffect, useRef } from 'react'
import { asRecord, num } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

function toRgba(
  encoding: string,
  width: number,
  height: number,
  step: number,
  data: Uint8Array,
): Uint8ClampedArray | null {
  const enc = encoding.toLowerCase()
  const out = new Uint8ClampedArray(width * height * 4)

  if (enc === 'rgb8') {
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const si = y * step + x * 3
        const di = (y * width + x) * 4
        out[di] = data[si]
        out[di + 1] = data[si + 1]
        out[di + 2] = data[si + 2]
        out[di + 3] = 255
      }
    }
    return out
  }
  if (enc === 'bgr8') {
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const si = y * step + x * 3
        const di = (y * width + x) * 4
        out[di] = data[si + 2]
        out[di + 1] = data[si + 1]
        out[di + 2] = data[si]
        out[di + 3] = 255
      }
    }
    return out
  }
  if (enc === 'rgba8') {
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const si = y * step + x * 4
        const di = (y * width + x) * 4
        out[di] = data[si]
        out[di + 1] = data[si + 1]
        out[di + 2] = data[si + 2]
        out[di + 3] = data[si + 3]
      }
    }
    return out
  }
  if (enc === 'mono8' || enc === '8uc1') {
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const si = y * step + x
        const di = (y * width + x) * 4
        const v = data[si]
        out[di] = v
        out[di + 1] = v
        out[di + 2] = v
        out[di + 3] = 255
      }
    }
    return out
  }
  return null
}

export default function RawImageVisualizer({ message }: VisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const rec = asRecord(message)
  const width = num(rec?.width)
  const height = num(rec?.height)
  const step = num(rec?.step, width)
  const encoding = typeof rec?.encoding === 'string' ? rec.encoding : 'rgb8'
  const data = rec?.data instanceof Uint8Array ? rec.data : null

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas || !data || width <= 0 || height <= 0) return
    const rgba = toRgba(encoding, width, height, step || width, data)
    if (!rgba) return
    canvas.width = width
    canvas.height = height
    const ctx = canvas.getContext('2d')
    if (!ctx) return
    const img = new ImageData(rgba, width, height)
    ctx.putImageData(img, 0, 0)
  }, [data, width, height, step, encoding])

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting for image…</div>
  }
  if (!data || width <= 0 || height <= 0) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Invalid image fields</div>
  }

  const rgba = toRgba(encoding, width, height, step || width, data)
  if (!rgba) {
    return (
      <div className="h-full flex items-center justify-center font-mono text-xs text-bus-amber p-3 text-center">
        Unsupported encoding: {encoding}
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col items-center justify-center gap-2 p-2 bg-black/40">
      <canvas ref={canvasRef} className="max-w-full max-h-full object-contain" />
      <span className="font-mono text-[10px] text-bus-muted">
        {width}×{height} · {encoding}
      </span>
    </div>
  )
}
