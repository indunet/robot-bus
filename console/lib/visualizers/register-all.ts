'use client'

/** Side-effect: register all built-in visualizers. */

import { registerVisualizer } from '@/lib/visualizers/registry'
import RawJsonVisualizer from '@/components/dashboard/visualizers/RawJsonVisualizer'
import CompressedImageVisualizer from '@/components/dashboard/visualizers/CompressedImageVisualizer'
import RawImageVisualizer from '@/components/dashboard/visualizers/RawImageVisualizer'
import ImuVisualizer from '@/components/dashboard/visualizers/ImuVisualizer'
import ScalarVisualizer from '@/components/dashboard/visualizers/ScalarVisualizer'
import StringVisualizer from '@/components/dashboard/visualizers/StringVisualizer'
import CompressedVideoVisualizer from '@/components/dashboard/visualizers/CompressedVideoVisualizer'
import LaserScanVisualizer from '@/components/dashboard/visualizers/LaserScanVisualizer'
import TwistVisualizer from '@/components/dashboard/visualizers/TwistVisualizer'
import Pose2DVisualizer from '@/components/dashboard/visualizers/Pose2DVisualizer'
import OdometryVisualizer from '@/components/dashboard/visualizers/OdometryVisualizer'
import JointStateVisualizer from '@/components/dashboard/visualizers/JointStateVisualizer'
import JoyVisualizer from '@/components/dashboard/visualizers/JoyVisualizer'
import NavSatFixVisualizer from '@/components/dashboard/visualizers/NavSatFixVisualizer'
import TfSummaryVisualizer from '@/components/dashboard/visualizers/TfSummaryVisualizer'
import CameraInfoVisualizer from '@/components/dashboard/visualizers/CameraInfoVisualizer'
import OccupancyGridVisualizer from '@/components/dashboard/visualizers/OccupancyGridVisualizer'

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
    id: 'compressed-image',
    label: 'Compressed Image',
    types: [
      'sensor_msgs.msg.v1.CompressedImage',
      'sensor_msgs/msg/CompressedImage',
      'foxglove_msgs.msg.v1.CompressedImage',
      'foxglove_msgs/msg/CompressedImage',
    ],
    component: CompressedImageVisualizer,
    priority: 20,
  })

  registerVisualizer({
    id: 'raw-image',
    label: 'Raw Image',
    types: [
      'sensor_msgs.msg.v1.Image',
      'sensor_msgs/msg/Image',
      'foxglove_msgs.msg.v1.RawImage',
      'foxglove_msgs/msg/RawImage',
    ],
    component: RawImageVisualizer,
    priority: 20,
  })

  registerVisualizer({
    id: 'imu',
    label: 'IMU',
    types: ['sensor_msgs.msg.v1.Imu', 'sensor_msgs/msg/Imu'],
    component: ImuVisualizer,
    priority: 20,
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

  registerVisualizer({
    id: 'compressed-video',
    label: 'Compressed Video',
    types: [
      'foxglove_msgs.msg.v1.CompressedVideo',
      'foxglove_msgs/msg/CompressedVideo',
    ],
    component: CompressedVideoVisualizer,
    priority: 20,
  })

  registerVisualizer({
    id: 'laser-scan',
    label: 'Laser Scan',
    types: [
      'sensor_msgs.msg.v1.LaserScan',
      'sensor_msgs/msg/LaserScan',
      'foxglove_msgs.msg.v1.LaserScan',
      'foxglove_msgs/msg/LaserScan',
    ],
    component: LaserScanVisualizer,
    priority: 20,
  })

  registerVisualizer({
    id: 'twist',
    label: 'Twist',
    types: [
      'geometry_msgs.msg.v1.Twist',
      'geometry_msgs/msg/Twist',
      'geometry_msgs.msg.v1.TwistStamped',
      'geometry_msgs/msg/TwistStamped',
    ],
    component: TwistVisualizer,
    priority: 15,
  })

  registerVisualizer({
    id: 'pose-2d',
    label: 'Pose 2D',
    types: [
      'geometry_msgs.msg.v1.Pose',
      'geometry_msgs/msg/Pose',
      'geometry_msgs.msg.v1.PoseStamped',
      'geometry_msgs/msg/PoseStamped',
      'foxglove_msgs.msg.v1.PoseInFrame',
      'foxglove_msgs/msg/PoseInFrame',
    ],
    component: Pose2DVisualizer,
    priority: 15,
  })

  registerVisualizer({
    id: 'odometry',
    label: 'Odometry',
    types: [
      'nav_msgs.msg.v1.Odometry',
      'nav_msgs/msg/Odometry',
      'foxglove_msgs.msg.v1.Odometry',
      'foxglove_msgs/msg/Odometry',
    ],
    component: OdometryVisualizer,
    priority: 20,
  })

  registerVisualizer({
    id: 'joint-state',
    label: 'Joint State',
    types: ['sensor_msgs.msg.v1.JointState', 'sensor_msgs/msg/JointState'],
    component: JointStateVisualizer,
    priority: 15,
  })

  registerVisualizer({
    id: 'joy',
    label: 'Joy',
    types: ['sensor_msgs.msg.v1.Joy', 'sensor_msgs/msg/Joy'],
    component: JoyVisualizer,
    priority: 15,
  })

  registerVisualizer({
    id: 'navsat',
    label: 'NavSat Fix',
    types: ['sensor_msgs.msg.v1.NavSatFix', 'sensor_msgs/msg/NavSatFix'],
    component: NavSatFixVisualizer,
    priority: 15,
  })

  registerVisualizer({
    id: 'tf',
    label: 'TF Summary',
    types: [
      'tf2_msgs.msg.v1.TFMessage',
      'tf2_msgs/msg/TFMessage',
      'foxglove_msgs.msg.v1.FrameTransforms',
      'foxglove_msgs/msg/FrameTransforms',
    ],
    component: TfSummaryVisualizer,
    priority: 15,
  })

  registerVisualizer({
    id: 'camera-info',
    label: 'Camera Info',
    types: [
      'sensor_msgs.msg.v1.CameraInfo',
      'sensor_msgs/msg/CameraInfo',
      'foxglove_msgs.msg.v1.CameraCalibration',
      'foxglove_msgs/msg/CameraCalibration',
    ],
    component: CameraInfoVisualizer,
    priority: 10,
  })

  registerVisualizer({
    id: 'occupancy-grid',
    label: 'Occupancy Grid',
    types: ['nav_msgs.msg.v1.OccupancyGrid', 'nav_msgs/msg/OccupancyGrid'],
    component: OccupancyGridVisualizer,
    priority: 15,
  })
}
