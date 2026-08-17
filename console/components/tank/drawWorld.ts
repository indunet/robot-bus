import { WORLD_SIZE } from '@/lib/tank'
import { TANK_SPRITE_SIZE_M, type Overlay, type Point, type Pose } from './model'

function syncCanvasSize(canvas: HTMLCanvasElement) {
  const rect = canvas.getBoundingClientRect()
  const w = Math.max(1, Math.round(rect.width))
  const h = Math.max(1, Math.round(rect.height))
  if (canvas.width !== w) canvas.width = w
  if (canvas.height !== h) canvas.height = h
}

export function drawWorld(
  canvas: HTMLCanvasElement | null,
  pose: Pose,
  trail: Point[],
  sprite: HTMLImageElement | null,
  overlay: Overlay,
) {
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  syncCanvasSize(canvas)
  const { width, height } = canvas
  const scaleX = width / WORLD_SIZE
  const scaleY = height / WORLD_SIZE
  ctx.clearRect(0, 0, width, height)
  ctx.fillStyle = '#101214'
  ctx.fillRect(0, 0, width, height)
  ctx.strokeStyle = '#2a2f35'
  ctx.lineWidth = 1

  for (let i = 0; i <= WORLD_SIZE; i += 1) {
    const x = i * scaleX
    const y = height - i * scaleY
    ctx.beginPath()
    ctx.moveTo(x, 0)
    ctx.lineTo(x, height)
    ctx.stroke()
    ctx.beginPath()
    ctx.moveTo(0, y)
    ctx.lineTo(width, y)
    ctx.stroke()
  }

  if (trail.length > 1) {
    ctx.beginPath()
    ctx.moveTo(trail[0].x * scaleX, height - trail[0].y * scaleY)
    for (let i = 1; i < trail.length; i += 1) {
      ctx.lineTo(trail[i].x * scaleX, height - trail[i].y * scaleY)
    }
    ctx.strokeStyle = '#f59e0b'
    ctx.lineWidth = 2.5
    ctx.lineJoin = 'round'
    ctx.lineCap = 'round'
    ctx.stroke()
  }

  drawGoals(ctx, overlay, scaleX, scaleY, height)

  const x = pose.x * scaleX
  const y = height - pose.y * scaleY
  const size = TANK_SPRITE_SIZE_M * Math.min(scaleX, scaleY)
  ctx.save()
  ctx.translate(x, y)
  // Canvas Y is down; world Y is up — negate theta so heading matches motion.
  ctx.rotate(-pose.theta)
  if (sprite?.complete && sprite.naturalWidth > 0) {
    ctx.drawImage(sprite, -size / 2, -size / 2, size, size)
  } else {
    drawTankFallback(ctx, size)
  }
  ctx.restore()
}

function drawGoals(
  ctx: CanvasRenderingContext2D,
  overlay: Overlay,
  scaleX: number,
  scaleY: number,
  height: number,
) {
  if (overlay.goals.length === 0) return

  ctx.beginPath()
  const first = overlay.goals[0]
  ctx.moveTo(first.x * scaleX, height - first.y * scaleY)
  for (let i = 1; i < overlay.goals.length; i += 1) {
    ctx.lineTo(overlay.goals[i].x * scaleX, height - overlay.goals[i].y * scaleY)
  }
  ctx.strokeStyle = 'rgba(0, 183, 216, 0.55)'
  ctx.lineWidth = 1.5
  ctx.setLineDash([6, 4])
  ctx.stroke()
  ctx.setLineDash([])

  const headingSet = new Set(overlay.headingIndices)
  overlay.goals.forEach((g, i) => {
    const gx = g.x * scaleX
    const gy = height - g.y * scaleY
    const active = i === overlay.activeIndex
    const r = active ? 7 : 5
    const color = active ? '#f59e0b' : '#00b7d8'

    if (headingSet.has(i)) {
      const dirX = Math.cos(g.theta)
      const dirY = -Math.sin(g.theta)
      const nx = -dirY
      const ny = dirX
      const tipX = gx + dirX * (r + 8)
      const tipY = gy + dirY * (r + 8)
      const baseX = gx - dirX * r * 0.2
      const baseY = gy - dirY * r * 0.2
      const half = r * 0.98
      ctx.beginPath()
      ctx.moveTo(tipX, tipY)
      ctx.lineTo(baseX + nx * half, baseY + ny * half)
      ctx.lineTo(baseX - nx * half, baseY - ny * half)
      ctx.closePath()
      ctx.fillStyle = color
      ctx.fill()
    }

    ctx.beginPath()
    ctx.arc(gx, gy, r, 0, Math.PI * 2)
    ctx.fillStyle = color
    ctx.fill()
    if (overlay.goals.length > 1) {
      ctx.fillStyle = '#d8faff'
      ctx.font = '10px ui-monospace, monospace'
      ctx.fillText(String(i + 1), gx + 8, gy - 8)
    }
  })
}

/** Simple top-down rover used until the SVG sprite loads. */
function drawTankFallback(ctx: CanvasRenderingContext2D, size: number) {
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
