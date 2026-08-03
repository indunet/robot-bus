'use client'

import { useEffect, useRef } from 'react'
import { asRecord, num } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

export default function LaserScanVisualizer({ message }: VisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    const rec = asRecord(message)
    if (!canvas || !rec) return

    const ranges = Array.isArray(rec.ranges) ? (rec.ranges as number[]) : []
    const angleMin = num(rec.angleMin ?? rec.startAngle ?? rec.angle_min)
    const angleMax = num(rec.angleMax ?? rec.endAngle ?? rec.angle_max, Math.PI)
    const angleInc =
      num(rec.angleIncrement ?? rec.angle_increment) ||
      (ranges.length > 1 ? (angleMax - angleMin) / (ranges.length - 1) : 0)
    const rangeMax = num(rec.rangeMax ?? rec.range_max, 10) || 10

    const w = canvas.clientWidth || 320
    const h = canvas.clientHeight || 240
    canvas.width = w
    canvas.height = h
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    ctx.fillStyle = '#0e1012'
    ctx.fillRect(0, 0, w, h)

    const cx = w / 2
    const cy = h * 0.55
    const scale = (Math.min(w, h) * 0.42) / rangeMax

    ctx.strokeStyle = '#2a2f35'
    ctx.lineWidth = 1
    for (let r = rangeMax / 4; r <= rangeMax; r += rangeMax / 4) {
      ctx.beginPath()
      ctx.arc(cx, cy, r * scale, 0, Math.PI * 2)
      ctx.stroke()
    }

    ctx.strokeStyle = '#00d4ff'
    ctx.fillStyle = 'rgba(0, 212, 255, 0.35)'
    ctx.beginPath()
    ctx.moveTo(cx, cy)
    let started = false
    for (let i = 0; i < ranges.length; i++) {
      const d = ranges[i]
      if (!Number.isFinite(d) || d <= 0) continue
      const a = angleMin + i * angleInc
      // ROS: 0 forward, CCW; canvas y down → flip y
      const x = cx + Math.cos(a) * d * scale
      const y = cy - Math.sin(a) * d * scale
      if (!started) {
        ctx.lineTo(x, y)
        started = true
      } else {
        ctx.lineTo(x, y)
      }
    }
    ctx.closePath()
    ctx.fill()
    ctx.stroke()

    ctx.fillStyle = '#22c55e'
    ctx.beginPath()
    ctx.arc(cx, cy, 3, 0, Math.PI * 2)
    ctx.fill()
  }, [message])

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting for scan…</div>
  }

  const rec = asRecord(message)
  const n = Array.isArray(rec?.ranges) ? rec!.ranges.length : 0

  return (
    <div className="h-full flex flex-col min-h-0">
      <canvas ref={canvasRef} className="flex-1 w-full min-h-0" />
      <div className="px-2 py-1 font-mono text-[10px] text-bus-muted shrink-0">{n} ranges</div>
    </div>
  )
}
