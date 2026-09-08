'use client'

import { useEffect, useState } from 'react'
import { ListFilter } from 'lucide-react'
import { resolveBusUrl } from '@/lib/console-bus'
import { useI18n } from '@/lib/i18n'
import { PanelHeader } from './BrokerOverview'

interface Snapshot {
  totalDropped: number
  subscriptions: {
    id: number
    filter: string
    policy: 'drop_newest' | 'drop_oldest' | 'latest'
    pending: number
    capacity: number
    dropped: number
  }[]
}

export default function SubscriptionQueues() {
  const { t } = useI18n()
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null)
  const [unavailable, setUnavailable] = useState(false)

  useEffect(() => {
    const controller = new AbortController()
    let timer: ReturnType<typeof setTimeout> | undefined
    async function poll() {
      const request = new AbortController()
      const timeout = setTimeout(() => request.abort(), 5000)
      const abort = () => request.abort()
      controller.signal.addEventListener('abort', abort, { once: true })
      try {
        const response = await fetch(`${resolveBusUrl()}/api/v1/subscriptions`, { signal: request.signal, cache: 'no-store' })
        if (!response.ok) throw new Error('Subscription metrics unavailable')
        const data: Snapshot = await response.json()
        if (!Array.isArray(data.subscriptions) || typeof data.totalDropped !== 'number') throw new Error('Invalid metrics')
        if (!controller.signal.aborted) { setSnapshot(data); setUnavailable(false) }
      } catch {
        if (!controller.signal.aborted) { setSnapshot(null); setUnavailable(true) }
      } finally {
        clearTimeout(timeout)
        controller.signal.removeEventListener('abort', abort)
        if (!controller.signal.aborted) timer = setTimeout(poll, 1000)
      }
    }
    void poll()
    return () => { controller.abort(); clearTimeout(timer) }
  }, [])

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm shrink-0 min-w-0">
      <PanelHeader icon={<ListFilter size={14} />} title={t('queueTitle')} sub={snapshot ? t('queueTotal', { n: snapshot.totalDropped }) : '—'} />
      <p className="px-3 py-2 text-xs text-bus-muted">{t('queueScope')}</p>
      <div className="max-h-48 overflow-auto bus-scroll">
        {snapshot && snapshot.subscriptions.length > 0 ? (
          <table className="w-full min-w-[560px] text-xs font-mono text-left">
            <thead className="text-bus-muted"><tr>
              {[t('queueFilter'), t('queuePolicy'), t('queuePending'), t('queueDropped')].map(label => <th key={label} className="px-3 py-2 font-normal">{label}</th>)}
            </tr></thead>
            <tbody>{snapshot.subscriptions.map(row => <tr key={row.id} className="border-t border-bus-border">
              <td className="px-3 py-2 break-all text-bus-text">#{row.id} {row.filter || '*'}</td>
              <td className="px-3 py-2 text-bus-cyan">{row.policy === 'drop_newest' ? t('queueDropNewest') : t('queueKeepLast', { n: row.capacity })}</td>
              <td className="px-3 py-2 text-bus-text">{row.pending} / {row.capacity}</td>
              <td className={`px-3 py-2 ${row.dropped > 0 ? 'text-bus-amber' : 'text-bus-muted'}`}>{row.dropped}</td>
            </tr>)}</tbody>
          </table>
        ) : <p role="status" className="px-3 pb-3 text-xs text-bus-muted">{t(unavailable ? 'queueUnavailable' : snapshot ? 'queueEmpty' : 'queueLoading')}</p>}
      </div>
    </section>
  )
}
