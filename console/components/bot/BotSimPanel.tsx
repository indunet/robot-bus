'use client'

import {
  useEffect,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type PointerEvent as ReactPointerEvent,
} from 'react'
import { Node as RobotBusNode } from 'robot-bus'
import { Pose2D } from 'robot-bus/geometry_msgs/msg/v1/pose2d'
import { Twist } from 'robot-bus/geometry_msgs/msg/v1/twist'
import {
  MultiWaypointNavigationFeedback,
  MultiWaypointNavigationGoal,
  MultiWaypointNavigationResult,
} from 'robot-bus/robot_bus_interface/action/v1/multi_waypoint_navigation'
import {
  PointNavigationFeedback,
  PointNavigationGoal,
  PointNavigationResult,
} from 'robot-bus/robot_bus_interface/action/v1/point_navigation'
import {
  ResetRequest,
  ResetResponse,
} from 'robot-bus/robot_bus_interface/srv/v1/reset'
import { useI18n } from '@/lib/i18n'
import {
  CMD_VEL_TOPIC,
  HOME_POSE,
  MULTI_WAYPOINT_NAV_ACTION,
  POINT_NAV_ACTION,
  POSE_TOPIC,
  RESET_SERVICE,
  WORLD_SIZE,
  acquireBotSimSession,
  heartbeatBotSimSession,
  releaseBotSimSession,
  resolveGrpcUrl,
} from '@/lib/bot'

const MAX_TRAIL_POINTS = 4000
const LINEAR_SPEED = 1.5
const ANGULAR_SPEED = 1.8
const BOT_SPRITE_SRC = '/bot-rover.svg'
const BOT_SPRITE_SIZE_M = 1.25
/** Pose older than this ⇒ Rust `bot_sim` treated as offline. */
const SIM_STALE_MS = 1500

type Pose = { x: number; y: number; theta: number }
type Point = Pick<Pose, 'x' | 'y'>
type Direction = 'forward' | 'back' | 'left' | 'right'
type Capability = 'keyboard' | 'point_nav' | 'multi_waypoint'
type NavPhase = 'idle' | 'running' | 'done' | 'failed'

type Overlay = {
  goals: Pose[]
  activeIndex: number
  /** Show heading arrow for these goal indices (point goal / last multi waypoint). */
  headingIndices: number[]
}

function headingIndicesFor(capability: Capability, goals: Pose[]): number[] {
  if (goals.length === 0) return []
  if (capability === 'point_nav') return [0]
  if (capability === 'multi_waypoint') return [goals.length - 1]
  return []
}

function pathLength(points: Point[]): number {
  let len = 0
  for (let i = 1; i < points.length; i += 1) {
    len += Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y)
  }
  return len
}

/** Live progress from current pose vs planned polyline (action feedback arrives at end). */
function estimateNavProgress(current: Pose, start: Pose, goals: Pose[]): number {
  if (goals.length === 0) return 0
  const nodes = [start, ...goals]
  const total = pathLength(nodes)
  if (total < 1e-6) return 1
  let best = 0
  let bestDist = Infinity
  let covered = 0
  for (let i = 0; i < nodes.length - 1; i += 1) {
    const a = nodes[i]
    const b = nodes[i + 1]
    const seg = Math.hypot(b.x - a.x, b.y - a.y)
    if (seg < 1e-9) continue
    const t = Math.max(
      0,
      Math.min(1, ((current.x - a.x) * (b.x - a.x) + (current.y - a.y) * (b.y - a.y)) / (seg * seg)),
    )
    const px = a.x + (b.x - a.x) * t
    const py = a.y + (b.y - a.y) * t
    const d = Math.hypot(current.x - px, current.y - py)
    if (d < bestDist) {
      bestDist = d
      best = covered + seg * t
    }
    covered += seg
  }
  return Math.max(0, Math.min(1, best / total))
}

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

/** Browser viz / ops panel — one node: SUB pose, PUB cmd_vel, action/service clients. */
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
  const trailRef = useRef<Point[]>([{ x: HOME_POSE.x, y: HOME_POSE.y }])
  const dirtyRef = useRef(true)
  const cancelNavRef = useRef<(() => void) | null>(null)
  const navEpochRef = useRef(0)
  const pointNavRef = useRef<{
    sendGoal: (
      goal: ReturnType<typeof PointNavigationGoal.create>,
      opts?: { onFeedback?: (fb: PointNavigationFeedback) => void; timeoutSeconds?: number },
    ) => { cancel: () => void; result: () => Promise<PointNavigationResult> }
  } | null>(null)
  const multiNavRef = useRef<{
    sendGoal: (
      goal: ReturnType<typeof MultiWaypointNavigationGoal.create>,
      opts?: {
        onFeedback?: (fb: MultiWaypointNavigationFeedback) => void
        timeoutSeconds?: number
      },
    ) => { cancel: () => void; result: () => Promise<MultiWaypointNavigationResult> }
  } | null>(null)
  const resetClientRef = useRef<{
    call: (
      req: ReturnType<typeof ResetRequest.create>,
      timeoutSeconds?: number,
    ) => Promise<ResetResponse>
  } | null>(null)
  const overlayRef = useRef<Overlay>({ goals: [], activeIndex: -1, headingIndices: [] })
  const dragRef = useRef<{
    mode: 'point' | 'multi'
    index: number
    originX: number
    originY: number
    dragged: boolean
  } | null>(null)
  const navPlanRef = useRef<{ start: Pose; goals: Pose[] } | null>(null)
  const navPhaseRef = useRef<NavPhase>('idle')
  const poseRef = useRef<Pose>({ ...HOME_POSE })

  const [pose, setPose] = useState<Pose>({ ...HOME_POSE })
  const [capability, setCapability] = useState<Capability>('keyboard')
  const [pressed, setPressed] = useState<Set<Direction>>(() => new Set())
  const [velocity, setVelocity] = useState({ linear: 0, angular: 0 })
  const [busOk, setBusOk] = useState<boolean | null>(null)
  const [simOnline, setSimOnline] = useState(false)
  const [sessionReady, setSessionReady] = useState(false)
  const [viewers, setViewers] = useState(0)
  const [grpcUrl, setGrpcUrl] = useState('—')
  const [waypoints, setWaypoints] = useState<Pose[]>([])
  const [pointGoal, setPointGoal] = useState<Pose | null>(null)
  const [navPhase, setNavPhase] = useState<NavPhase>('idle')
  const [navProgress, setNavProgress] = useState(0)
  const [navDetail, setNavDetail] = useState('')
  const [resetting, setResetting] = useState(false)

  const syncOverlay = (
    goals: Pose[],
    activeIndex = -1,
    cap: Capability = capability,
  ) => {
    overlayRef.current = {
      goals,
      activeIndex,
      headingIndices: headingIndicesFor(cap, goals),
    }
    dirtyRef.current = true
  }

  const setNavPhaseBoth = (phase: NavPhase) => {
    navPhaseRef.current = phase
    setNavPhase(phase)
  }

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

  const dropGoalHandle = () => {
    cancelNavRef.current?.()
    cancelNavRef.current = null
  }

  /** Soft-cancel in-flight server nav without snapping pose (empty multi-waypoint goal). */
  const abortServerNav = () => {
    navEpochRef.current += 1
    dropGoalHandle()
    const client = multiNavRef.current
    if (!client) return
    const handle = client.sendGoal(MultiWaypointNavigationGoal.create({ poses: [] }), {
      timeoutSeconds: 5,
    })
    void handle.result().catch(() => {})
  }

  const clearTrail = () => {
    trailRef.current = [{ x: HOME_POSE.x, y: HOME_POSE.y }]
    dirtyRef.current = true
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
    const current = { ...HOME_POSE }
    trailRef.current = [{ x: current.x, y: current.y }]
    dirtyRef.current = true

    const paint = () => {
      drawWorld(
        canvasRef.current,
        current,
        trailRef.current,
        spriteRef.current,
        overlayRef.current,
      )
    }
    paint()

    const img = new Image()
    img.onload = () => {
      spriteRef.current = img
      dirtyRef.current = true
    }
    img.src = BOT_SPRITE_SRC
    spriteRef.current = img

    void (async () => {
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

      pointNavRef.current = node.createActionClient(
        POINT_NAV_ACTION,
        PointNavigationGoal,
        PointNavigationFeedback,
        PointNavigationResult,
      )
      multiNavRef.current = node.createActionClient(
        MULTI_WAYPOINT_NAV_ACTION,
        MultiWaypointNavigationGoal,
        MultiWaypointNavigationFeedback,
        MultiWaypointNavigationResult,
      )
      resetClientRef.current = node.createClient(RESET_SERVICE, ResetRequest, ResetResponse)

      node.createSubscription(
        POSE_TOPIC,
        (_topic, message: Pose2D) => {
          current.x = message.x ?? HOME_POSE.x
          current.y = message.y ?? HOME_POSE.y
          current.theta = message.theta ?? 0
          lastPoseAtRef.current = Date.now()

          const trail = trailRef.current
          const last = trail[trail.length - 1]
          const jump = Math.hypot(current.x - last.x, current.y - last.y)
          if (jump >= 0.02) {
            if (jump >= 2) {
              trailRef.current = [{ x: current.x, y: current.y }]
            } else {
              trail.push({ x: current.x, y: current.y })
              if (trail.length > MAX_TRAIL_POINTS) {
                trail.splice(0, trail.length - MAX_TRAIL_POINTS)
              }
            }
          }
          dirtyRef.current = true
          poseRef.current = { ...current }
          setPose({ ...current })
          setSimOnline(true)
          if (navPhaseRef.current === 'running' && navPlanRef.current) {
            const p = estimateNavProgress(current, navPlanRef.current.start, navPlanRef.current.goals)
            setNavProgress(Math.round(p * 100))
            const goals = navPlanRef.current.goals
            if (goals.length > 1) {
              let covered = 0
              const total = pathLength([navPlanRef.current.start, ...goals]) || 1
              let idx = 0
              for (let i = 0; i < goals.length; i += 1) {
                const prev = i === 0 ? navPlanRef.current.start : goals[i - 1]
                covered += Math.hypot(goals[i].x - prev.x, goals[i].y - prev.y)
                if (p * total <= covered + 1e-6) {
                  idx = i
                  break
                }
                idx = i
              }
              syncOverlay(goals, idx, 'multi_waypoint')
            }
          }
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
        if (dirtyRef.current) {
          dirtyRef.current = false
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
      dropGoalHandle()
      cancelAnimationFrame(frame)
      if (pubTimer) clearInterval(pubTimer)
      if (heartbeatTimer) clearInterval(heartbeatTimer)
      velocityRef.current = { linear: 0, angular: 0 }
      publishStopRef.current?.()
      publishStopRef.current = null
      pointNavRef.current = null
      multiNavRef.current = null
      resetClientRef.current = null
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
    abortServerNav()
    setNavPhaseBoth('idle')
    setNavProgress(0)
    setNavDetail('')
    navPlanRef.current = null
    dragRef.current = null
    if (next !== 'point_nav') setPointGoal(null)
    if (next !== 'multi_waypoint') setWaypoints([])
    syncOverlay([], -1, next)
    setCapability(next)
    if (next === 'keyboard') rootRef.current?.focus()
  }

  const clientToWorld = (clientX: number, clientY: number): Point | null => {
    const canvas = canvasRef.current
    if (!canvas) return null
    const rect = canvas.getBoundingClientRect()
    if (rect.width <= 0 || rect.height <= 0) return null
    const px = ((clientX - rect.left) / rect.width) * canvas.width
    const py = ((clientY - rect.top) / rect.height) * canvas.height
    return {
      x: Math.min(WORLD_SIZE, Math.max(0, (px / canvas.width) * WORLD_SIZE)),
      y: Math.min(WORLD_SIZE, Math.max(0, ((canvas.height - py) / canvas.height) * WORLD_SIZE)),
    }
  }

  const runPointNav = () => {
    const client = pointNavRef.current
    const goal = pointGoal
    if (!client || !goal || navPhase === 'running') return
    stop()
    dropGoalHandle()
    const epoch = ++navEpochRef.current
    syncOverlay([goal], 0, 'point_nav')
    navPlanRef.current = { start: { ...poseRef.current }, goals: [goal] }
    setNavPhaseBoth('running')
    setNavProgress(0)
    setNavDetail('')
    const handle = client.sendGoal(
      PointNavigationGoal.create({
        pose: { x: goal.x, y: goal.y, theta: goal.theta },
      }),
      {
        timeoutSeconds: 120,
        onFeedback: (fb) => {
          if (navEpochRef.current !== epoch) return
          setNavProgress(Math.round((fb.progress ?? 0) * 100))
        },
      },
    )
    cancelNavRef.current = () => handle.cancel()
    void handle
      .result()
      .then((result) => {
        if (navEpochRef.current !== epoch) return
        cancelNavRef.current = null
        navPlanRef.current = null
        if (result.success) {
          setNavPhaseBoth('done')
          setNavProgress(100)
        } else {
          setNavPhaseBoth('failed')
          setNavDetail(result.msg || t('botNavFailed'))
        }
      })
      .catch((err: unknown) => {
        if (navEpochRef.current !== epoch) return
        cancelNavRef.current = null
        navPlanRef.current = null
        setNavPhaseBoth('failed')
        setNavDetail(err instanceof Error ? err.message : t('botNavFailed'))
      })
  }

  const runMultiNav = () => {
    const client = multiNavRef.current
    if (!client || waypoints.length === 0 || navPhase === 'running') return
    stop()
    dropGoalHandle()
    const epoch = ++navEpochRef.current
    const goals = waypoints.map((w, i) =>
      i === waypoints.length - 1 ? w : { ...w, theta: 0 },
    )
    syncOverlay(goals, 0, 'multi_waypoint')
    navPlanRef.current = { start: { ...poseRef.current }, goals }
    setNavPhaseBoth('running')
    setNavProgress(0)
    setNavDetail('')
    const handle = client.sendGoal(
      MultiWaypointNavigationGoal.create({
        poses: goals.map((w) => ({ x: w.x, y: w.y, theta: w.theta })),
      }),
      {
        timeoutSeconds: 300,
        onFeedback: (fb) => {
          if (navEpochRef.current !== epoch) return
          setNavProgress(Math.round((fb.progress ?? 0) * 100))
        },
      },
    )
    cancelNavRef.current = () => handle.cancel()
    void handle
      .result()
      .then((result) => {
        if (navEpochRef.current !== epoch) return
        cancelNavRef.current = null
        navPlanRef.current = null
        if (result.success) {
          setNavPhaseBoth('done')
          setNavProgress(100)
          syncOverlay(goals, goals.length - 1, 'multi_waypoint')
        } else {
          setNavPhaseBoth('failed')
          setNavDetail(result.msg || t('botNavFailed'))
        }
      })
      .catch((err: unknown) => {
        if (navEpochRef.current !== epoch) return
        cancelNavRef.current = null
        navPlanRef.current = null
        setNavPhaseBoth('failed')
        setNavDetail(err instanceof Error ? err.message : t('botNavFailed'))
      })
  }

  const onCanvasPointerDown = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    if (capability === 'keyboard' || navPhase === 'running') return
    if (event.button !== 0) return
    const pt = clientToWorld(event.clientX, event.clientY)
    if (!pt) return
    event.currentTarget.setPointerCapture(event.pointerId)
    const theta = poseRef.current.theta
    if (capability === 'point_nav') {
      const goal = { x: pt.x, y: pt.y, theta }
      setPointGoal(goal)
      syncOverlay([goal], 0, 'point_nav')
      dragRef.current = {
        mode: 'point',
        index: 0,
        originX: pt.x,
        originY: pt.y,
        dragged: false,
      }
      return
    }
    if (capability === 'multi_waypoint') {
      setWaypoints((prev) => {
        const next = [...prev, { x: pt.x, y: pt.y, theta }]
        syncOverlay(next, next.length - 1, 'multi_waypoint')
        dragRef.current = {
          mode: 'multi',
          index: next.length - 1,
          originX: pt.x,
          originY: pt.y,
          dragged: false,
        }
        return next
      })
    }
  }

  const onCanvasPointerMove = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    const drag = dragRef.current
    if (!drag) return
    const pt = clientToWorld(event.clientX, event.clientY)
    if (!pt) return
    const dx = pt.x - drag.originX
    const dy = pt.y - drag.originY
    if (!drag.dragged && Math.hypot(dx, dy) < 0.15) return
    drag.dragged = true
    const theta = Math.atan2(dy, dx)
    if (drag.mode === 'point') {
      setPointGoal((prev) => {
        if (!prev) return prev
        const next = { ...prev, theta }
        syncOverlay([next], 0, 'point_nav')
        return next
      })
      return
    }
    setWaypoints((prev) => {
      if (drag.index < 0 || drag.index >= prev.length) return prev
      const next = prev.map((w, i) => (i === drag.index ? { ...w, theta } : w))
      syncOverlay(next, drag.index, 'multi_waypoint')
      return next
    })
  }

  const onCanvasPointerUp = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    if (dragRef.current) {
      dragRef.current = null
      try {
        event.currentTarget.releasePointerCapture(event.pointerId)
      } catch {
        /* already released */
      }
    }
  }

  const onReset = () => {
    const client = resetClientRef.current
    if (!client || resetting) return
    stop()
    dropGoalHandle()
    navEpochRef.current += 1
    navPlanRef.current = null
    setResetting(true)
    setNavPhaseBoth('idle')
    setNavProgress(0)
    setNavDetail('')
    setPointGoal(null)
    setWaypoints([])
    syncOverlay([], -1, capability)
    void client
      .call(ResetRequest.create({}), 5)
      .then((res) => {
        if (!res.success) {
          setNavDetail(res.msg || t('botNavFailed'))
        } else {
          clearTrail()
        }
      })
      .catch((err: unknown) => {
        setNavDetail(err instanceof Error ? err.message : t('botNavFailed'))
      })
      .finally(() => setResetting(false))
  }

  const cancelRunningNav = () => {
    abortServerNav()
    navPlanRef.current = null
    setNavPhaseBoth('idle')
    setNavProgress(0)
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

  const navStatusLabel =
    navPhase === 'running'
      ? t('botNavRunning', { p: navProgress })
      : navPhase === 'done'
        ? t('botNavDone')
        : navPhase === 'failed'
          ? navDetail || t('botNavFailed')
          : t('botNavIdle')

  const capabilities: { id: Capability; label: string }[] = [
    { id: 'keyboard', label: t('botCapKeyboard') },
    { id: 'point_nav', label: t('botCapPointNav') },
    { id: 'multi_waypoint', label: t('botCapMultiNav') },
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
        <section className="flex items-center justify-center min-w-0 min-h-0">
          <canvas
            ref={canvasRef}
            width={640}
            height={640}
            onPointerDown={onCanvasPointerDown}
            onPointerMove={onCanvasPointerMove}
            onPointerUp={onCanvasPointerUp}
            onPointerCancel={onCanvasPointerUp}
            className={`w-full max-h-full aspect-square rounded-sm border border-bus-border touch-none ${
              capability === 'keyboard' ? 'cursor-default' : 'cursor-crosshair'
            } ${compact ? 'bg-[#121518]/80' : 'bg-[#101214]'}`}
          />
        </section>

        <aside
          className={`${compact ? 'bg-bus-bg/25' : 'bg-bus-panel'} border border-bus-border rounded-sm font-mono shadow-[0_1px_0_rgb(255_255_255_/06%)_inset] flex flex-col min-h-0 overflow-auto ${compact ? 'p-2' : 'p-4'}`}
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
                    onClick={() => selectCapability(item.id)}
                    className={`w-full text-left rounded-sm border px-2 py-1.5 transition-colors ${
                      active
                        ? 'border-bus-cyan bg-bus-cyan/15 text-bus-cyan'
                        : 'border-bus-border bg-[#1f2428] hover:border-bus-cyan-dim text-bus-text'
                    }`}
                  >
                    <span className="text-[11px]">{item.label}</span>
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
            <div className="border-t border-bus-border pt-3 text-[10px] leading-5">
              <p className="text-bus-muted mb-2">{t('botCapPointNavHelp')}</p>
              {pointGoal && (
                <dl className="grid grid-cols-2 gap-y-1 text-[11px] mb-2">
                  <dt className="text-bus-muted">goal.x</dt>
                  <dd className="text-right">{pointGoal.x.toFixed(2)}</dd>
                  <dt className="text-bus-muted">goal.y</dt>
                  <dd className="text-right">{pointGoal.y.toFixed(2)}</dd>
                  <dt className="text-bus-muted">{t('botNavTheta')}</dt>
                  <dd className="text-right">{pointGoal.theta.toFixed(2)}</dd>
                </dl>
              )}
              <button
                type="button"
                disabled={!pointGoal || navPhase === 'running'}
                onClick={runPointNav}
                className="w-full h-8 mb-2 border border-bus-border rounded-sm text-[11px] bg-[#1f2428] hover:border-bus-cyan hover:text-bus-cyan disabled:opacity-40"
              >
                {t('botNavGo')}
              </button>
              <NavProgress
                phase={navPhase}
                progress={navProgress}
                label={navStatusLabel}
                progressLabel={t('botNavProgress')}
              />
              {navPhase === 'running' && (
                <button
                  type="button"
                  onClick={cancelRunningNav}
                  className="mt-2 w-full h-8 border border-bus-border rounded-sm text-[11px] bg-[#1f2428] hover:border-bus-amber hover:text-bus-amber"
                >
                  {t('botNavCancel')}
                </button>
              )}
            </div>
          )}

          {capability === 'multi_waypoint' && (
            <div className="border-t border-bus-border pt-3 text-[10px] leading-5">
              <p className="text-bus-muted mb-2">{t('botCapMultiNavHelp')}</p>
              <p className="text-[11px] mb-2">{t('botNavWaypoints', { n: waypoints.length })}</p>
              {waypoints.length > 0 && (
                <dl className="grid grid-cols-2 gap-y-1 text-[11px] mb-2">
                  <dt className="text-bus-muted">last.x</dt>
                  <dd className="text-right">{waypoints[waypoints.length - 1].x.toFixed(2)}</dd>
                  <dt className="text-bus-muted">last.y</dt>
                  <dd className="text-right">{waypoints[waypoints.length - 1].y.toFixed(2)}</dd>
                  <dt className="text-bus-muted">{t('botNavTheta')}</dt>
                  <dd className="text-right">{waypoints[waypoints.length - 1].theta.toFixed(2)}</dd>
                </dl>
              )}
              <div className="grid grid-cols-3 gap-1.5 mb-2">
                <button
                  type="button"
                  disabled={waypoints.length === 0 || navPhase === 'running'}
                  onClick={runMultiNav}
                  className="h-8 border border-bus-border rounded-sm text-[11px] bg-[#1f2428] hover:border-bus-cyan hover:text-bus-cyan disabled:opacity-40"
                >
                  {t('botNavGo')}
                </button>
                <button
                  type="button"
                  disabled={waypoints.length === 0 || navPhase === 'running'}
                  onClick={() => {
                    setWaypoints((prev) => {
                      const next = prev.slice(0, -1)
                      syncOverlay(next, -1, 'multi_waypoint')
                      return next
                    })
                  }}
                  className="h-8 border border-bus-border rounded-sm text-[11px] bg-[#1f2428] hover:border-bus-cyan-dim disabled:opacity-40"
                >
                  {t('botNavUndo')}
                </button>
                <button
                  type="button"
                  disabled={waypoints.length === 0 || navPhase === 'running'}
                  onClick={() => {
                    setWaypoints([])
                    syncOverlay([], -1, 'multi_waypoint')
                  }}
                  className="h-8 border border-bus-border rounded-sm text-[11px] bg-[#1f2428] hover:border-bus-amber disabled:opacity-40"
                >
                  {t('botNavClear')}
                </button>
              </div>
              <NavProgress
                phase={navPhase}
                progress={navProgress}
                label={navStatusLabel}
                progressLabel={t('botNavProgress')}
              />
              {navPhase === 'running' && (
                <button
                  type="button"
                  onClick={cancelRunningNav}
                  className="mt-2 w-full h-8 border border-bus-border rounded-sm text-[11px] bg-[#1f2428] hover:border-bus-amber hover:text-bus-amber"
                >
                  {t('botNavCancel')}
                </button>
              )}
            </div>
          )}

          <div className="mt-auto pt-3 border-t border-bus-border">
            <div className="flex items-center justify-between gap-2 mb-1.5">
              <div className="text-[10px] text-bus-muted">{t('botPoseTitle')}</div>
              <button
                type="button"
                disabled={resetting || !sessionReady}
                title={t('botResetHint')}
                onClick={onReset}
                className="h-6 px-2 border border-bus-border rounded-sm text-[10px] bg-[#1f2428] hover:border-bus-amber hover:text-bus-amber disabled:opacity-40"
              >
                {resetting ? t('botResetting') : t('botReset')}
              </button>
            </div>
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
              <div className="mt-1 text-bus-muted/80">ACT {POINT_NAV_ACTION}</div>
              <div className="text-bus-muted/80">ACT {MULTI_WAYPOINT_NAV_ACTION}</div>
              <div className="text-bus-muted/80">SRV {RESET_SERVICE}</div>
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

function NavProgress({
  phase,
  progress,
  label,
  progressLabel,
}: {
  phase: NavPhase
  progress: number
  label: string
  progressLabel: string
}) {
  const tone =
    phase === 'failed'
      ? 'text-bus-red'
      : phase === 'done'
        ? 'text-bus-green'
        : 'text-bus-cyan'
  const bar =
    phase === 'failed' ? 'bg-bus-red' : phase === 'done' ? 'bg-bus-green' : 'bg-bus-cyan'
  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between gap-2">
        <span className={`text-[11px] ${tone}`}>{label}</span>
        {(phase === 'running' || phase === 'done') && (
          <span className="text-[10px] text-bus-muted tabular-nums">
            {progressLabel} {progress}%
          </span>
        )}
      </div>
      <div className="h-1.5 rounded-full bg-[#1a1f24] border border-bus-border overflow-hidden">
        <div
          className={`h-full rounded-full transition-[width] duration-150 ${bar}`}
          style={{ width: `${phase === 'idle' ? 0 : Math.max(0, Math.min(100, progress))}%` }}
        />
      </div>
    </div>
  )
}

function drawWorld(
  canvas: HTMLCanvasElement | null,
  pose: Pose,
  trail: Point[],
  sprite: HTMLImageElement | null,
  overlay: Overlay,
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

  if (overlay.goals.length > 0) {
    ctx.beginPath()
    const first = overlay.goals[0]
    ctx.moveTo(first.x * scale, height - first.y * scale)
    for (let i = 1; i < overlay.goals.length; i += 1) {
      ctx.lineTo(overlay.goals[i].x * scale, height - overlay.goals[i].y * scale)
    }
    ctx.strokeStyle = 'rgba(0, 183, 216, 0.55)'
    ctx.lineWidth = 1.5
    ctx.setLineDash([6, 4])
    ctx.stroke()
    ctx.setLineDash([])

    const headingSet = new Set(overlay.headingIndices)
    overlay.goals.forEach((g, i) => {
      const gx = g.x * scale
      const gy = height - g.y * scale
      const active = i === overlay.activeIndex
      ctx.beginPath()
      ctx.arc(gx, gy, active ? 7 : 5, 0, Math.PI * 2)
      ctx.fillStyle = active ? '#f59e0b' : '#00b7d8'
      ctx.fill()
      ctx.fillStyle = '#d8faff'
      ctx.font = '10px ui-monospace, monospace'
      ctx.fillText(String(i + 1), gx + 8, gy - 8)

      if (headingSet.has(i)) {
        const len = 18
        const dirX = Math.cos(g.theta)
        const dirY = -Math.sin(g.theta)
        const tipX = gx + dirX * len
        const tipY = gy + dirY * len
        ctx.beginPath()
        ctx.moveTo(gx, gy)
        ctx.lineTo(tipX, tipY)
        ctx.strokeStyle = '#f59e0b'
        ctx.lineWidth = 2
        ctx.stroke()
        const nx = -dirY
        const ny = dirX
        ctx.beginPath()
        ctx.moveTo(tipX, tipY)
        ctx.lineTo(tipX - dirX * 7 + nx * 4, tipY - dirY * 7 + ny * 4)
        ctx.lineTo(tipX - dirX * 7 - nx * 4, tipY - dirY * 7 - ny * 4)
        ctx.closePath()
        ctx.fillStyle = '#f59e0b'
        ctx.fill()
      }
    })
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

/** Simple top-down rover used until the SVG sprite loads. */
function drawBotFallback(ctx: CanvasRenderingContext2D, size: number) {
  const u = size / 2
  ctx.fillStyle = 'rgba(0,0,0,0.35)'
  ctx.beginPath()
  ctx.ellipse(u * 0.05, u * 0.08, u * 0.7, u * 0.45, 0, 0, Math.PI * 2)
  ctx.fill()
  ctx.fillStyle = '#5a4634'
  ctx.fillRect(-u * 0.45, -u * 0.72, u * 0.9, u * 0.28)
  ctx.fillRect(-u * 0.45, u * 0.44, u * 0.9, u * 0.28)
  ctx.strokeStyle = '#d4a574'
  ctx.lineWidth = 1.5
  ctx.strokeRect(-u * 0.45, -u * 0.72, u * 0.9, u * 0.28)
  ctx.strokeRect(-u * 0.45, u * 0.44, u * 0.9, u * 0.28)
  ctx.fillStyle = '#c4a06a'
  for (const y of [-u * 0.65, u * 0.51]) {
    for (const x of [-u * 0.28, -u * 0.05, u * 0.18]) {
      ctx.fillRect(x, y, u * 0.16, u * 0.16)
    }
  }
  ctx.fillStyle = '#00b7d8'
  ctx.beginPath()
  ctx.roundRect(-u * 0.55, -u * 0.4, u * 1.0, u * 0.8, u * 0.15)
  ctx.fill()
  ctx.strokeStyle = '#9af0ff'
  ctx.lineWidth = 2
  ctx.stroke()
  ctx.fillStyle = '#d8faff'
  ctx.beginPath()
  ctx.arc(-u * 0.05, 0, u * 0.22, 0, Math.PI * 2)
  ctx.fill()
  // tank barrel (heading)
  ctx.strokeStyle = '#fbbf24'
  ctx.lineWidth = 1
  ctx.fillStyle = '#c9892a'
  ctx.beginPath()
  ctx.roundRect(u * 0.35, -u * 0.14, u * 0.2, u * 0.28, u * 0.04)
  ctx.fill()
  ctx.stroke()
  ctx.fillStyle = '#f59e0b'
  ctx.beginPath()
  ctx.roundRect(u * 0.5, -u * 0.08, u * 0.35, u * 0.16, u * 0.04)
  ctx.fill()
  ctx.stroke()
  ctx.fillStyle = '#d97706'
  ctx.beginPath()
  ctx.roundRect(u * 0.82, -u * 0.11, u * 0.12, u * 0.22, u * 0.03)
  ctx.fill()
  ctx.stroke()
}
