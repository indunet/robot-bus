'use client'

import { useEffect, useRef, useState } from 'react'
import { Node as RobotBusNode } from 'robot-bus'
import { Pose2D } from 'robot-bus/geometry_msgs/msg/v1/pose2d'
import { Twist } from 'robot-bus/geometry_msgs/msg/v1/twist'
import { useI18n } from '@/lib/i18n'
import { CMD_VEL_TOPIC, POSE_TOPIC, resolveGrpcUrl } from '@/lib/turtle'

const WORLD_SIZE = 11
const MAX_TRAIL_POINTS = 4000

type Pose = { x: number; y: number; theta: number }
type Point = Pick<Pose, 'x' | 'y'>

interface Props {
  compact?: boolean
}

export default function TurtleSim({ compact = false }: Props) {
  const { t } = useI18n()
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [pose, setPose] = useState<Pose>({
    x: WORLD_SIZE / 2,
    y: WORLD_SIZE / 2,
    theta: 0,
  })
  const [status, setStatus] = useState('CONNECTING')
  const [grpcUrl, setGrpcUrl] = useState('—')

  useEffect(() => {
    let disposed = false
    let frame = 0
    let node: ReturnType<typeof RobotBusNode.grpcAt> | undefined
    let lastFrame = performance.now()
    let lastPublish = 0
    let lastCommand = 0
    const current = { x: WORLD_SIZE / 2, y: WORLD_SIZE / 2, theta: 0 }
    const velocity = { linear: 0, angular: 0 }
    const trail: Point[] = [{ x: current.x, y: current.y }]

    drawWorld(canvasRef.current, current, trail)

    void resolveGrpcUrl().then((url) => {
      if (disposed) return
      setGrpcUrl(url)

      node = RobotBusNode.grpcAt('turtle_sim_node', url)
      const publisher = node.createPublisher(POSE_TOPIC, Pose2D)
      node.createSubscription(
        CMD_VEL_TOPIC,
        (_topic, message: Twist) => {
          velocity.linear = message.linear?.x ?? 0
          velocity.angular = message.angular?.z ?? 0
          lastCommand = performance.now()
        },
        Twist,
      )
      node.start()
      setStatus('ONLINE')

      const tick = (now: number) => {
        if (disposed) return
        const dt = Math.min(0.05, (now - lastFrame) / 1000)
        lastFrame = now

        if (now - lastCommand > 400) {
          velocity.linear = 0
          velocity.angular = 0
        }

        current.theta += velocity.angular * dt
        current.x = clamp(
          current.x + Math.cos(current.theta) * velocity.linear * dt,
          0,
          WORLD_SIZE,
        )
        current.y = clamp(
          current.y + Math.sin(current.theta) * velocity.linear * dt,
          0,
          WORLD_SIZE,
        )

        const last = trail[trail.length - 1]
        if (Math.hypot(current.x - last.x, current.y - last.y) >= 0.02) {
          trail.push({ x: current.x, y: current.y })
          if (trail.length > MAX_TRAIL_POINTS) trail.splice(0, trail.length - MAX_TRAIL_POINTS)
        }

        drawWorld(canvasRef.current, current, trail)
        setPose({ ...current })

        if (now - lastPublish >= 50) {
          lastPublish = now
          void publisher
            .publish(Pose2D.create(current))
            .catch(() => setStatus('PUBLISH ERROR'))
        }
        frame = requestAnimationFrame(tick)
      }

      frame = requestAnimationFrame(tick)
    })

    return () => {
      disposed = true
      cancelAnimationFrame(frame)
      node?.shutdown()
    }
  }, [])

  const statusLabel =
    status === 'ONLINE'
      ? t('turtleOnline')
      : status === 'PUBLISH ERROR'
        ? t('turtlePublishError')
        : t('turtleConnecting')

  return (
    <div className={`${compact ? 'h-full bg-transparent' : 'min-h-screen bg-bus-bg'} text-bus-text`}>
      <div className={`grid gap-3 ${compact ? 'grid-cols-[minmax(0,1fr)_150px] p-2' : 'max-w-5xl mx-auto p-3 grid-cols-1 lg:grid-cols-[minmax(0,1fr)_240px]'}`}>
        <section className={`${compact ? 'bg-bus-panel/76' : 'bg-bus-panel'} border border-bus-border/90 rounded-sm p-2 flex items-center justify-center min-w-0`}>
          <canvas
            ref={canvasRef}
            width={640}
            height={640}
            className="w-full max-h-full aspect-square bg-[#101214] border border-bus-border"
          />
        </section>
        <aside className={`${compact ? 'bg-bus-panel/76' : 'bg-bus-panel'} border border-bus-border/90 rounded-sm h-fit font-mono ${compact ? 'p-2' : 'p-4'}`}>
          <div className="flex items-center justify-between gap-2 mb-3">
            <h1 className="text-xs text-bus-cyan">{t('turtlePoseTitle')}</h1>
            <span className={`text-[9px] ${status === 'ONLINE' ? 'text-bus-green' : 'text-bus-red'}`}>
              {statusLabel}
            </span>
          </div>
          <dl className="grid grid-cols-2 gap-y-1.5 text-xs">
            <dt className="text-bus-muted">x</dt>
            <dd className="text-right">{pose.x.toFixed(3)}</dd>
            <dt className="text-bus-muted">y</dt>
            <dd className="text-right">{pose.y.toFixed(3)}</dd>
            <dt className="text-bus-muted">θ</dt>
            <dd className="text-right">{pose.theta.toFixed(3)}</dd>
          </dl>
          <div className="mt-3 pt-3 border-t border-bus-border text-[9px] leading-4 text-bus-muted break-all">
            <div>{t('turtleSubscribe')} {CMD_VEL_TOPIC}</div>
            <div>{t('turtlePublish')} {POSE_TOPIC}</div>
            {!compact && <div className="mt-2">{grpcUrl}</div>}
          </div>
        </aside>
      </div>
    </div>
  )
}

function clamp(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value))
}

function drawWorld(canvas: HTMLCanvasElement | null, pose: Pose, trail: Point[]) {
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const { width, height } = canvas
  const scale = width / WORLD_SIZE
  ctx.fillStyle = '#101214'
  ctx.fillRect(0, 0, width, height)
  ctx.strokeStyle = '#2a2f35'
  ctx.lineWidth = 1

  for (let i = 0; i <= WORLD_SIZE; i += 1) {
    const p = i * scale
    ctx.beginPath()
    ctx.moveTo(p, 0)
    ctx.lineTo(p, height)
    ctx.stroke()
    ctx.beginPath()
    ctx.moveTo(0, p)
    ctx.lineTo(width, p)
    ctx.stroke()
  }

  if (trail.length > 1) {
    ctx.beginPath()
    ctx.moveTo(trail[0].x * scale, height - trail[0].y * scale)
    for (let i = 1; i < trail.length; i += 1) {
      ctx.lineTo(trail[i].x * scale, height - trail[i].y * scale)
    }
    ctx.strokeStyle = '#f59e0b'
    ctx.lineWidth = 2.5
    ctx.lineJoin = 'round'
    ctx.lineCap = 'round'
    ctx.stroke()
  }

  const x = pose.x * scale
  const y = height - pose.y * scale
  const unit = scale
  ctx.save()
  ctx.translate(x, y)
  ctx.rotate(-pose.theta)

  ctx.fillStyle = '#007fa0'
  ctx.beginPath()
  ctx.moveTo(-unit * 0.58, 0)
  ctx.lineTo(-unit * 0.82, unit * 0.13)
  ctx.lineTo(-unit * 0.82, -unit * 0.13)
  ctx.closePath()
  ctx.fill()

  for (const [legX, legY, angle] of [
    [-0.3, -0.38, -0.65],
    [0.28, -0.38, 0.65],
    [-0.3, 0.38, 0.65],
    [0.28, 0.38, -0.65],
  ] as const) {
    ctx.save()
    ctx.translate(unit * legX, unit * legY)
    ctx.rotate(angle)
    ctx.fillStyle = '#00a8cc'
    ctx.beginPath()
    ctx.ellipse(0, 0, unit * 0.24, unit * 0.12, 0, 0, Math.PI * 2)
    ctx.fill()
    ctx.restore()
  }

  ctx.fillStyle = '#00d4ff'
  ctx.beginPath()
  ctx.arc(unit * 0.62, 0, unit * 0.22, 0, Math.PI * 2)
  ctx.fill()
  ctx.fillStyle = '#101214'
  ctx.beginPath()
  ctx.arc(unit * 0.7, -unit * 0.08, unit * 0.035, 0, Math.PI * 2)
  ctx.fill()
  ctx.beginPath()
  ctx.arc(unit * 0.7, unit * 0.08, unit * 0.035, 0, Math.PI * 2)
  ctx.fill()

  ctx.fillStyle = '#007fa0'
  ctx.beginPath()
  ctx.ellipse(0, 0, unit * 0.58, unit * 0.43, 0, 0, Math.PI * 2)
  ctx.fill()
  ctx.strokeStyle = '#00d4ff'
  ctx.lineWidth = 2
  ctx.stroke()
  ctx.beginPath()
  ctx.moveTo(-unit * 0.5, 0)
  ctx.lineTo(unit * 0.5, 0)
  ctx.moveTo(0, -unit * 0.38)
  ctx.lineTo(0, unit * 0.38)
  ctx.stroke()
  ctx.restore()
}
