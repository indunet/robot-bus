/**
 * Map registered topic type names → protobuf-ts MessageType for decode.
 */

import type { MessageType } from 'robot-bus'
import { Imu } from 'robot-bus/sensor_msgs/msg/v1/imu.js'
import { Image } from 'robot-bus/sensor_msgs/msg/v1/image.js'
import { CompressedImage } from 'robot-bus/sensor_msgs/msg/v1/compressed_image.js'
import { LaserScan } from 'robot-bus/sensor_msgs/msg/v1/laser_scan.js'
import { Temperature } from 'robot-bus/sensor_msgs/msg/v1/temperature.js'
import { FluidPressure } from 'robot-bus/sensor_msgs/msg/v1/fluid_pressure.js'
import { Illuminance } from 'robot-bus/sensor_msgs/msg/v1/illuminance.js'
import { RelativeHumidity } from 'robot-bus/sensor_msgs/msg/v1/relative_humidity.js'
import { Range } from 'robot-bus/sensor_msgs/msg/v1/range.js'
import { BatteryState } from 'robot-bus/sensor_msgs/msg/v1/battery_state.js'
import { JointState } from 'robot-bus/sensor_msgs/msg/v1/joint_state.js'
import { NavSatFix } from 'robot-bus/sensor_msgs/msg/v1/nav_sat_fix.js'
import { CameraInfo } from 'robot-bus/sensor_msgs/msg/v1/camera_info.js'
import {
  Bool,
  Float32,
  Float64,
  Int32,
  Int64,
  String$,
  UInt32,
  UInt64,
} from 'robot-bus/std_msgs/msg/v1/primitives.js'
import { Header } from 'robot-bus/std_msgs/msg/v1/header.js'
import { Pose } from 'robot-bus/geometry_msgs/msg/v1/pose.js'
import { Twist } from 'robot-bus/geometry_msgs/msg/v1/twist.js'
import { PoseStamped, TwistStamped } from 'robot-bus/geometry_msgs/msg/v1/stamped.js'
import { Odometry as NavOdometry } from 'robot-bus/nav_msgs/msg/v1/odometry.js'
import { Path } from 'robot-bus/nav_msgs/msg/v1/path.js'
import { OccupancyGrid } from 'robot-bus/nav_msgs/msg/v1/occupancy_grid.js'
import { TFMessage } from 'robot-bus/tf2_msgs/msg/v1/tf_message.js'
import { CompressedVideo } from 'robot-bus/foxglove_msgs/msg/v1/compressed_video.js'
import { CompressedImage as FoxCompressedImage } from 'robot-bus/foxglove_msgs/msg/v1/compressed_image.js'
import { RawImage } from 'robot-bus/foxglove_msgs/msg/v1/raw_image.js'
import { LaserScan as FoxLaserScan } from 'robot-bus/foxglove_msgs/msg/v1/laser_scan.js'
import { Odometry as FoxOdometry } from 'robot-bus/foxglove_msgs/msg/v1/odometry.js'
import { PoseInFrame } from 'robot-bus/foxglove_msgs/msg/v1/pose_in_frame.js'
import { FrameTransforms } from 'robot-bus/foxglove_msgs/msg/v1/frame_transforms.js'
import { CameraCalibration } from 'robot-bus/foxglove_msgs/msg/v1/camera_calibration.js'

type Entry = {
  msgType: MessageType<object>
  /** Canonical protobuf full name */
  fullName: string
  aliases: string[]
}

const ENTRIES: Entry[] = [
  { msgType: Imu as MessageType<object>, fullName: Imu.typeName, aliases: ['sensor_msgs/msg/Imu', 'sensor_msgs/Imu'] },
  { msgType: Image as MessageType<object>, fullName: Image.typeName, aliases: ['sensor_msgs/msg/Image', 'sensor_msgs/Image'] },
  {
    msgType: CompressedImage as MessageType<object>,
    fullName: CompressedImage.typeName,
    aliases: ['sensor_msgs/msg/CompressedImage', 'sensor_msgs/CompressedImage'],
  },
  {
    msgType: LaserScan as MessageType<object>,
    fullName: LaserScan.typeName,
    aliases: ['sensor_msgs/msg/LaserScan', 'sensor_msgs/LaserScan'],
  },
  {
    msgType: Temperature as MessageType<object>,
    fullName: Temperature.typeName,
    aliases: ['sensor_msgs/msg/Temperature'],
  },
  {
    msgType: FluidPressure as MessageType<object>,
    fullName: FluidPressure.typeName,
    aliases: ['sensor_msgs/msg/FluidPressure'],
  },
  {
    msgType: Illuminance as MessageType<object>,
    fullName: Illuminance.typeName,
    aliases: ['sensor_msgs/msg/Illuminance'],
  },
  {
    msgType: RelativeHumidity as MessageType<object>,
    fullName: RelativeHumidity.typeName,
    aliases: ['sensor_msgs/msg/RelativeHumidity'],
  },
  { msgType: Range as MessageType<object>, fullName: Range.typeName, aliases: ['sensor_msgs/msg/Range'] },
  {
    msgType: BatteryState as MessageType<object>,
    fullName: BatteryState.typeName,
    aliases: ['sensor_msgs/msg/BatteryState'],
  },
  {
    msgType: JointState as MessageType<object>,
    fullName: JointState.typeName,
    aliases: ['sensor_msgs/msg/JointState'],
  },
  {
    msgType: NavSatFix as MessageType<object>,
    fullName: NavSatFix.typeName,
    aliases: ['sensor_msgs/msg/NavSatFix'],
  },
  {
    msgType: CameraInfo as MessageType<object>,
    fullName: CameraInfo.typeName,
    aliases: ['sensor_msgs/msg/CameraInfo'],
  },
  { msgType: Bool as MessageType<object>, fullName: Bool.typeName, aliases: ['std_msgs/msg/Bool'] },
  { msgType: Float32 as MessageType<object>, fullName: Float32.typeName, aliases: ['std_msgs/msg/Float32'] },
  { msgType: Float64 as MessageType<object>, fullName: Float64.typeName, aliases: ['std_msgs/msg/Float64'] },
  { msgType: Int32 as MessageType<object>, fullName: Int32.typeName, aliases: ['std_msgs/msg/Int32'] },
  { msgType: Int64 as MessageType<object>, fullName: Int64.typeName, aliases: ['std_msgs/msg/Int64'] },
  { msgType: UInt32 as MessageType<object>, fullName: UInt32.typeName, aliases: ['std_msgs/msg/UInt32'] },
  { msgType: UInt64 as MessageType<object>, fullName: UInt64.typeName, aliases: ['std_msgs/msg/UInt64'] },
  {
    msgType: String$ as MessageType<object>,
    fullName: String$.typeName,
    aliases: ['std_msgs/msg/String', 'std_msgs/String'],
  },
  { msgType: Header as MessageType<object>, fullName: Header.typeName, aliases: ['std_msgs/msg/Header'] },
  { msgType: Pose as MessageType<object>, fullName: Pose.typeName, aliases: ['geometry_msgs/msg/Pose'] },
  { msgType: Twist as MessageType<object>, fullName: Twist.typeName, aliases: ['geometry_msgs/msg/Twist'] },
  {
    msgType: PoseStamped as MessageType<object>,
    fullName: PoseStamped.typeName,
    aliases: ['geometry_msgs/msg/PoseStamped'],
  },
  {
    msgType: TwistStamped as MessageType<object>,
    fullName: TwistStamped.typeName,
    aliases: ['geometry_msgs/msg/TwistStamped'],
  },
  {
    msgType: NavOdometry as MessageType<object>,
    fullName: NavOdometry.typeName,
    aliases: ['nav_msgs/msg/Odometry'],
  },
  { msgType: Path as MessageType<object>, fullName: Path.typeName, aliases: ['nav_msgs/msg/Path'] },
  {
    msgType: OccupancyGrid as MessageType<object>,
    fullName: OccupancyGrid.typeName,
    aliases: ['nav_msgs/msg/OccupancyGrid'],
  },
  {
    msgType: TFMessage as MessageType<object>,
    fullName: TFMessage.typeName,
    aliases: ['tf2_msgs/msg/TFMessage', 'tf2_msgs/TFMessage'],
  },
  {
    msgType: CompressedVideo as MessageType<object>,
    fullName: CompressedVideo.typeName,
    aliases: ['foxglove_msgs/msg/CompressedVideo', 'foxglove_msgs/CompressedVideo'],
  },
  {
    msgType: FoxCompressedImage as MessageType<object>,
    fullName: FoxCompressedImage.typeName,
    aliases: ['foxglove_msgs/msg/CompressedImage'],
  },
  {
    msgType: RawImage as MessageType<object>,
    fullName: RawImage.typeName,
    aliases: ['foxglove_msgs/msg/RawImage'],
  },
  {
    msgType: FoxLaserScan as MessageType<object>,
    fullName: FoxLaserScan.typeName,
    aliases: ['foxglove_msgs/msg/LaserScan'],
  },
  {
    msgType: FoxOdometry as MessageType<object>,
    fullName: FoxOdometry.typeName,
    aliases: ['foxglove_msgs/msg/Odometry'],
  },
  {
    msgType: PoseInFrame as MessageType<object>,
    fullName: PoseInFrame.typeName,
    aliases: ['foxglove_msgs/msg/PoseInFrame'],
  },
  {
    msgType: FrameTransforms as MessageType<object>,
    fullName: FrameTransforms.typeName,
    aliases: ['foxglove_msgs/msg/FrameTransforms'],
  },
  {
    msgType: CameraCalibration as MessageType<object>,
    fullName: CameraCalibration.typeName,
    aliases: ['foxglove_msgs/msg/CameraCalibration'],
  },
]

const BY_NAME = new Map<string, MessageType<object>>()

function index(name: string, msgType: MessageType<object>) {
  BY_NAME.set(name, msgType)
  BY_NAME.set(name.toLowerCase(), msgType)
}

for (const e of ENTRIES) {
  index(e.fullName, e.msgType)
  for (const a of e.aliases) index(a, e.msgType)
}

export function resolveMsgType(typeName: string | undefined | null): MessageType<object> | undefined {
  if (!typeName) return undefined
  return BY_NAME.get(typeName) ?? BY_NAME.get(typeName.toLowerCase())
}

export function knownTypeNames(): string[] {
  return ENTRIES.map((e) => e.fullName)
}
