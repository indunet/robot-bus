'use client'

import { useEffect, useRef, useState, type KeyboardEvent as ReactKeyboardEvent } from 'react'
import { Node as RobotBusNode } from 'robot-bus'
import { Pose2D } from 'robot-bus/geometry_msgs/msg/v1/pose2d'
import { Twist } from 'robot-bus/geometry_msgs/msg/v1/twist'
import { useI18n } from '@/lib/i18n'
import { CMD_VEL_TOPIC, POSE_TOPIC, resolveGrpcUrl } from '@/lib/bot'

const LINEAR_SPEED = 1.5
const ANGULAR_SPEED = 1.8

type Direction = 'forward' | 'back' | 'left' | 'right'

const buttons: { direction: Direction; label: string; className: string }[] = [
  { direction: 'forward', label: '↑', className: 'col-start-2 row-start-1' },
  { direction: 'left', label: '←', className: 'col-start-1 row-start-2' },
  { direction: 'back', label: '↓', className: 'col-start-2 row-start-2' },
  { direction: 'right', label: '→', className: 'col-start-3 row-start-2' },
]

interface Props {
  compact?: boolean
  autoFocus?: boolean
}

export default function BotTeleop({ compact = false, autoFocus = false }: Props) {
  const { t } = useI18n()
  const rootRef = useRef<HTMLDivElement>(null)
  const pressedRef = useRef(new Set<Direction>())
  const publishStopRef = useRef<(() => void) | null>(null)
  const [pressed, setPressed] = useState<Set<Direction>>(() => new Set())
  const [velocity, setVelocity] = useState({ linear: 0, angular: 0 })
  const velocityRef = useRef(velocity)
  const [status, setStatus] = useState('CONNECTING')
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
    if (pressedRef.current.has(direction)) return
    pressedRef.current.add(direction)
    recompute()
  }

  const release = (direction: Direction) => {
    if (!pressedRef.current.delete(direction)) return
    recompute()
  }

  const stop = () => {
    if (pressedRef.current.size === 0 && velocityRef.current.linear === 0 && velocityRef.current.angular === 0) return
    pressedRef.current.clear()
    recompute()
    publishStopRef.current?.()
  }

  useEffect(() => {
    if (autoFocus) rootRef.current?.focus()
  }, [autoFocus])

  useEffect(() => {
    let disposed = false
    let timer: ReturnType<typeof setInterval> | undefined
    let node: ReturnType<typeof RobotBusNode.grpcAt> | undefined

    void resolveGrpcUrl().then((url) => {
      if (disposed) return
      setGrpcUrl(url)
      node = RobotBusNode.grpcAt('bot_teleop_node', url)
      const publisher = node.createPublisher(CMD_VEL_TOPIC, Twist)
      node.createSubscription(POSE_TOPIC, () => undefined, Pose2D)
      publishStopRef.current = () => {
        void publisher.publish(
          Twist.create({
            linear: { x: 0, y: 0, z: 0 },
            angular: { x: 0, y: 0, z: 0 },
          }),
        ).catch(() => {})
      }
      node.start()
      setStatus('ONLINE')

      timer = setInterval(() => {
        const { linear, angular } = velocityRef.current
        void publisher
          .publish(
            Twist.create({
              linear: { x: linear, y: 0, z: 0 },
              angular: { x: 0, y: 0, z: angular },
            }),
          )
          .catch(() => setStatus('PUBLISH ERROR'))
      }, 50)
    })

    const onWindowBlur = () => stop()
    window.addEventListener('blur', onWindowBlur)
    return () => {
      disposed = true
      if (timer) clearInterval(timer)
      velocityRef.current = { linear: 0, angular: 0 }
      publishStopRef.current?.()
      publishStopRef.current = null
      node?.shutdown()
      window.removeEventListener('blur', onWindowBlur)
    }
  }, [])

  const keyFor = (code: string): Direction | null => {
    if (code === 'ArrowUp' || code === 'KeyW') return 'forward'
    if (code === 'ArrowDown' || code === 'KeyS') return 'back'
    if (code === 'ArrowLeft' || code === 'KeyA') return 'left'
    if (code === 'ArrowRight' || code === 'KeyD') return 'right'
    return null
  }

  const onKeyDown = (event: ReactKeyboardEvent) => {
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

  const statusLabel =
    status === 'ONLINE'
      ? t('botOnline')
      : status === 'PUBLISH ERROR'
        ? t('botPublishError')
        : t('botConnecting')

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
      <main className={`${compact ? 'p-2' : 'max-w-md mx-auto p-3'}`}>
        <section className={`${compact ? 'bg-bus-panel/18' : 'bg-bus-panel'} border border-white/10 rounded-sm shadow-[0_1px_0_rgb(255_255_255_/08%)_inset] ${compact ? 'p-3' : 'p-5'}`}>
          <div className="flex items-center justify-between gap-2">
            <h1 className="font-mono text-xs text-bus-cyan">{t('botTeleopTitle')}</h1>
            <span className={`font-mono text-[9px] ${status === 'ONLINE' ? 'text-bus-green' : 'text-bus-red'}`}>
              {statusLabel}
            </span>
          </div>
          <p className="font-mono text-[10px] leading-5 text-bus-muted mt-2">
            {t('botArrowHelp')}
          </p>

          <div className={`grid grid-cols-3 gap-2 mx-auto ${compact ? 'max-w-[230px] my-3' : 'max-w-[280px] my-5'}`}>
            {buttons.map(({ direction, label, className }) => (
              <button
                type="button"
                key={direction}
                onPointerDown={(event) => {
                  event.currentTarget.setPointerCapture(event.pointerId)
                  press(direction)
                }}
                onPointerUp={() => release(direction)}
                onPointerCancel={() => release(direction)}
                className={`${className} h-12 border rounded-sm font-mono text-xs transition-all ${
                  pressed.has(direction)
                    ? 'border-bus-cyan text-bus-cyan bg-bus-cyan/20 translate-y-px shadow-inner'
                    : 'border-bus-border bg-[#1f2428] hover:border-bus-cyan-dim hover:text-bus-cyan'
                }`}
              >
                {label}
              </button>
            ))}
          </div>

          <dl className="grid grid-cols-2 gap-y-1.5 font-mono text-xs border-t border-bus-border pt-3">
            <dt className="text-bus-muted">linear.x</dt>
            <dd className="text-right">{velocity.linear.toFixed(2)}</dd>
            <dt className="text-bus-muted">angular.z</dt>
            <dd className="text-right">{velocity.angular.toFixed(2)}</dd>
          </dl>
          <div className="mt-3 font-mono text-[9px] text-bus-muted break-all">
            <div>{t('botPublish')} {CMD_VEL_TOPIC}</div>
            <div>{t('botSubscribe')} {POSE_TOPIC}</div>
            {!compact && <div className="mt-1">{grpcUrl}</div>}
          </div>
        </section>
      </main>
    </div>
  )
}
