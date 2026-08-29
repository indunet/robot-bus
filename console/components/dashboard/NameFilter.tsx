'use client'

import { Search, X } from 'lucide-react'
import { useI18n } from '@/lib/i18n'

export function nameMatches(query: string, ...fields: (string | undefined)[]): boolean {
  const q = query.trim().toLowerCase()
  if (!q) return true
  return fields.some((f) => f?.toLowerCase().includes(q))
}

export default function NameFilter({
  value,
  onChange,
}: {
  value: string
  onChange: (next: string) => void
}) {
  const { t } = useI18n()

  return (
    <label className="relative flex items-center shrink-0">
      <Search size={12} className="absolute left-1.5 text-bus-muted pointer-events-none" />
      <input
        type="text"
        role="searchbox"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Escape' && value) {
            e.preventDefault()
            onChange('')
          }
        }}
        placeholder={t('nameFilterPlaceholder')}
        aria-label={t('nameFilterAria')}
        className="h-6 w-[10.5rem] rounded-sm border border-bus-border bg-bus-bg pl-6 pr-6 font-mono text-xs text-bus-text outline-none placeholder:text-bus-muted focus:border-bus-cyan"
      />
      {value ? (
        <button
          type="button"
          onClick={() => onChange('')}
          className="absolute right-1 flex h-4 w-4 items-center justify-center text-bus-muted hover:text-bus-text"
          aria-label={t('nameFilterClear')}
        >
          <X size={11} />
        </button>
      ) : null}
    </label>
  )
}
