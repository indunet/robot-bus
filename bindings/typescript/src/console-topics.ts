/** Fixed system topic names for broker console introspection (mirror Rust `console_topics`). */

export const PREFIX = "/robot_bus";

export const STATUS = "/robot_bus/status";
export const TOPICS = "/robot_bus/topics";
export const SERVICES = "/robot_bus/services";
export const ACTIONS = "/robot_bus/actions";
export const TOPOLOGY = "/robot_bus/topology";
export const EVENTS = "/robot_bus/events";

export const TOPOLOGY_REGISTER = "/robot_bus/topology/register";
export const TOPOLOGY_UNREGISTER = "/robot_bus/topology/unregister";
export const TOPIC_TYPE_REGISTER = "/robot_bus/topic_type/register";

export const SNAPSHOT_PUBLISH = [STATUS, TOPICS, SERVICES, ACTIONS, TOPOLOGY, EVENTS] as const;

export const CONTROL_SUBSCRIBE = [
  TOPOLOGY_REGISTER,
  TOPOLOGY_UNREGISTER,
  TOPIC_TYPE_REGISTER,
] as const;

/** True for names under the reserved console namespace (`/robot_bus` and `/robot_bus/*`). */
export function isReservedName(name: string): boolean {
  const trimmed = name.trim();
  return trimmed === PREFIX || trimmed.startsWith("/robot_bus/");
}
