'use client'

/** Shared helpers for chart/history visualizers. */

export type SeriesPoint = { t: number; [key: string]: number }

const MAX_POINTS = 120

export function pushPoint(history: SeriesPoint[], point: SeriesPoint, max = MAX_POINTS): SeriesPoint[] {
  const next = [...history, point]
  return next.length > max ? next.slice(next.length - max) : next
}

export function jsonSafe(value: unknown, depth = 0): unknown {
  if (value == null) return value
  if (value instanceof Uint8Array) {
    return `<bytes ${value.byteLength}>`
  }
  if (ArrayBuffer.isView(value)) {
    return `<typed-array ${(value as ArrayBufferView).byteLength}>`
  }
  if (typeof value === 'bigint') return value.toString()
  if (typeof value !== 'object') return value
  if (depth > 6) return '…'
  if (Array.isArray(value)) {
    if (value.length > 64) {
      return [...value.slice(0, 32).map((v) => jsonSafe(v, depth + 1)), `…+${value.length - 32}`]
    }
    return value.map((v) => jsonSafe(v, depth + 1))
  }
  const out: Record<string, unknown> = {}
  for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
    out[k] = jsonSafe(v, depth + 1)
  }
  return out
}

export function hexPreview(bytes: Uint8Array, max = 64): string {
  const slice = bytes.subarray(0, max)
  const hex = [...slice].map((b) => b.toString(16).padStart(2, '0')).join(' ')
  return bytes.length > max ? `${hex} …` : hex
}

export function asRecord(message: unknown): Record<string, unknown> | null {
  if (message && typeof message === 'object' && !Array.isArray(message)) {
    return message as Record<string, unknown>
  }
  return null
}

export function num(v: unknown, fallback = 0): number {
  return typeof v === 'number' && Number.isFinite(v) ? v : fallback
}
