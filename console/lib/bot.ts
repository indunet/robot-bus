import { resolveBusUrl } from '@/lib/console-bus'

export const CMD_VEL_TOPIC = '/bot1/cmd_vel'
export const POSE_TOPIC = '/bot1/pose'
/** World extents shared with the Rust `bot_sim` example (meters / arbitrary units). */
export const WORLD_SIZE = 11

/** Same-origin gRPC-Web (single-port console + gateway). */
export async function resolveGrpcUrl(): Promise<string> {
  return resolveBusUrl()
}
