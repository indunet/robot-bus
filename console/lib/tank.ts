import { resolveBusUrl } from '@/lib/console-bus'

/** Built-in tank demo under the reserved `/robot_bus/*` namespace. */
export const TANK_PREFIX = '/robot_bus/tank'
export const CMD_VEL_TOPIC = '/robot_bus/tank/cmd_vel'
export const POSE_TOPIC = '/robot_bus/tank/pose'
export const POINT_NAV_ACTION = '/robot_bus/tank/point_navigation'
export const MULTI_WAYPOINT_NAV_ACTION = '/robot_bus/tank/multi_waypoint_navigation'
export const RESET_SERVICE = '/robot_bus/tank/reset'
/** World extents shared with the in-process `tank` (meters / arbitrary units). */
export const WORLD_SIZE = 11
/** Home pose used by `/robot_bus/tank/reset` (world center). */
export const HOME_POSE: { x: number; y: number; theta: number } = {
  x: WORLD_SIZE / 2,
  y: WORLD_SIZE / 2,
  theta: 0,
}

export type TankSessionInfo = {
  sessionId: string
  leaseMs: number
  viewers: number
}

export type TankStatusInfo = {
  /** False when broker started with `--no-tank`. */
  enabled: boolean
  running: boolean
  viewers: number
}

/** Same-origin WebSocket RPC + REST (single-port console + gateway). */
export async function resolveGrpcUrl(): Promise<string> {
  return resolveBusUrl()
}

function apiBase(): string {
  return resolveBusUrl()
}

/** Ask the broker to start / keep the shared tank singleton alive. */
export async function acquireTankSession(): Promise<TankSessionInfo> {
  const res = await fetch(`${apiBase()}/api/v1/tank/session`, { method: 'POST' })
  if (!res.ok) {
    const body = await res.text().catch(() => '')
    throw new Error(body || `acquire tank session failed (${res.status})`)
  }
  const data = (await res.json()) as {
    sessionId: string
    leaseMs: number
    viewers: number
  }
  return {
    sessionId: data.sessionId,
    leaseMs: data.leaseMs,
    viewers: data.viewers,
  }
}

export async function heartbeatTankSession(sessionId: string): Promise<TankSessionInfo> {
  const res = await fetch(`${apiBase()}/api/v1/tank/session/${encodeURIComponent(sessionId)}/heartbeat`, {
    method: 'POST',
  })
  if (!res.ok) {
    throw new Error(`tank heartbeat failed (${res.status})`)
  }
  const data = (await res.json()) as {
    sessionId: string
    leaseMs: number
    viewers: number
  }
  return {
    sessionId: data.sessionId,
    leaseMs: data.leaseMs,
    viewers: data.viewers,
  }
}

export async function releaseTankSession(sessionId: string): Promise<void> {
  try {
    await fetch(`${apiBase()}/api/v1/tank/session/${encodeURIComponent(sessionId)}`, {
      method: 'DELETE',
      keepalive: true,
    })
  } catch {
    /* ignore unload races */
  }
}

export async function fetchTankStatus(): Promise<TankStatusInfo> {
  const res = await fetch(`${apiBase()}/api/v1/tank`)
  if (!res.ok) {
    throw new Error(`tank status failed (${res.status})`)
  }
  const data = (await res.json()) as {
    enabled?: boolean
    running: boolean
    viewers: number
  }
  return {
    enabled: data.enabled !== false,
    running: data.running,
    viewers: data.viewers,
  }
}
