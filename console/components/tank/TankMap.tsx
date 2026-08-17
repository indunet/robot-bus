'use client'

import type { PointerEvent as ReactPointerEvent, RefObject } from 'react'
import {
  CMD_VEL_TOPIC,
  MULTI_WAYPOINT_NAV_ACTION,
  POINT_NAV_ACTION,
  POSE_TOPIC,
  RESET_SERVICE,
} from '@/lib/tank'
import { useI18n } from '@/lib/i18n'
import type { Capability, Pose } from './model'

interface Props {
  canvasRef: RefObject<HTMLCanvasElement | null>
  compact: boolean
  capability: Capability
  pose: Pose
  onPointerDown: (event: ReactPointerEvent<HTMLCanvasElement>) => void
  onPointerMove: (event: ReactPointerEvent<HTMLCanvasElement>) => void
  onPointerUp: (event: ReactPointerEvent<HTMLCanvasElement>) => void
}

export default function TankMap({
  canvasRef,
  compact,
  capability,
  pose,
  onPointerDown,
  onPointerMove,
  onPointerUp,
}: Props) {
  const { t } = useI18n()
  const placing = capability === 'point_nav' || capability === 'multi_waypoint'

  return (
    <section className="min-w-0 min-h-0">
      <div className="relative h-full w-full">
        <canvas
          ref={canvasRef}
          width={640}
          height={640}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
          className={`absolute inset-0 h-full w-full rounded-sm border border-bus-border touch-none ${
            placing ? 'cursor-crosshair' : 'cursor-default'
          } ${compact ? 'bg-[#121518]/80' : 'bg-[#101214]'}`}
        />
        <div className="pointer-events-none absolute inset-0 font-mono text-[9px] leading-4">
          <div className="absolute top-2 left-2 max-w-[90%] space-y-0.5 break-all text-bus-text drop-shadow-[0_1px_2px_rgb(0_0_0_/80%)]">
            <div>
              {t('tankSubscribe')} {POSE_TOPIC}
            </div>
            <div>
              {t('tankPublish')} {CMD_VEL_TOPIC}
            </div>
            <div>SRV {RESET_SERVICE}</div>
            <div>ACT {POINT_NAV_ACTION}</div>
            <div>ACT {MULTI_WAYPOINT_NAV_ACTION}</div>
          </div>
          <div className="absolute bottom-2 right-2 whitespace-nowrap text-right text-bus-cyan drop-shadow-[0_1px_2px_rgb(0_0_0_/80%)]">
            {t('tankPoseTitle')}  x {pose.x.toFixed(3)}  y {pose.y.toFixed(3)}  θ {pose.theta.toFixed(3)}
          </div>
        </div>
      </div>
    </section>
  )
}
