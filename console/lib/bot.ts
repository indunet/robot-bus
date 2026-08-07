import { resolveBusUrl } from '@/lib/console-bus'

export const CMD_VEL_TOPIC = '/bot1/cmd_vel'
export const POSE_TOPIC = '/bot1/pose'

/** Same-origin gRPC-Web (single-port console + gateway). */
export async function resolveGrpcUrl(): Promise<string> {
  return resolveBusUrl()
}
