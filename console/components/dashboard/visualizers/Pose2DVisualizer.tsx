'use client'

import { useEffect, useRef, useState } from 'react'
import { asRecord, num } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

type Pt = { x: number; y: number; yaw: number }

function yawFromQuat(q: Record<string, unknown> | null): number {
  if (!q) return 0
  const x = num(q.x)
  const y = num(q.y)
  const z = num(q.z)
  const w = num(q.w, 1)
  return Math.atan2(2 * (w * z + x * y), 1 - 2 * (y * y + z * z))
}

function extractPose(message: unknown): Pt | null {
  const rec = asRecord(message)
  if (!rec) return null

  // Pose / PoseStamped / PoseInFrame / Odometry variants
  let pose = asRecord(rec.pose)
  if (pose && asRecord(pose.pose)) pose = asRecord(pose.pose) // PoseWithCovariance
  if (!pose && asRecord(rec.position)) pose = rec

  const position = asRecord(pose?.position) ?? asRecord(rec.position)
  if (!position) return null
  const orientation = asRecord(pose?.orientation) ?? asRecord(rec.orientation)
  return {
    x: num(position.x),
    y: num(position.y),
    yaw: yawFromQuat(orientation),
  }
}

export default function Pose2DVisualizer({ message }: VisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [trail, setTrail] = useState<Pt[]>([])

  useEffect(() => {
    const p = extractPose(message)
    if (!p) return
    setTrail((t) => {
      const next = [...t, p]
      return next.length > 300 ? next.slice(next.length - 300) : next
    })
  }, [message])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const w = canvas.clientWidth || 320
    const h = canvas.clientHeight || 240
    canvas.width = w
    canvas.height = h
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    ctx.fillStyle = '#0e1012'
    ctx.fillRect(0, 0, w, h)

    if (trail.length === 0) return

    let minX = Infinity
    let maxX = -Infinity
    let minY = Infinity
    let maxY = -Infinity
    for (const p of trail) {
      minX = Math.min(minX, p.x)
      maxX = Math.max(maxX, p.x)
      minY = Math.min(minY, p.y)
      maxY = Math.max(maxY, p.y)
    }
    const pad = 0.5
    minX -= pad
    maxX += pad
    minY -= pad
    maxY += pad
    const span = Math.max(maxX - minX, maxY - minY, 1)
    const scale = (Math.min(w, h) * 0.8) / span
    const cx = w / 2
    const cy = h / 2
    const midX = (minX + maxX) / 2
    const midY = (minY + maxY) / 2

    const toScreen = (p: Pt) => ({
      x: cx + (p.x - midX) * scale,
      y: cy - (p.y - midY) * scale,
    })

    ctx.strokeStyle = '#00d4ff'
    ctx.lineWidth = 1.5
    ctx.beginPath()
    trail.forEach((p, i) => {
      const s = toScreen(p)
      if (i === 0) ctx.moveTo(s.x, s.y)
      else ctx.lineTo(s.x, s.y)
    })
    ctx.stroke()

    const last = trail[trail.length - 1]
    const s = toScreen(last)
    ctx.fillStyle = '#22c55e'
    ctx.beginPath()
    ctx.arc(s.x, s.y, 4, 0, Math.PI * 2)
    ctx.fill()
    ctx.strokeStyle = '#f59e0b'
    ctx.beginPath()
    ctx.moveTo(s.x, s.y)
    ctx.lineTo(s.x + Math.cos(last.yaw) * 16, s.y - Math.sin(last.yaw) * 16)
    ctx.stroke()
  }, [trail])

  const last = trail[trail.length - 1]

  if (!message) {
    return <div className="h-full flex items-center justify-center font-mono text-xs text-bus-muted">Waiting…</div>
  }

  return (
    <div className="h-full flex flex-col min-h-0">
      <canvas ref={canvasRef} className="flex-1 w-full min-h-0" />
      {last && (
        <div className="px-2 py-1 font-mono text-[10px] text-bus-muted shrink-0">
          x={last.x.toFixed(3)} y={last.y.toFixed(3)} yaw={last.yaw.toFixed(3)}
        </div>
      )}
    </div>
  )
}
