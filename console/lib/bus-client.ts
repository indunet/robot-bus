/** Browser gRPC-Web client helpers for console topic visualization. */

import { Node, DEFAULT_GRPC_URL, type MessageType } from 'robot-bus'

const STORAGE_KEY = 'robot-bus-console-grpc-url'

/** Normalize broker status `grpcAddr` into an http(s) base URL for gRPC-Web. */
export function normalizeGrpcUrl(raw: string | undefined | null): string {
  if (typeof window !== 'undefined') {
    try {
      const stored = window.localStorage.getItem(STORAGE_KEY)
      if (stored) return stored.replace(/\/$/, '')
    } catch {
      /* ignore */
    }
  }

  const fallback = DEFAULT_GRPC_URL
  if (!raw || raw === '—' || raw.trim() === '') return fallback

  let s = raw.trim()
  if (s.startsWith('http://') || s.startsWith('https://')) {
    return s.replace(/\/$/, '')
  }

  // SocketAddr / host:port from broker status.
  if (s.startsWith('[') || /^\d+\.\d+\.\d+\.\d+:/.test(s) || s.includes(':')) {
    // Replace unspecified bind addresses with the page host.
    if (typeof window !== 'undefined') {
      s = s
        .replace(/^0\.0\.0\.0/, window.location.hostname || '127.0.0.1')
        .replace(/^\[::\]/, window.location.hostname || '127.0.0.1')
        .replace(/^:::/, `${window.location.hostname || '127.0.0.1'}:`)
    } else {
      s = s.replace(/^0\.0\.0\.0/, '127.0.0.1').replace(/^\[::\]/, '127.0.0.1')
    }
    const proto = typeof window !== 'undefined' && window.location.protocol === 'https:' ? 'https' : 'http'
    return `${proto}://${s}`.replace(/\/$/, '')
  }

  return fallback
}

export function setGrpcUrlOverride(url: string | null): void {
  if (typeof window === 'undefined') return
  try {
    if (!url) window.localStorage.removeItem(STORAGE_KEY)
    else window.localStorage.setItem(STORAGE_KEY, url.replace(/\/$/, ''))
  } catch {
    /* ignore */
  }
}

export type Unsubscribe = () => void

/**
 * Subscribe to one topic via a dedicated GrpcNode (start after register).
 * Cleanup shuts the node down.
 */
export function subscribeTopic(
  topic: string,
  grpcUrl: string,
  onMessage: (topic: string, payload: Uint8Array | object) => void,
  msgType?: MessageType<object>,
): Unsubscribe {
  const name = `console-viz-${Math.random().toString(36).slice(2, 10)}`
  const node = Node.grpcAt(name, grpcUrl)

  if (msgType) {
    node.createSubscription(topic, onMessage as (t: string, msg: object) => void, msgType)
  } else {
    node.createSubscription(topic, onMessage as (t: string, payload: Uint8Array) => void)
  }

  node.start()
  return () => {
    try {
      node.shutdown()
    } catch {
      /* ignore */
    }
  }
}
