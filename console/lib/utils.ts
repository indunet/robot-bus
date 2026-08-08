import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** Fixed `HH:MM:SS` clock label (locale-independent); used in chart tooltips. */
export function formatHms(date: Date = new Date()): string {
  const hh = String(date.getHours()).padStart(2, '0')
  const mm = String(date.getMinutes()).padStart(2, '0')
  const ss = String(date.getSeconds()).padStart(2, '0')
  return `${hh}:${mm}:${ss}`
}

/** Axis tick from an `HH:MM:SS` (or already `MM:SS`) label → `MM:SS`. */
export function formatMsTick(label: string): string {
  const parts = String(label).split(':')
  if (parts.length >= 3) return `${parts[1]}:${parts[2]}`
  if (parts.length === 2) return `${parts[0]}:${parts[1]}`
  return String(label)
}
