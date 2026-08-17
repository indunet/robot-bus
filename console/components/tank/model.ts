import { WORLD_SIZE } from '@/lib/tank'

export type Pose = { x: number; y: number; theta: number }
export type Point = Pick<Pose, 'x' | 'y'>
export type Direction = 'forward' | 'back' | 'left' | 'right'
export type Capability = 'keyboard' | 'point_nav' | 'multi_waypoint' | 'reset'
export type NavPhase = 'idle' | 'running' | 'done' | 'failed'

export type Overlay = {
  goals: Pose[]
  activeIndex: number
  /** Show heading cap for these goal indices (point goal / last multi waypoint). */
  headingIndices: number[]
}

export type GoalDrag = {
  mode: 'point' | 'multi'
  index: number
  originX: number
  originY: number
  dragged: boolean
}

export const MAX_TRAIL_POINTS = 4000
export const LINEAR_SPEED = 1.5
export const ANGULAR_SPEED = 1.8
export const TANK_SPRITE_SRC = '/bot-rover.svg'
export const TANK_SPRITE_SIZE_M = 0.88
/** Pose older than this ⇒ Rust `tank` treated as offline. */
export const SIM_STALE_MS = 1500

export const KEY_BUTTONS: { direction: Direction; label: string; className: string }[] = [
  { direction: 'forward', label: '↑', className: 'col-start-2 row-start-1' },
  { direction: 'left', label: '←', className: 'col-start-1 row-start-2' },
  { direction: 'back', label: '↓', className: 'col-start-2 row-start-2' },
  { direction: 'right', label: '→', className: 'col-start-3 row-start-2' },
]

export function headingIndicesFor(capability: Capability, goals: Pose[]): number[] {
  if (goals.length === 0) return []
  if (capability === 'point_nav') return [0]
  if (capability === 'multi_waypoint') return [goals.length - 1]
  return []
}

export function overlayFor(
  goals: Pose[],
  activeIndex: number,
  capability: Capability,
): Overlay {
  return {
    goals,
    activeIndex,
    headingIndices: headingIndicesFor(capability, goals),
  }
}

export function directionFromCode(code: string): Direction | null {
  if (code === 'ArrowUp') return 'forward'
  if (code === 'ArrowDown') return 'back'
  if (code === 'ArrowLeft') return 'left'
  if (code === 'ArrowRight') return 'right'
  return null
}

export function velocityFromPressed(pressed: Set<Direction>) {
  return {
    linear:
      (pressed.has('forward') ? LINEAR_SPEED : 0) - (pressed.has('back') ? LINEAR_SPEED : 0),
    angular:
      (pressed.has('left') ? ANGULAR_SPEED : 0) - (pressed.has('right') ? ANGULAR_SPEED : 0),
  }
}

export function pathLength(points: Point[]): number {
  let len = 0
  for (let i = 1; i < points.length; i += 1) {
    len += Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y)
  }
  return len
}

/** Live progress from current pose vs planned polyline (action feedback arrives at end). */
export function estimateNavProgress(current: Pose, start: Pose, goals: Pose[]): number {
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

export function activeGoalIndex(start: Pose, goals: Pose[], progress: number): number {
  if (goals.length <= 1) return 0
  const total = pathLength([start, ...goals]) || 1
  let covered = 0
  let idx = 0
  for (let i = 0; i < goals.length; i += 1) {
    const prev = i === 0 ? start : goals[i - 1]
    covered += Math.hypot(goals[i].x - prev.x, goals[i].y - prev.y)
    idx = i
    if (progress * total <= covered + 1e-6) break
  }
  return idx
}

export function clientToWorld(
  canvas: HTMLCanvasElement,
  clientX: number,
  clientY: number,
): Point | null {
  const rect = canvas.getBoundingClientRect()
  if (rect.width <= 0 || rect.height <= 0) return null
  const px = ((clientX - rect.left) / rect.width) * canvas.width
  const py = ((clientY - rect.top) / rect.height) * canvas.height
  return {
    x: Math.min(WORLD_SIZE, Math.max(0, (px / canvas.width) * WORLD_SIZE)),
    y: Math.min(WORLD_SIZE, Math.max(0, ((canvas.height - py) / canvas.height) * WORLD_SIZE)),
  }
}

export function appendTrail(trail: Point[], pose: Pose): Point[] {
  const last = trail[trail.length - 1]
  const jump = Math.hypot(pose.x - last.x, pose.y - last.y)
  if (jump < 0.02) return trail
  if (jump >= 2) return [{ x: pose.x, y: pose.y }]
  const next = trail.length >= MAX_TRAIL_POINTS ? trail.slice(-(MAX_TRAIL_POINTS - 1)) : trail
  next.push({ x: pose.x, y: pose.y })
  return next
}
