/** Plumbing node catalog for the Flow canvas (config-driven rbus_* + ros2_bridge). */

export type PortDirection = 'in' | 'out'

export type ParamFieldType = 'string' | 'number' | 'boolean'

export interface PortDef {
  /** Param key holding the topic name (e.g. input_topic). */
  id: string
  direction: PortDirection
  label: string
  optional?: boolean
}

export interface ParamFieldDef {
  key: string
  type: ParamFieldType
  label: string
  /** Hide from simple form when true (still exported). */
  advanced?: boolean
}

export interface NodeTypeDef {
  type: string
  label: string
  /** CLI binary name, or null for library-only (ros2_bridge). */
  binary: string | null
  defaultName: string
  ports: PortDef[]
  fields: ParamFieldDef[]
  defaultParams: Record<string, unknown>
}

export const FLOW_NODE_TYPES: NodeTypeDef[] = [
  {
    type: 'usb_camera',
    label: 'USB Camera',
    binary: 'rbus_usb_camera',
    defaultName: 'usb_camera',
    ports: [{ id: 'output_topic', direction: 'out', label: 'output' }],
    fields: [
      { key: 'output_topic', type: 'string', label: 'output_topic' },
      { key: 'device', type: 'string', label: 'device' },
      { key: 'width', type: 'number', label: 'width' },
      { key: 'height', type: 'number', label: 'height' },
      { key: 'fps', type: 'number', label: 'fps' },
      { key: 'frame_id', type: 'string', label: 'frame_id' },
    ],
    defaultParams: {
      output_topic: '/camera/image_raw',
      device: '',
      width: 640,
      height: 480,
      fps: 30,
      frame_id: 'camera',
    },
  },
  {
    type: 'image_encoder',
    label: 'Image Encoder',
    binary: 'rbus_image_encoder',
    defaultName: 'image_encoder',
    ports: [
      { id: 'input_topic', direction: 'in', label: 'input' },
      { id: 'output_topic', direction: 'out', label: 'output' },
    ],
    fields: [
      { key: 'input_topic', type: 'string', label: 'input_topic' },
      { key: 'output_topic', type: 'string', label: 'output_topic' },
      { key: 'codec', type: 'string', label: 'codec' },
      { key: 'bitrate', type: 'number', label: 'bitrate' },
      { key: 'gop_size', type: 'number', label: 'gop_size' },
      { key: 'fps', type: 'number', label: 'fps' },
      { key: 'encoder', type: 'string', label: 'encoder', advanced: true },
      { key: 'width', type: 'number', label: 'width', advanced: true },
      { key: 'height', type: 'number', label: 'height', advanced: true },
    ],
    defaultParams: {
      input_topic: '/camera/image_raw',
      output_topic: '/camera/video',
      codec: 'h264',
      bitrate: 2000000,
      gop_size: 30,
      fps: 30,
      encoder: '',
      width: 0,
      height: 0,
    },
  },
  {
    type: 'image_decoder',
    label: 'Image Decoder',
    binary: 'rbus_image_decoder',
    defaultName: 'image_decoder',
    ports: [
      { id: 'input_topic', direction: 'in', label: 'input' },
      { id: 'output_topic', direction: 'out', label: 'output' },
    ],
    fields: [
      { key: 'input_topic', type: 'string', label: 'input_topic' },
      { key: 'output_topic', type: 'string', label: 'output_topic' },
      { key: 'codec', type: 'string', label: 'codec' },
      { key: 'decoder', type: 'string', label: 'decoder', advanced: true },
      { key: 'output_encoding', type: 'string', label: 'output_encoding' },
    ],
    defaultParams: {
      input_topic: '/camera/video',
      output_topic: '/camera/image_decoded',
      codec: 'h264',
      decoder: '',
      output_encoding: 'rgb8',
    },
  },
  {
    type: 'webrtc',
    label: 'WebRTC (WHEP)',
    binary: 'rbus_webrtc',
    defaultName: 'webrtc',
    ports: [
      { id: 'image_topic', direction: 'in', label: 'image', optional: true },
      { id: 'audio_topic', direction: 'in', label: 'audio', optional: true },
    ],
    fields: [
      { key: 'image_topic', type: 'string', label: 'image_topic' },
      { key: 'audio_topic', type: 'string', label: 'audio_topic' },
      { key: 'data_topics', type: 'string', label: 'data_topics', advanced: true },
      { key: 'listen', type: 'string', label: 'listen' },
      { key: 'bitrate', type: 'number', label: 'bitrate' },
      { key: 'gop_size', type: 'number', label: 'gop_size', advanced: true },
      { key: 'fps', type: 'number', label: 'fps' },
      { key: 'encoder', type: 'string', label: 'encoder', advanced: true },
      { key: 'width', type: 'number', label: 'width', advanced: true },
      { key: 'height', type: 'number', label: 'height', advanced: true },
      { key: 'sample_rate', type: 'number', label: 'sample_rate', advanced: true },
      { key: 'channels', type: 'number', label: 'channels', advanced: true },
      { key: 'opus_bitrate', type: 'number', label: 'opus_bitrate', advanced: true },
    ],
    defaultParams: {
      image_topic: '/camera/image_raw',
      audio_topic: '/audio/mic',
      data_topics: '',
      listen: '0.0.0.0:8090',
      bitrate: 2000000,
      gop_size: 30,
      fps: 30,
      encoder: '',
      width: 0,
      height: 0,
      sample_rate: 16000,
      channels: 1,
      opus_bitrate: 32000,
    },
  },
  {
    type: 'ros2_bridge',
    label: 'ROS 2 Bridge',
    binary: null,
    defaultName: 'ros2_bridge',
    /** Dynamic ports derived from bridge routes at runtime. */
    ports: [],
    fields: [],
    defaultParams: {
      routes: [],
      services: [],
      actions: [],
    },
  },
]

const byType = new Map(FLOW_NODE_TYPES.map((t) => [t.type, t]))

export function getNodeType(type: string): NodeTypeDef | undefined {
  return byType.get(type)
}

export function isPlumbingType(type: string): boolean {
  return byType.has(type)
}

/** Topic string from node params for a fixed port id. */
export function topicFromParams(
  params: Record<string, unknown>,
  portId: string,
): string {
  const v = params[portId]
  return typeof v === 'string' ? v.trim() : ''
}

export function shortTopic(topic: string): string {
  if (topic.length <= 28) return topic
  const parts = topic.split('/').filter(Boolean)
  if (parts.length <= 1) return topic.slice(0, 26) + '…'
  return `…/${parts[parts.length - 1]}`
}
