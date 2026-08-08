'use client'

import {
  useEffect,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
} from 'react'
import { Node as RobotBusNode } from 'robot-bus'
import { Pose2D } from 'robot-bus/geometry_msgs/msg/v1/pose2d'
import { Twist } from 'robot-bus/geometry_msgs/msg/v1/twist'
import { useI18n } from '@/lib/i18n'
import {
  CMD_VEL_TOPIC,
  POSE_TOPIC,
  WORLD_SIZE,
  acquireBotSimSession,
  heartbeatBotSimSession,
  releaseBotSimSession,
  resolveGrpcUrl,
} from '@/lib/bot'

const MAX_TRAIL_POINTS = 4000
const LINEAR_SPEED = 1.5
const ANGULAR_SPEED = 1.8
const BOT_SPRITE_SRC = '/bot-rover.png'
const BOT_SPRITE_SIZE_M = 1.25
/** Pose older than this ⇒ Rust `bot_sim` treated as offline. */
const SIM_STALE_MS = 1500

type Pose = { x: number; y: number; theta: number }
type Point = Pick<Pose, 'x' | 'y'>
type Direction = 'forward' | 'back' | 'left' | 'right'
type Capability = 'keyboard' | 'point_nav'

const KEY_BUTTONS: { direction: Direction; label: string; className: string }[] = [
  { direction: 'forward', label: '↑', className: 'col-start-2 row-start-1' },
  { direction: 'left', label: '←', className: 'col-start-1 row-start-2' },
  { direction: 'back', label: '↓', className: 'col-start-2 row-start-2' },
  { direction: 'right', label: '→', className: 'col-start-3 row-start-2' },
]

interface Props {
  compact?: boolean
  autoFocus?: boolean
}

/** Browser viz / ops panel — one node: SUB pose, PUB cmd_vel. Physics stays in Rust `bot_sim`. */
export default function BotSimPanel({ compact = false, autoFocus = false }: Props) {
  const { t } = useI18n()
  const rootRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const spriteRef = useRef<HTMLImageElement | null>(null)
  const pressedRef = useRef(new Set<Direction>())
  const publishStopRef = useRef<(() => void) | null>(null)
  const velocityRef = useRef({ linear: 0, angular: 0 })
  const lastPoseAtRef = useRef(0)
  const forcePublishRef = useRef(false)

  const [pose, setPose] = useState<Pose>({
    x: WORLD_SIZE / 2,
    y: WORLD_SIZE / 2,
    theta: 0,
  })
  const [capability, setCapability] = useState<Capability>('keyboard')
  const [pressed, setPressed] = useState<Set<Direction>>(() => new Set())
  const [velocity, setVelocity] = useState({ linear: 0, angular: 0 })
  const [busOk, setBusOk] = useState<boolean | null>(null)
  const [simOnline, setSimOnline] = useState(false)
  const [sessionReady, setSessionReady] = useState(false)
  const [viewers, setViewers] = useState(0)
  const [grpcUrl, setGrpcUrl] = useState('—')

  const recompute = () => {
    const next = {
      linear:
        (pressedRef.current.has('forward') ? LINEAR_SPEED : 0) -
        (pressedRef.current.has('back') ? LINEAR_SPEED : 0),
      angular:
        (pressedRef.current.has('left') ? ANGULAR_SPEED : 0) -
        (pressedRef.current.has('right') ? ANGULAR_SPEED : 0),
    }
    velocityRef.current = next
    setVelocity(next)
    setPressed(new Set(pressedRef.current))
  }

  const press = (direction: Direction) => {
    if (capability !== 'keyboard') return
    if (pressedRef.current.has(direction)) return
    pressedRef.current.add(direction)
    recompute()
  }

  const release = (direction: Direction) => {
    if (!pressedRef.current.delete(direction)) return
    recompute()
  }

  const stop = () => {
    if (
      pressedRef.current.size === 0 &&
      velocityRef.current.linear === 0 &&
      velocityRef.current.angular === 0
    ) {
      return
    }
    pressedRef.current.clear()
    recompute()
    forcePublishRef.current = true
    publishStopRef.current?.()
  }

  useEffect(() => {
    if (autoFocus) rootRef.current?.focus()
  }, [autoFocus])

  useEffect(() => {
    let disposed = false
    let frame = 0
    let pubTimer: ReturnType<typeof setInterval> | undefined
    let heartbeatTimer: ReturnType<typeof setInterval> | undefined
    let node: ReturnType<typeof RobotBusNode.grpcAt> | undefined
    let sessionId: string | undefined
    const current = { x: WORLD_SIZE / 2, y: WORLD_SIZE / 2, theta: 0 }
    const trail: Point[] = [{ x: current.x, y: current.y }]
    let dirty = true

    const paint = () => {
      drawWorld(canvasRef.current, current, trail, spriteRef.current)
    }
    paint()

    const img = new Image()
    img.onload = () => {
      spriteRef.current = img
      dirty = true
    }
    img.src = BOT_SPRITE_SRC
    spriteRef.current = img

    void (async () => {
      // Retry acquire — React StrictMode remount / brief broker settle.
      let session: Awaited<ReturnType<typeof acquireBotSimSession>> | undefined
      for (let attempt = 0; attempt < 5 && !disposed; attempt += 1) {
        try {
          session = await acquireBotSimSession()
          break
        } catch (err) {
          console.warn('bot_sim session acquire failed', err)
          await new Promise((r) => setTimeout(r, 300 * (attempt + 1)))
        }
      }
      if (!session) {
        if (!disposed) setBusOk(false)
        return
      }
      if (disposed) {
        await releaseBotSimSession(session.sessionId)
        return
      }
      sessionId = session.sessionId
      setViewers(session.viewers)
      setSessionReady(true)
      const beatMs = Math.max(2000, Math.floor(session.leaseMs / 3))
      heartbeatTimer = setInterval(() => {
        if (!sessionId) return
        void heartbeatBotSimSession(sessionId)
          .then((s) => {
            if (!disposed) setViewers(s.viewers)
          })
          .catch((err) => {
            console.warn('bot_sim heartbeat failed', err)
          })
      }, beatMs)

      const url = await resolveGrpcUrl()
      if (disposed) return
      setGrpcUrl(url)

      node = RobotBusNode.grpcAt('bot_viz', url)
      const publisher = node.createPublisher(CMD_VEL_TOPIC, Twist)
      const publishTwist = (linear: number, angular: number) =>
        publisher
          .publish(
            Twist.create({
              linear: { x: linear, y: 0, z: 0 },
              angular: { x: 0, y: 0, z: angular },
            }),
          )
          .then(() => {
            if (!disposed) setBusOk(true)
          })
          .catch(() => {
            if (!disposed) setBusOk(false)
          })

      publishStopRef.current = () => {
        void publishTwist(0, 0)
      }

      node.createSubscription(
        POSE_TOPIC,
        (_topic, message: Pose2D) => {
          current.x = message.x ?? WORLD_SIZE / 2
          current.y = message.y ?? WORLD_SIZE / 2
          current.theta = message.theta ?? 0
          lastPoseAtRef.current = Date.now()

          const last = trail[trail.length - 1]
          if (Math.hypot(current.x - last.x, current.y - last.y) >= 0.02) {
            trail.push({ x: current.x, y: current.y })
            if (trail.length > MAX_TRAIL_POINTS) {
              trail.splice(0, trail.length - MAX_TRAIL_POINTS)
            }
          }
          dirty = true
          setPose({ ...current })
          setSimOnline(true)
        },
        Pose2D,
      )
      node.start()
      forcePublishRef.current = true
      void publishTwist(0, 0)

      pubTimer = setInterval(() => {
        const { linear, angular } = velocityRef.current
        const moving = linear !== 0 || angular !== 0
        if (!moving && !forcePublishRef.current) {
          if (lastPoseAtRef.current > 0) {
            setSimOnline(Date.now() - lastPoseAtRef.current < SIM_STALE_MS)
          }
          return
        }
        forcePublishRef.current = false
        void publishTwist(linear, angular)
        if (lastPoseAtRef.current > 0) {
          setSimOnline(Date.now() - lastPoseAtRef.current < SIM_STALE_MS)
        }
      }, 50)

      const tick = () => {
        if (disposed) return
        if (dirty) {
          dirty = false
          paint()
        }
        frame = requestAnimationFrame(tick)
      }
      frame = requestAnimationFrame(tick)
    })()

    const onWindowBlur = () => stop()
    window.addEventListener('blur', onWindowBlur)
    return () => {
      disposed = true
      cancelAnimationFrame(frame)
      if (pubTimer) clearInterval(pubTimer)
      if (heartbeatTimer) clearInterval(heartbeatTimer)
      velocityRef.current = { linear: 0, angular: 0 }
      publishStopRef.current?.()
      publishStopRef.current = null
      node?.shutdown()
      if (sessionId) void releaseBotSimSession(sessionId)
      window.removeEventListener('blur', onWindowBlur)
      spriteRef.current = null
    }
    // stop is stable enough via refs; mount-only bus wiring
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const keyFor = (code: string): Direction | null => {
    if (code === 'ArrowUp' || code === 'KeyW') return 'forward'
    if (code === 'ArrowDown' || code === 'KeyS') return 'back'
    if (code === 'ArrowLeft' || code === 'KeyA') return 'left'
    if (code === 'ArrowRight' || code === 'KeyD') return 'right'
    return null
  }

  const onKeyDown = (event: ReactKeyboardEvent) => {
    if (capability !== 'keyboard') return
    if (event.code === 'Space') {
      stop()
      event.preventDefault()
      return
    }
    const direction = keyFor(event.code)
    if (!direction) return
    press(direction)
    event.preventDefault()
  }

  const onKeyUp = (event: ReactKeyboardEvent) => {
    const direction = keyFor(event.code)
    if (!direction) return
    release(direction)
    event.preventDefault()
  }

  const selectCapability = (next: Capability) => {
    if (next === capability) return
    stop()
    setCapability(next)
    if (next === 'keyboard') rootRef.current?.focus()
  }

  const busLabel =
    busOk === null ? t('botConnecting') : busOk ? t('botBusOk') : t('botBusError')
  const simLabel = simOnline
    ? t('botSimOnline')
    : sessionReady
      ? t('botSimHint')
      : t('botSimStarting')
  const hint =
    busOk === false
      ? t('botBusHint')
      : !simOnline
        ? sessionReady
          ? t('botSimHint')
          : t('botSimStarting')
        : t('botSharedHint')

  const capabilities: { id: Capability; label: string; enabled: boolean; hint: string }[] = [
    {
      id: 'keyboard',
      label: t('botCapKeyboard'),
      enabled: true,
      hint: t('botArrowHelp'),
    },
    {
      id: 'point_nav',
      label: t('botCapPointNav'),
      enabled: false,
      hint: t('botCapPointNavSoon'),
    },
  ]

  return (
    <div
      ref={rootRef}
      tabIndex={0}
      onKeyDown={onKeyDown}
      onKeyUp={onKeyUp}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget)) stop()
      }}
      className={`${compact ? 'h-full bg-transparent' : 'min-h-screen bg-bus-bg'} text-bus-text outline-none focus:ring-1 focus:ring-inset focus:ring-bus-cyan/40`}
    >
      <div
        className={`grid gap-3 h-full min-h-0 ${
          compact
            ? 'grid-cols-[minmax(0,1fr)_220px] p-2'
            : 'max-w-5xl mx-auto p-3 grid-cols-1 lg:grid-cols-[minmax(0,1fr)_260px]'
        }`}
      >
        <section
          className={`${compact ? 'bg-bus-bg/55' : 'bg-bus-panel'} border border-bus-border rounded-sm p-2 flex items-center justify-center min-w-0 min-h-0 shadow-[0_1px_0_rgb(255_255_255_/06%)_inset]`}
        >
          <canvas
            ref={canvasRef}
            width={640}
            height={640}
            className={`w-full max-h-full aspect-square border border-bus-border ${compact ? 'bg-[#121518]' : 'bg-[#101214]'}`}
          />
        </section>

        <aside
          className={`${compact ? 'bg-bus-bg/55' : 'bg-bus-panel'} border border-bus-border rounded-sm font-mono shadow-[0_1px_0_rgb(255_255_255_/06%)_inset] flex flex-col min-h-0 overflow-auto ${compact ? 'p-2' : 'p-4'}`}
        >
          <div className="mb-3">
            <h1 className="text-xs text-bus-cyan mb-2">{t('botCapabilitiesTitle')}</h1>
            <div className="grid grid-cols-2 gap-1.5 text-[9px]">
              <div className="flex items-center justify-between gap-1 border border-bus-border rounded-sm px-1.5 py-1">
                <span className="text-bus-muted">{t('botBusLabel')}</span>
                <span
                  className={
                    busOk === null
                      ? 'text-bus-cyan'
                      : busOk
                        ? 'text-bus-green'
                        : 'text-bus-red'
                  }
                >
                  {busLabel}
                </span>
              </div>
              <div className="flex items-center justify-between gap-1 border border-bus-border rounded-sm px-1.5 py-1">
                <span className="text-bus-muted">{t('botSimLabel')}</span>
                <span
                  className={
                    simOnline ? 'text-bus-green' : sessionReady ? 'text-bus-cyan' : 'text-bus-amber'
                  }
                >
                  {simLabel}
                </span>
              </div>
            </div>
            <p
              className={`mt-1.5 text-[9px] leading-4 ${
                !simOnline || busOk === false ? 'text-bus-amber' : 'text-bus-muted'
              }`}
            >
              {hint}
              {viewers > 0 ? ` · ${t('botViewers', { n: viewers })}` : ''}
            </p>
          </div>

          <ul className="flex flex-col gap-1.5 mb-3">
            {capabilities.map((item) => {
              const active = capability === item.id
              return (
                <li key={item.id}>
                  <button
                    type="button"
                    disabled={!item.enabled}
                    onClick={() => selectCapability(item.id)}
                    className={`w-full text-left rounded-sm border px-2 py-1.5 transition-colors ${
                      active
                        ? 'border-bus-cyan bg-bus-cyan/15 text-bus-cyan'
                        : item.enabled
                          ? 'border-bus-border bg-[#1f2428] hover:border-bus-cyan-dim text-bus-text'
                          : 'border-bus-border/60 bg-transparent text-bus-muted cursor-not-allowed opacity-60'
                    }`}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="text-[11px]">{item.label}</span>
                      {!item.enabled && (
                        <span className="text-[8px] tracking-wider uppercase">{t('botCapSoon')}</span>
                      )}
                    </div>
                  </button>
                </li>
              )
            })}
          </ul>

          {capability === 'keyboard' && (
            <div className="border-t border-bus-border pt-3">
              <p className="text-[10px] leading-5 text-bus-muted mb-2">{t('botArrowHelp')}</p>
              <div className="grid grid-cols-3 gap-2 mx-auto max-w-[200px] mb-3">
                {KEY_BUTTONS.map(({ direction, label, className }) => (
                  <button
                    type="button"
                    key={direction}
                    onPointerDown={(event) => {
                      event.currentTarget.setPointerCapture(event.pointerId)
                      press(direction)
                    }}
                    onPointerUp={() => release(direction)}
                    onPointerCancel={() => release(direction)}
                    className={`${className} h-10 border rounded-sm text-xs transition-all ${
                      pressed.has(direction)
                        ? 'border-bus-cyan text-bus-cyan bg-bus-cyan/20 translate-y-px shadow-inner'
                        : 'border-bus-border bg-[#1f2428] hover:border-bus-cyan-dim hover:text-bus-cyan'
                    }`}
                  >
                    {label}
                  </button>
                ))}
              </div>
              <dl className="grid grid-cols-2 gap-y-1 text-[11px]">
                <dt className="text-bus-muted">linear.x</dt>
                <dd className="text-right">{velocity.linear.toFixed(2)}</dd>
                <dt className="text-bus-muted">angular.z</dt>
                <dd className="text-right">{velocity.angular.toFixed(2)}</dd>
              </dl>
            </div>
          )}

          {capability === 'point_nav' && (
            <div className="border-t border-bus-border pt-3 text-[10px] leading-5 text-bus-muted">
              {t('botCapPointNavSoon')}
            </div>
          )}

          <div className="mt-auto pt-3 border-t border-bus-border">
            <div className="text-[10px] text-bus-muted mb-1.5">{t('botPoseTitle')}</div>
            <dl className="grid grid-cols-2 gap-y-1 text-[11px] mb-3">
              <dt className="text-bus-muted">x</dt>
              <dd className="text-right">{pose.x.toFixed(3)}</dd>
              <dt className="text-bus-muted">y</dt>
              <dd className="text-right">{pose.y.toFixed(3)}</dd>
              <dt className="text-bus-muted">θ</dt>
              <dd className="text-right">{pose.theta.toFixed(3)}</dd>
            </dl>
            <div className="text-[9px] leading-4 text-bus-muted break-all">
              <div>
                {t('botSubscribe')} {POSE_TOPIC}
              </div>
              <div>
                {t('botPublish')} {CMD_VEL_TOPIC}
              </div>
              <div className="mt-1 text-bus-muted/80">{t('botViewerHint')}</div>
              <div className="mt-1 text-bus-muted/80">bot_viz ↔ bot_sim</div>
              {!compact && <div className="mt-2">{grpcUrl}</div>}
            </div>
          </div>
        </aside>
      </div>
    </div>
  )
}

function drawWorld(
  canvas: HTMLCanvasElement | null,
  pose: Pose,
  trail: Point[],
  sprite: HTMLImageElement | null,
) {
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const { width, height } = canvas
  const scale = width / WORLD_SIZE
  ctx.clearRect(0, 0, width, height)
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
  const size = BOT_SPRITE_SIZE_M * scale
  ctx.save()
  ctx.translate(x, y)
  // Canvas Y is down; world Y is up — negate theta so heading matches motion.
  ctx.rotate(-pose.theta)
  if (sprite?.complete && sprite.naturalWidth > 0) {
    ctx.drawImage(sprite, -size / 2, -size / 2, size, size)
  } else {
    drawBotFallback(ctx, size)
  }
  ctx.restore()
}

/** Simple top-down rover used until the PNG sprite loads. */
function drawBotFallback(ctx: CanvasRenderingContext2D, size: number) {
  const u = size / 2
  ctx.fillStyle = 'rgba(0,0,0,0.35)'
  ctx.beginPath()
  ctx.ellipse(u * 0.05, u * 0.08, u * 0.7, u * 0.45, 0, 0, Math.PI * 2)
  ctx.fill()
  // wheels
  ctx.fillStyle = '#1a1f24'
  ctx.fillRect(-u * 0.45, -u * 0.85, u * 0.9, u * 0.28)
  ctx.fillRect(-u * 0.45, u * 0.57, u * 0.9, u * 0.28)
  // body
  ctx.fillStyle = '#00b7d8'
  ctx.beginPath()
  ctx.roundRect(-u * 0.55, -u * 0.4, u * 1.0, u * 0.8, u * 0.15)
  ctx.fill()
  ctx.strokeStyle = '#9af0ff'
  ctx.lineWidth = 2
  ctx.stroke()
  // dome
  ctx.fillStyle = '#d8faff'
  ctx.beginPath()
  ctx.arc(-u * 0.05, 0, u * 0.22, 0, Math.PI * 2)
  ctx.fill()
  // nose
  ctx.fillStyle = '#f59e0b'
  ctx.beginPath()
  ctx.moveTo(u * 0.85, 0)
  ctx.lineTo(u * 0.4, -u * 0.22)
  ctx.lineTo(u * 0.4, u * 0.22)
  ctx.closePath()
  ctx.fill()
}
