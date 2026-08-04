/** Ros2Bridge YAML schema helpers (used by Flow `ros2_bridge` nodes; mirrors src/ros2/yaml.rs). */

import { parse as parseYaml, stringify as stringifyYaml } from 'yaml'

export const TOPIC_TYPES = [
  'std_msgs/msg/String',
  'sensor_msgs/msg/Imu',
  'sensor_msgs/msg/Image',
  'foxglove_msgs/msg/CompressedVideo',
] as const

export const SERVICE_TYPES = ['std_srvs/srv/Trigger', 'std_srvs/srv/SetBool'] as const

export const ACTION_TYPES = ['example_interfaces/action/Fibonacci'] as const

export const TOPIC_DIRECTIONS = ['both', 'ros_to_bus', 'bus_to_ros'] as const
export const SRV_ACT_DIRECTIONS = ['ros_to_bus', 'bus_to_ros'] as const

export type TopicDirection = (typeof TOPIC_DIRECTIONS)[number]
export type SrvActDirection = (typeof SRV_ACT_DIRECTIONS)[number]

export interface RobotBusSection {
  transport: string
  host?: string
  ipc_path?: string
  discover?: {
    domain_id?: number
    timeout?: number
    broker_id?: string
  }
}

export interface TopicRoute {
  id: string
  ros_topic: string
  bus_topic: string
  type: string
  direction: TopicDirection
}

export interface ServiceRoute {
  id: string
  ros_service: string
  bus_service: string
  type: string
  direction: SrvActDirection
}

export interface ActionRoute {
  id: string
  ros_action: string
  bus_action: string
  type: string
  direction: SrvActDirection
}

export interface BridgeConfig {
  robot_bus: RobotBusSection
  routes: TopicRoute[]
  services: ServiceRoute[]
  actions: ActionRoute[]
}

export const CAMERA_H264_EXAMPLE = `# Example: ROS 2 camera Image → robot-bus → H.264 CompressedVideo → ROS 2.
robot_bus:
  transport: tcp
  host: localhost

routes:
  - ros_topic: /camera/image_raw
    bus_topic: /camera/image_raw
    type: sensor_msgs/msg/Image
    direction: ros_to_bus
  - ros_topic: /camera/video
    bus_topic: /camera/video
    type: foxglove_msgs/msg/CompressedVideo
    direction: bus_to_ros
`

function newId(prefix: string): string {
  return `${prefix}-${Math.random().toString(36).slice(2, 10)}`
}

export function emptyConfig(): BridgeConfig {
  return {
    robot_bus: { transport: 'tcp', host: 'localhost' },
    routes: [],
    services: [],
    actions: [],
  }
}

export function validateConfig(cfg: BridgeConfig): string[] {
  const errors: string[] = []
  if (!cfg.routes.length && !cfg.services.length && !cfg.actions.length) {
    errors.push('Need at least one route, service, or action')
  }
  for (const r of cfg.routes) {
    if (!r.ros_topic.trim() || !r.bus_topic.trim()) {
      errors.push(`Route ${r.id}: ros_topic and bus_topic required`)
    }
    if (!(TOPIC_TYPES as readonly string[]).includes(r.type)) {
      errors.push(`Route ${r.id}: unsupported type ${r.type}`)
    }
    if (!(TOPIC_DIRECTIONS as readonly string[]).includes(r.direction)) {
      errors.push(`Route ${r.id}: invalid direction ${r.direction}`)
    }
  }
  for (const s of cfg.services) {
    if (!s.ros_service.trim() || !s.bus_service.trim()) {
      errors.push(`Service ${s.id}: ros_service and bus_service required`)
    }
    if (!(SERVICE_TYPES as readonly string[]).includes(s.type)) {
      errors.push(`Service ${s.id}: unsupported type ${s.type}`)
    }
    if (!(SRV_ACT_DIRECTIONS as readonly string[]).includes(s.direction)) {
      errors.push(`Service ${s.id}: direction must be ros_to_bus or bus_to_ros`)
    }
  }
  for (const a of cfg.actions) {
    if (!a.ros_action.trim() || !a.bus_action.trim()) {
      errors.push(`Action ${a.id}: ros_action and bus_action required`)
    }
    if (!(ACTION_TYPES as readonly string[]).includes(a.type)) {
      errors.push(`Action ${a.id}: unsupported type ${a.type}`)
    }
    if (!(SRV_ACT_DIRECTIONS as readonly string[]).includes(a.direction)) {
      errors.push(`Action ${a.id}: direction must be ros_to_bus or bus_to_ros`)
    }
  }
  return errors
}

export function parseBridgeYaml(text: string): { config: BridgeConfig; errors: string[] } {
  try {
    const raw = parseYaml(text) as Record<string, unknown> | null
    if (!raw || typeof raw !== 'object') {
      return { config: emptyConfig(), errors: ['YAML root must be a mapping'] }
    }
    const rb = (raw.robot_bus as Record<string, unknown> | undefined) ?? {}
    const routesRaw = Array.isArray(raw.routes) ? raw.routes : []
    const servicesRaw = Array.isArray(raw.services) ? raw.services : []
    const actionsRaw = Array.isArray(raw.actions) ? raw.actions : []

    const config: BridgeConfig = {
      robot_bus: {
        transport: String(rb.transport ?? 'tcp'),
        host: rb.host !== undefined ? String(rb.host) : 'localhost',
        ipc_path: rb.ipc_path !== undefined ? String(rb.ipc_path) : undefined,
        discover: rb.discover as RobotBusSection['discover'],
      },
      routes: routesRaw.map((r: Record<string, unknown>) => ({
        id: newId('route'),
        ros_topic: String(r.ros_topic ?? ''),
        bus_topic: String(r.bus_topic ?? ''),
        type: String(r.type ?? TOPIC_TYPES[0]),
        direction: (String(r.direction ?? 'both') as TopicDirection),
      })),
      services: servicesRaw.map((s: Record<string, unknown>) => ({
        id: newId('svc'),
        ros_service: String(s.ros_service ?? ''),
        bus_service: String(s.bus_service ?? ''),
        type: String(s.type ?? SERVICE_TYPES[0]),
        direction: (String(s.direction ?? 'ros_to_bus') as SrvActDirection),
      })),
      actions: actionsRaw.map((a: Record<string, unknown>) => ({
        id: newId('act'),
        ros_action: String(a.ros_action ?? ''),
        bus_action: String(a.bus_action ?? ''),
        type: String(a.type ?? ACTION_TYPES[0]),
        direction: (String(a.direction ?? 'ros_to_bus') as SrvActDirection),
      })),
    }
    return { config, errors: validateConfig(config) }
  } catch (err) {
    return {
      config: emptyConfig(),
      errors: [err instanceof Error ? err.message : 'Failed to parse YAML'],
    }
  }
}

export function exportBridgeYaml(cfg: BridgeConfig): string {
  const doc: Record<string, unknown> = {
    robot_bus: {
      transport: cfg.robot_bus.transport,
      ...(cfg.robot_bus.host !== undefined ? { host: cfg.robot_bus.host } : {}),
      ...(cfg.robot_bus.ipc_path ? { ipc_path: cfg.robot_bus.ipc_path } : {}),
      ...(cfg.robot_bus.discover ? { discover: cfg.robot_bus.discover } : {}),
    },
  }
  if (cfg.routes.length) {
    doc.routes = cfg.routes.map(({ ros_topic, bus_topic, type, direction }) => ({
      ros_topic,
      bus_topic,
      type,
      direction,
    }))
  }
  if (cfg.services.length) {
    doc.services = cfg.services.map(({ ros_service, bus_service, type, direction }) => ({
      ros_service,
      bus_service,
      type,
      direction,
    }))
  }
  if (cfg.actions.length) {
    doc.actions = cfg.actions.map(({ ros_action, bus_action, type, direction }) => ({
      ros_action,
      bus_action,
      type,
      direction,
    }))
  }
  return stringifyYaml(doc)
}

export function addTopicRoute(cfg: BridgeConfig): BridgeConfig {
  return {
    ...cfg,
    routes: [
      ...cfg.routes,
      {
        id: newId('route'),
        ros_topic: '/chatter',
        bus_topic: '/chatter',
        type: 'std_msgs/msg/String',
        direction: 'both',
      },
    ],
  }
}

export function addServiceRoute(cfg: BridgeConfig): BridgeConfig {
  return {
    ...cfg,
    services: [
      ...cfg.services,
      {
        id: newId('svc'),
        ros_service: '/trigger',
        bus_service: '/trigger',
        type: 'std_srvs/srv/Trigger',
        direction: 'ros_to_bus',
      },
    ],
  }
}

export function addActionRoute(cfg: BridgeConfig): BridgeConfig {
  return {
    ...cfg,
    actions: [
      ...cfg.actions,
      {
        id: newId('act'),
        ros_action: '/fibonacci',
        bus_action: '/fibonacci',
        type: 'example_interfaces/action/Fibonacci',
        direction: 'ros_to_bus',
      },
    ],
  }
}
