import { resolveBusUrl } from '@/lib/console-bus'

export type ConsoleUiFlags = {
  /** False when broker started with `--no-tank`. Default true. */
  tankEnabled: boolean
  /** False when broker started with `--no-docs`. Default true. */
  docsEnabled: boolean
}

/** Sidebar feature flags from `GET /api/v1/console`. */
export async function fetchConsoleUiFlags(): Promise<ConsoleUiFlags> {
  const res = await fetch(`${resolveBusUrl()}/api/v1/console`)
  if (!res.ok) {
    throw new Error(`console ui flags failed (${res.status})`)
  }
  return (await res.json()) as ConsoleUiFlags
}
