'use client'

import type { ComponentType } from 'react'

export interface VisualizerProps {
  message: unknown
  raw: Uint8Array | null
  topic: string
  typeName?: string
}

export type VisualizerDef = {
  id: string
  label: string
  /** Match protobuf full names and ROS aliases (case-sensitive preferred). */
  types: string[]
  component: ComponentType<VisualizerProps>
  /** Prefer this visualizer when multiple match. Higher wins. */
  priority?: number
}

const REGISTRY: VisualizerDef[] = []
const BY_ID = new Map<string, VisualizerDef>()

export function registerVisualizer(def: VisualizerDef): void {
  const existing = BY_ID.get(def.id)
  if (existing) {
    const idx = REGISTRY.indexOf(existing)
    if (idx >= 0) REGISTRY[idx] = def
  } else {
    REGISTRY.push(def)
  }
  BY_ID.set(def.id, def)
}

export function getVisualizer(id: string): VisualizerDef | undefined {
  return BY_ID.get(id)
}

export function listVisualizers(): VisualizerDef[] {
  return [...REGISTRY]
}

function normalize(name: string): string {
  return name.trim()
}

export function visualizersForType(typeName: string | undefined | null): VisualizerDef[] {
  if (!typeName) return REGISTRY.filter((v) => v.id === 'raw')
  const n = normalize(typeName)
  const lower = n.toLowerCase()
  const matched = REGISTRY.filter((v) =>
    v.types.some((t) => t === n || t.toLowerCase() === lower || t === '*'),
  )
  matched.sort((a, b) => (b.priority ?? 0) - (a.priority ?? 0))
  if (!matched.some((v) => v.id === 'raw')) {
    const raw = BY_ID.get('raw')
    if (raw) matched.push(raw)
  }
  return matched.length ? matched : REGISTRY.filter((v) => v.id === 'raw')
}

export function preferVisualizer(typeName: string | undefined | null): VisualizerDef {
  const list = visualizersForType(typeName)
  return list[0] ?? BY_ID.get('raw')!
}
