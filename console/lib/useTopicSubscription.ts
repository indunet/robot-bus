'use client'

import { useEffect, useRef, useState } from 'react'
import type { MessageType } from 'robot-bus'
import { subscribeTopic } from './bus-client'

export type SubscriptionStatus = 'idle' | 'connecting' | 'live' | 'error'

export interface TopicSubscriptionState<T = unknown> {
  message: T | null
  raw: Uint8Array | null
  status: SubscriptionStatus
  error: string | null
  msgCount: number
  lastAt: number
  hz: number
}

/**
 * Subscribe to a topic with optional protobuf decode + UI update throttling.
 */
export function useTopicSubscription<T extends object = object>(options: {
  topic: string
  grpcUrl: string
  msgType?: MessageType<T>
  enabled?: boolean
  /** Cap React updates (default 15 Hz). */
  maxHz?: number
}): TopicSubscriptionState<T> {
  const { topic, grpcUrl, msgType, enabled = true, maxHz = 15 } = options
  const [state, setState] = useState<TopicSubscriptionState<T>>({
    message: null,
    raw: null,
    status: 'idle',
    error: null,
    msgCount: 0,
    lastAt: 0,
    hz: 0,
  })

  const pendingRef = useRef<{ msg: T | null; raw: Uint8Array | null; at: number } | null>(null)
  const countRef = useRef(0)
  const windowStartRef = useRef(0)
  const windowCountRef = useRef(0)
  const rafRef = useRef(0)
  const lastFlushRef = useRef(0)
  const minInterval = 1000 / Math.max(1, maxHz)

  useEffect(() => {
    if (!enabled || !topic || !grpcUrl) {
      setState((s) => ({ ...s, status: 'idle' }))
      return
    }

    countRef.current = 0
    windowStartRef.current = performance.now()
    windowCountRef.current = 0
    setState({
      message: null,
      raw: null,
      status: 'connecting',
      error: null,
      msgCount: 0,
      lastAt: 0,
      hz: 0,
    })

    const flush = () => {
      rafRef.current = 0
      const pending = pendingRef.current
      if (!pending) return
      pendingRef.current = null
      lastFlushRef.current = performance.now()

      const now = pending.at
      if (now - windowStartRef.current >= 1000) {
        const hz = windowCountRef.current / ((now - windowStartRef.current) / 1000)
        windowStartRef.current = now
        windowCountRef.current = 0
        setState((s) => ({
          ...s,
          message: pending.msg,
          raw: pending.raw,
          status: 'live',
          error: null,
          msgCount: countRef.current,
          lastAt: pending.at,
          hz: Math.round(hz * 10) / 10,
        }))
      } else {
        setState((s) => ({
          ...s,
          message: pending.msg,
          raw: pending.raw,
          status: 'live',
          error: null,
          msgCount: countRef.current,
          lastAt: pending.at,
        }))
      }
    }

    const schedule = () => {
      const now = performance.now()
      if (now - lastFlushRef.current >= minInterval) {
        if (rafRef.current) cancelAnimationFrame(rafRef.current)
        flush()
        return
      }
      if (!rafRef.current) {
        rafRef.current = requestAnimationFrame(flush)
      }
    }

    let unsub: (() => void) | undefined
    try {
      unsub = subscribeTopic(
        topic,
        grpcUrl,
        (_t, payload) => {
          countRef.current += 1
          windowCountRef.current += 1
          const at = performance.now()
          if (payload instanceof Uint8Array) {
            pendingRef.current = { msg: null, raw: payload, at }
          } else {
            pendingRef.current = { msg: payload as T, raw: null, at }
          }
          schedule()
        },
        msgType as MessageType<object> | undefined,
      )
    } catch (err) {
      setState((s) => ({
        ...s,
        status: 'error',
        error: err instanceof Error ? err.message : String(err),
      }))
      return
    }

    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current)
      unsub?.()
    }
  }, [topic, grpcUrl, msgType, enabled, minInterval])

  return state
}
