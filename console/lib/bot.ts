import { resolveBusUrl } from '@/lib/console-bus'

/** Built-in bot demo under the reserved `/robot_bus/*` namespace. */
export const BOT_PREFIX = '/robot_bus/bot'
export const CMD_VEL_TOPIC = '/robot_bus/bot/cmd_vel'
export const POSE_TOPIC = '/robot_bus/bot/pose'
export const POINT_NAV_ACTION = '/robot_bus/bot/point_navigation'
export const MULTI_WAYPOINT_NAV_ACTION = '/robot_bus/bot/multi_waypoint_navigation'
export const RESET_SERVICE = '/robot_bus/bot/reset'
/** World extents shared with the in-process `bot_sim` (meters / arbitrary units). */
export const WORLD_SIZE = 11
/** Home pose used by `/robot_bus/bot/reset` (world center). */
export const HOME_POSE: { x: number; y: number; theta: number } = {
  x: WORLD_SIZE / 2,
  y: WORLD_SIZE / 2,
  theta: 0,
}

export type BotSimSessionInfo = {
  sessionId: string
  leaseMs: number
  viewers: number
}

export type BotSimStatusInfo = {
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

/** Ask the broker to start / keep the shared bot_sim singleton alive. */
export async function acquireBotSimSession(): Promise<BotSimSessionInfo> {
  const res = await fetch(`${apiBase()}/api/v1/bot-sim/session`, { method: 'POST' })
  if (!res.ok) {
    const body = await res.text().catch(() => '')
    throw new Error(body || `acquire bot_sim session failed (${res.status})`)
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

export async function heartbeatBotSimSession(sessionId: string): Promise<BotSimSessionInfo> {
  const res = await fetch(`${apiBase()}/api/v1/bot-sim/session/${encodeURIComponent(sessionId)}/heartbeat`, {
    method: 'POST',
  })
  if (!res.ok) {
    throw new Error(`bot_sim heartbeat failed (${res.status})`)
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

export async function releaseBotSimSession(sessionId: string): Promise<void> {
  try {
    await fetch(`${apiBase()}/api/v1/bot-sim/session/${encodeURIComponent(sessionId)}`, {
      method: 'DELETE',
      keepalive: true,
    })
  } catch {
    /* ignore unload races */
  }
}

export async function fetchBotSimStatus(): Promise<BotSimStatusInfo> {
  const res = await fetch(`${apiBase()}/api/v1/bot-sim`)
  if (!res.ok) {
    throw new Error(`bot_sim status failed (${res.status})`)
  }
  const data = (await res.json()) as { running: boolean; viewers: number }
  return { running: data.running, viewers: data.viewers }
}
