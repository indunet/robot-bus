'use client'

import Pose2DVisualizer from './Pose2DVisualizer'
import TwistVisualizer from './TwistVisualizer'
import { asRecord } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

/** Odometry: pose trail + twist charts. */
export default function OdometryVisualizer(props: VisualizerProps) {
  const rec = asRecord(props.message)
  const twistMsg = rec?.twist ? { twist: rec.twist } : props.message

  return (
    <div className="h-full grid grid-cols-1 md:grid-cols-2 gap-1 min-h-0 p-1">
      <div className="min-h-0 border border-bus-border/60 rounded-sm overflow-hidden">
        <Pose2DVisualizer {...props} />
      </div>
      <div className="min-h-0 border border-bus-border/60 rounded-sm overflow-hidden">
        <TwistVisualizer message={twistMsg} raw={null} topic={props.topic} typeName={props.typeName} />
      </div>
    </div>
  )
}
