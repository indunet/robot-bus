'use client'

import { useEffect, useRef, useState } from 'react'
import { Node as RobotBusNode } from 'robot-bus'
import { Pose2D } from 'robot-bus/geometry_msgs/msg/v1/pose2d'
import { Twist } from 'robot-bus/geometry_msgs/msg/v1/twist'
import { useI18n } from '@/lib/i18n'
import { CMD_VEL_TOPIC, POSE_TOPIC, resolveGrpcUrl } from '@/lib/bot'

const WORLD_SIZE = 11
const MAX_TRAIL_POINTS = 4000

type Pose = { x: number; y: number; theta: number }
type Point = Pick<Pose, 'x' | 'y'>

interface Props {
  compact?: boolean
}

export default function BotSim({ compact = false }: Props) {
  const { t } = useI18n()
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const compactRef = useRef(compact)
  compactRef.current = compact
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

    drawWorld(canvasRef.current, current, trail, compactRef.current)

    void resolveGrpcUrl().then((url) => {
      if (disposed) return
      setGrpcUrl(url)

      node = RobotBusNode.grpcAt('bot_sim_node', url)
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

        drawWorld(canvasRef.current, current, trail, compactRef.current)
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
      ? t('botOnline')
      : status === 'PUBLISH ERROR'
        ? t('botPublishError')
        : t('botConnecting')

  return (
    <div className={`${compact ? 'h-full bg-transparent' : 'min-h-screen bg-bus-bg'} text-bus-text`}>
      <div className={`grid gap-3 ${compact ? 'grid-cols-[minmax(0,1fr)_150px] p-2' : 'max-w-5xl mx-auto p-3 grid-cols-1 lg:grid-cols-[minmax(0,1fr)_240px]'}`}>
        <section className={`${compact ? 'bg-bus-panel/18' : 'bg-bus-panel'} border border-white/10 rounded-sm p-2 flex items-center justify-center min-w-0 shadow-[0_1px_0_rgb(255_255_255_/08%)_inset]`}>
          <canvas
            ref={canvasRef}
            width={640}
            height={640}
            className={`w-full max-h-full aspect-square border border-bus-border ${compact ? 'bg-transparent' : 'bg-[#101214]'}`}
          />
        </section>
        <aside className={`${compact ? 'bg-bus-panel/18' : 'bg-bus-panel'} border border-white/10 rounded-sm h-fit font-mono shadow-[0_1px_0_rgb(255_255_255_/08%)_inset] ${compact ? 'p-2' : 'p-4'}`}>
          <div className="flex items-center justify-between gap-2 mb-3">
            <h1 className="text-xs text-bus-cyan">{t('botPoseTitle')}</h1>
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
            <div>{t('botSubscribe')} {CMD_VEL_TOPIC}</div>
            <div>{t('botPublish')} {POSE_TOPIC}</div>
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

function roundRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
) {
  const radius = Math.min(r, Math.abs(w) / 2, Math.abs(h) / 2)
  ctx.beginPath()
  ctx.moveTo(x + radius, y)
  ctx.arcTo(x + w, y, x + w, y + h, radius)
  ctx.arcTo(x + w, y + h, x, y + h, radius)
  ctx.arcTo(x, y + h, x, y, radius)
  ctx.arcTo(x, y, x + w, y, radius)
  ctx.closePath()
}

function drawWorld(
  canvas: HTMLCanvasElement | null,
  pose: Pose,
  trail: Point[],
  translucent = false,
) {
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const { width, height } = canvas
  const scale = width / WORLD_SIZE
  ctx.clearRect(0, 0, width, height)
  ctx.fillStyle = translucent ? 'rgba(16, 18, 20, 0.42)' : '#101214'
  ctx.fillRect(0, 0, width, height)
  ctx.strokeStyle = translucent ? 'rgba(42, 47, 53, 0.7)' : '#2a2f35'
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
  ctx.save()
  ctx.translate(x, y)
  ctx.rotate(-pose.theta)
  drawBot(ctx, scale * 1.35)
  ctx.restore()
}

/** Top-down micro rover: chassis, wheels, head, sensors. */
function drawBot(ctx: CanvasRenderingContext2D, unit: number) {
  // Soft ground shadow
  ctx.fillStyle = 'rgba(0, 0, 0, 0.4)'
  ctx.beginPath()
  ctx.ellipse(unit * 0.05, unit * 0.08, unit * 0.58, unit * 0.38, 0, 0, Math.PI * 2)
  ctx.fill()

  // Side wheels
  for (const side of [-1, 1] as const) {
    ctx.fillStyle = '#151a1f'
    ctx.beginPath()
    ctx.ellipse(unit * -0.08, side * unit * 0.42, unit * 0.34, unit * 0.16, 0, 0, Math.PI * 2)
    ctx.fill()
    ctx.strokeStyle = '#4a5560'
    ctx.lineWidth = 2
    ctx.stroke()

    ctx.fillStyle = '#2c353f'
    ctx.beginPath()
    ctx.ellipse(unit * -0.08, side * unit * 0.42, unit * 0.16, unit * 0.07, 0, 0, Math.PI * 2)
    ctx.fill()
  }

  // Main rectangular chassis
  const bodyGrad = ctx.createLinearGradient(-unit * 0.5, -unit * 0.32, unit * 0.55, unit * 0.32)
  bodyGrad.addColorStop(0, '#086a84')
  bodyGrad.addColorStop(0.4, '#00c4ef')
  bodyGrad.addColorStop(1, '#007fa0')
  ctx.fillStyle = bodyGrad
  roundRect(ctx, -unit * 0.5, -unit * 0.32, unit * 0.88, unit * 0.64, unit * 0.12)
  ctx.fill()
  ctx.strokeStyle = '#9af0ff'
  ctx.lineWidth = 2.2
  ctx.stroke()

  // Shoulder / arm stubs (reads as a robot, not a car blob)
  for (const side of [-1, 1] as const) {
    ctx.fillStyle = '#00a8cc'
    roundRect(ctx, -unit * 0.18, side * unit * 0.34 - unit * 0.07, unit * 0.28, unit * 0.14, unit * 0.04)
    ctx.fill()
  }

  // Upper deck / battery pack
  ctx.fillStyle = '#0c1218'
  roundRect(ctx, -unit * 0.34, -unit * 0.18, unit * 0.5, unit * 0.36, unit * 0.07)
  ctx.fill()
  ctx.strokeStyle = '#00d4ff'
  ctx.lineWidth = 1.4
  ctx.stroke()

  // Lidar / head dome
  const domeGrad = ctx.createRadialGradient(-unit * 0.06, -unit * 0.05, 0, -unit * 0.02, 0, unit * 0.18)
  domeGrad.addColorStop(0, '#d8faff')
  domeGrad.addColorStop(0.45, '#00d4ff')
  domeGrad.addColorStop(1, '#006a88')
  ctx.fillStyle = domeGrad
  ctx.beginPath()
  ctx.arc(-unit * 0.02, 0, unit * 0.18, 0, Math.PI * 2)
  ctx.fill()
  ctx.strokeStyle = '#ffffff'
  ctx.lineWidth = 1.2
  ctx.stroke()

  // Antenna
  ctx.strokeStyle = '#c8ced6'
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(-unit * 0.28, -unit * 0.1)
  ctx.lineTo(-unit * 0.42, -unit * 0.36)
  ctx.stroke()
  ctx.fillStyle = '#f59e0b'
  ctx.beginPath()
  ctx.arc(-unit * 0.42, -unit * 0.36, unit * 0.05, 0, Math.PI * 2)
  ctx.fill()

  // Front sensor face / bumper
  ctx.fillStyle = '#0a1016'
  roundRect(ctx, unit * 0.28, -unit * 0.22, unit * 0.2, unit * 0.44, unit * 0.06)
  ctx.fill()
  // "eyes"
  ctx.fillStyle = '#00d4ff'
  ctx.beginPath()
  ctx.arc(unit * 0.38, -unit * 0.09, unit * 0.045, 0, Math.PI * 2)
  ctx.fill()
  ctx.beginPath()
  ctx.arc(unit * 0.38, unit * 0.09, unit * 0.045, 0, Math.PI * 2)
  ctx.fill()
  ctx.fillStyle = '#22c55e'
  ctx.beginPath()
  ctx.arc(unit * 0.38, 0, unit * 0.03, 0, Math.PI * 2)
  ctx.fill()

  // Heading chevron
  ctx.fillStyle = '#f59e0b'
  ctx.beginPath()
  ctx.moveTo(unit * 0.58, 0)
  ctx.lineTo(unit * 0.42, -unit * 0.1)
  ctx.lineTo(unit * 0.42, unit * 0.1)
  ctx.closePath()
  ctx.fill()

  // Status LED
  ctx.fillStyle = '#22c55e'
  ctx.beginPath()
  ctx.arc(-unit * 0.38, unit * 0.18, unit * 0.045, 0, Math.PI * 2)
  ctx.fill()
}
