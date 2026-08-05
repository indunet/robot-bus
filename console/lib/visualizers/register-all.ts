'use client'

/** Side-effect: register core generic visualizers only. */

import { registerVisualizer } from '@/lib/visualizers/registry'
import RawJsonVisualizer from '@/components/dashboard/visualizers/RawJsonVisualizer'
import ScalarVisualizer from '@/components/dashboard/visualizers/ScalarVisualizer'
import StringVisualizer from '@/components/dashboard/visualizers/StringVisualizer'

let registered = false

export function ensureVisualizersRegistered(): void {
  if (registered) return
  registered = true

  registerVisualizer({
    id: 'raw',
    label: 'Raw JSON',
    types: ['*'],
    component: RawJsonVisualizer,
    priority: -100,
  })

  registerVisualizer({
    id: 'scalar',
    label: 'Scalar',
    types: [
      'sensor_msgs.msg.v1.Temperature',
      'sensor_msgs/msg/Temperature',
      'sensor_msgs.msg.v1.FluidPressure',
      'sensor_msgs/msg/FluidPressure',
      'sensor_msgs.msg.v1.Illuminance',
      'sensor_msgs/msg/Illuminance',
      'sensor_msgs.msg.v1.RelativeHumidity',
      'sensor_msgs/msg/RelativeHumidity',
      'sensor_msgs.msg.v1.Range',
      'sensor_msgs/msg/Range',
      'sensor_msgs.msg.v1.BatteryState',
      'sensor_msgs/msg/BatteryState',
      'std_msgs.msg.v1.Float32',
      'std_msgs/msg/Float32',
      'std_msgs.msg.v1.Float64',
      'std_msgs/msg/Float64',
      'std_msgs.msg.v1.Int32',
      'std_msgs/msg/Int32',
      'std_msgs.msg.v1.Int64',
      'std_msgs/msg/Int64',
      'std_msgs.msg.v1.UInt32',
      'std_msgs/msg/UInt32',
      'std_msgs.msg.v1.Bool',
      'std_msgs/msg/Bool',
    ],
    component: ScalarVisualizer,
    priority: 15,
  })

  registerVisualizer({
    id: 'string',
    label: 'String',
    types: ['std_msgs.msg.v1.String', 'std_msgs/msg/String'],
    component: StringVisualizer,
    priority: 15,
  })
}
