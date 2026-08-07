import { fetchStatus } from '@/lib/mock-data'

export const CMD_VEL_TOPIC = '/bot1/cmd_vel'
export const POSE_TOPIC = '/bot1/pose'

const DEFAULT_GRPC_URL = 'http://127.0.0.1:15770'

export async function resolveGrpcUrl(): Promise<string> {
  try {
    const { grpcAddr } = await fetchStatus()
    if (!grpcAddr || grpcAddr === '—') return DEFAULT_GRPC_URL
    if (/^https?:\/\//.test(grpcAddr)) return grpcAddr.replace(/\/$/, '')

    const parsed = new URL(`http://${grpcAddr}`)
    const host =
      parsed.hostname === '0.0.0.0' || parsed.hostname === '::'
        ? window.location.hostname
        : parsed.hostname
    return `${window.location.protocol}//${host}:${parsed.port || '15770'}`
  } catch {
    return `${window.location.protocol}//${window.location.hostname}:15770`
  }
}
