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
} from 'robot-bus/robot_bus_interfaces/action/v1/multi_waypoint_navigation'
import {
  PointNavigationFeedback,
  PointNavigationGoal,
  PointNavigationResult,
} from 'robot-bus/robot_bus_interfaces/action/v1/point_navigation'
import {
  ResetRequest,
  ResetResponse,
} from 'robot-bus/robot_bus_interfaces/srv/v1/reset'
import { useI18n } from '@/lib/i18n'
import {
  CMD_VEL_TOPIC,
  HOME_POSE,
  MULTI_WAYPOINT_NAV_ACTION,
  POINT_NAV_ACTION,
  POSE_TOPIC,
  RESET_SERVICE,
  acquireTankSession,
  heartbeatTankSession,
  releaseTankSession,
  resolveGrpcUrl,
} from '@/lib/tank'
import TankControls from './TankControls'
import TankMap from './TankMap'
import { drawWorld } from './drawWorld'
import {
  SIM_STALE_MS,
  TANK_SPRITE_SRC,
  activeGoalIndex,
  appendTrail,
  clientToWorld,
  directionFromCode,
  estimateNavProgress,
  overlayFor,
  velocityFromPressed,
  type Capability,
  type Direction,
  type GoalDrag,
  type NavPhase,
  type Overlay,
  type Point,
  type Pose,
} from './model'

interface Props {
  compact?: boolean
  autoFocus?: boolean
}

/** Browser viz / ops panel — SUB pose, PUB cmd_vel, action/service clients. */
export default function TankPanel({ compact = false, autoFocus = false }: Props) {
  const { t } = useI18n()
  const rootRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const spriteRef = useRef<HTMLImageElement | null>(null)
  const pressedRef = useRef(new Set<Direction>())
  const publishStopRef = useRef<(() => void) | null>(null)
  const publishNowRef = useRef<((linear: number, angular: number) => void) | null>(null)
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
  const dragRef = useRef<GoalDrag | null>(null)
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
  const [waypoints, setWaypoints] = useState<Pose[]>([])
  const [pointGoal, setPointGoal] = useState<Pose | null>(null)
  const [navPhase, setNavPhase] = useState<NavPhase>('idle')
  const [navProgress, setNavProgress] = useState(0)
  const [navDetail, setNavDetail] = useState('')
  const [resetting, setResetting] = useState(false)

  const syncOverlay = (goals: Pose[], activeIndex = -1, cap: Capability = capability) => {
    overlayRef.current = overlayFor(goals, activeIndex, cap)
    dirtyRef.current = true
  }

  const setNavPhaseBoth = (phase: NavPhase) => {
    navPhaseRef.current = phase
    setNavPhase(phase)
  }

  const recompute = () => {
    const next = velocityFromPressed(pressedRef.current)
    velocityRef.current = next
    setVelocity(next)
    setPressed(new Set(pressedRef.current))
  }

  const flushCmd = () => {
    forcePublishRef.current = true
    const { linear, angular } = velocityRef.current
    publishNowRef.current?.(linear, angular)
  }

  const press = (direction: Direction) => {
    if (capability !== 'keyboard') return
    if (pressedRef.current.has(direction)) return
    pressedRef.current.add(direction)
    recompute()
    flushCmd()
  }

  const release = (direction: Direction) => {
    if (!pressedRef.current.delete(direction)) return
    recompute()
    flushCmd()
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
    flushCmd()
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

  const failNav = (epoch: number, detail: string) => {
    if (navEpochRef.current !== epoch) return
    cancelNavRef.current = null
    navPlanRef.current = null
    setNavPhaseBoth('failed')
    setNavDetail(detail)
  }

  useEffect(() => {
    if (autoFocus) rootRef.current?.focus()
  }, [autoFocus])

  useEffect(() => {
    let disposed = false
    let frame = 0
    let pubTimer: ReturnType<typeof setInterval> | undefined
    let heartbeatTimer: ReturnType<typeof setInterval> | undefined
    let node: ReturnType<typeof RobotBusNode.wsAt> | undefined
    let sessionId: string | undefined
    const current = { ...HOME_POSE }
    trailRef.current = [{ x: current.x, y: current.y }]
    dirtyRef.current = true

    const paint = () => {
      drawWorld(canvasRef.current, current, trailRef.current, spriteRef.current, overlayRef.current)
    }
    paint()

    const mapEl = canvasRef.current
    const resizeObserver =
      mapEl && typeof ResizeObserver !== 'undefined'
        ? new ResizeObserver(() => {
            dirtyRef.current = true
            paint()
          })
        : null
    if (mapEl && resizeObserver) resizeObserver.observe(mapEl)

    const img = new Image()
    img.onload = () => {
      spriteRef.current = img
      dirtyRef.current = true
    }
    img.src = TANK_SPRITE_SRC
    spriteRef.current = img

    void (async () => {
      let session: Awaited<ReturnType<typeof acquireTankSession>> | undefined
      for (let attempt = 0; attempt < 5 && !disposed; attempt += 1) {
        try {
          session = await acquireTankSession()
          break
        } catch (err) {
          console.warn('tank session acquire failed', err)
          await new Promise((r) => setTimeout(r, 300 * (attempt + 1)))
        }
      }
      if (!session) {
        if (!disposed) setBusOk(false)
        return
      }
      if (disposed) {
        await releaseTankSession(session.sessionId)
        return
      }
      sessionId = session.sessionId
      setViewers(session.viewers)
      setSessionReady(true)
      const beatMs = Math.max(2000, Math.floor(session.leaseMs / 3))
      heartbeatTimer = setInterval(() => {
        if (!sessionId) return
        void heartbeatTankSession(sessionId)
          .then((s) => {
            if (!disposed) setViewers(s.viewers)
          })
          .catch((err) => {
            console.warn('tank heartbeat failed', err)
          })
      }, beatMs)

      const url = await resolveGrpcUrl()
      if (disposed) return

      node = RobotBusNode.wsAt('tank_viz', url)
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

      publishNowRef.current = (linear, angular) => {
        void publishTwist(linear, angular)
      }
      publishStopRef.current = () => {
        publishNowRef.current?.(0, 0)
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
          trailRef.current = appendTrail(trailRef.current, current)
          dirtyRef.current = true
          poseRef.current = { ...current }
          setPose({ ...current })
          setSimOnline(true)
          const plan = navPlanRef.current
          if (navPhaseRef.current !== 'running' || !plan) return
          const p = estimateNavProgress(current, plan.start, plan.goals)
          setNavProgress(Math.round(p * 100))
          if (plan.goals.length > 1) {
            syncOverlay(plan.goals, activeGoalIndex(plan.start, plan.goals, p), 'multi_waypoint')
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
      publishNowRef.current = null
      pointNavRef.current = null
      multiNavRef.current = null
      resetClientRef.current = null
      node?.shutdown()
      if (sessionId) void releaseTankSession(sessionId)
      window.removeEventListener('blur', onWindowBlur)
      spriteRef.current = null
      resizeObserver?.disconnect()
    }
    // stop is stable enough via refs; mount-only bus wiring
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const onKeyDown = (event: ReactKeyboardEvent) => {
    if (capability !== 'keyboard') return
    if (event.code === 'Space') {
      stop()
      event.preventDefault()
      return
    }
    const direction = directionFromCode(event.code)
    if (!direction) return
    press(direction)
    event.preventDefault()
  }

  const onKeyUp = (event: ReactKeyboardEvent) => {
    const direction = directionFromCode(event.code)
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

  const worldAt = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current
    return canvas ? clientToWorld(canvas, event.clientX, event.clientY) : null
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
          failNav(epoch, result.msg || t('tankNavFailed'))
        }
      })
      .catch((err: unknown) => {
        failNav(epoch, err instanceof Error ? err.message : t('tankNavFailed'))
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
          failNav(epoch, result.msg || t('tankNavFailed'))
        }
      })
      .catch((err: unknown) => {
        failNav(epoch, err instanceof Error ? err.message : t('tankNavFailed'))
      })
  }

  const onCanvasPointerDown = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    if (capability === 'keyboard' || capability === 'reset' || navPhase === 'running') return
    if (event.button !== 0) return
    const pt = worldAt(event)
    if (!pt) return
    event.currentTarget.setPointerCapture(event.pointerId)
    const theta = poseRef.current.theta
    if (capability === 'point_nav') {
      const goal = { x: pt.x, y: pt.y, theta }
      setPointGoal(goal)
      syncOverlay([goal], 0, 'point_nav')
      dragRef.current = { mode: 'point', index: 0, originX: pt.x, originY: pt.y, dragged: false }
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
    const pt = worldAt(event)
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
    if (!dragRef.current) return
    dragRef.current = null
    try {
      event.currentTarget.releasePointerCapture(event.pointerId)
    } catch {
      /* already released */
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
          setNavDetail(res.msg || t('tankNavFailed'))
          return
        }
        trailRef.current = [{ x: HOME_POSE.x, y: HOME_POSE.y }]
        dirtyRef.current = true
      })
      .catch((err: unknown) => {
        setNavDetail(err instanceof Error ? err.message : t('tankNavFailed'))
      })
      .finally(() => setResetting(false))
  }

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
        <TankMap
          canvasRef={canvasRef}
          compact={compact}
          capability={capability}
          pose={pose}
          onPointerDown={onCanvasPointerDown}
          onPointerMove={onCanvasPointerMove}
          onPointerUp={onCanvasPointerUp}
        />
        <TankControls
          compact={compact}
          capability={capability}
          busOk={busOk}
          simOnline={simOnline}
          sessionReady={sessionReady}
          viewers={viewers}
          pressed={pressed}
          velocity={velocity}
          pointGoal={pointGoal}
          waypoints={waypoints}
          navPhase={navPhase}
          navProgress={navProgress}
          navDetail={navDetail}
          resetting={resetting}
          onSelectCapability={selectCapability}
          onPress={press}
          onRelease={release}
          onRunPointNav={runPointNav}
          onRunMultiNav={runMultiNav}
          onUndoWaypoint={() => {
            setWaypoints((prev) => {
              const next = prev.slice(0, -1)
              syncOverlay(next, -1, 'multi_waypoint')
              return next
            })
          }}
          onClearWaypoints={() => {
            setWaypoints([])
            syncOverlay([], -1, 'multi_waypoint')
          }}
          onCancelNav={() => {
            abortServerNav()
            navPlanRef.current = null
            setNavPhaseBoth('idle')
            setNavProgress(0)
          }}
          onReset={onReset}
        />
      </div>
    </div>
  )
}
