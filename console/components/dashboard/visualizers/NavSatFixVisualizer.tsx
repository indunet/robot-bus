'use client'

import { useEffect, useRef, useState } from 'react'
import { asRecord, num } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

type Fix = { lat: number; lon: number }

export default function NavSatFixVisualizer({ message }: VisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [trail, setTrail] = useState<Fix[]>([])
  const [current, setCurrent] = useState<Fix | null>(null)

  useEffect(() => {
    const rec = asRecord(message)
    if (!rec) return
    const lat = num(rec.latitude)
    const lon = num(rec.longitude)
    if (!Number.isFinite(lat) || !Number.isFinite(lon)) return
    const fix = { lat, lon }
    setCurrent(fix)
    setTrail((t) => {
      const next = [...t, fix]
      return next.length > 200 ? next.slice(next.length - 200) : next
    })
  }, [message])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas || trail.length === 0) return
    const w = canvas.clientWidth || 320
    const h = canvas.clientHeight || 200
    canvas.width = w
    canvas.height = h
    const ctx = canvas.getContext('2d')
    if (!ctx) return
    ctx.fillStyle = '#0e1012'
    ctx.fillRect(0, 0, w, h)

    let minLat = Infinity
    let maxLat = -Infinity
    let minLon = Infinity
    let maxLon = -Infinity
    for (const p of trail) {
      minLat = Math.min(minLat, p.lat)
      maxLat = Math.max(maxLat, p.lat)
      minLon = Math.min(minLon, p.lon)
      maxLon = Math.max(maxLon, p.lon)
    }
    const dLat = Math.max(maxLat - minLat, 1e-6)
    const dLon = Math.max(maxLon - minLon, 1e-6)

    ctx.strokeStyle = '#00d4ff'
    ctx.beginPath()
    trail.forEach((p, i) => {
      const x = ((p.lon - minLon) / dLon) * (w - 20) + 10
      const y = (1 - (p.lat - minLat) / dLat) * (h - 20) + 10
      if (i === 0) ctx.moveTo(x, y)
      else ctx.lineTo(x, y)
    })
    ctx.stroke()

    const last = trail[trail.length - 1]
    const lx = ((last.lon - minLon) / dLon) * (w - 20) + 10
    const ly = (1 - (last.lat - minLat) / dLat) * (h - 20) + 10
    ctx.fillStyle = '#22c55e'
    ctx.beginPath()
    ctx.arc(lx, ly, 4, 0, Math.PI * 2)
    ctx.fill()
  }, [trail])

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting…</div>
  }

  return (
    <div className="h-full flex flex-col min-h-0 p-2 gap-2">
      {current && (
        <div className="font-mono text-sm text-bus-cyan tabular-nums shrink-0">
          {current.lat.toFixed(6)}, {current.lon.toFixed(6)}
        </div>
      )}
      <canvas ref={canvasRef} className="flex-1 w-full min-h-0 rounded-sm border border-bus-border" />
    </div>
  )
}
