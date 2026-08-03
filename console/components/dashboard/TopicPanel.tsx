'use client'

import { useEffect, useMemo, useState } from 'react'
import { X } from 'lucide-react'
import { useTopicSubscription } from '@/lib/useTopicSubscription'
import { resolveMsgType } from '@/lib/msg-types'
import {
  getVisualizer,
  preferVisualizer,
  visualizersForType,
} from '@/lib/visualizers/registry'
import { ensureVisualizersRegistered } from '@/lib/visualizers/register-all'
import { useI18n } from '@/lib/i18n'

ensureVisualizersRegistered()

export interface TopicPanelModel {
  id: string
  topic: string
  typeName?: string
  visualizerId?: string
}

interface Props {
  panel: TopicPanelModel
  grpcUrl: string
  onClose: () => void
  onChangeVisualizer: (id: string) => void
}

export default function TopicPanel({ panel, grpcUrl, onClose, onChangeVisualizer }: Props) {
  const { t } = useI18n()
  const msgType = useMemo(() => resolveMsgType(panel.typeName), [panel.typeName])
  const options = useMemo(() => visualizersForType(panel.typeName), [panel.typeName])
  const preferred = useMemo(() => preferVisualizer(panel.typeName), [panel.typeName])
  const [vizId, setVizId] = useState(panel.visualizerId ?? preferred.id)

  useEffect(() => {
    if (panel.visualizerId) setVizId(panel.visualizerId)
  }, [panel.visualizerId])

  const viz = getVisualizer(vizId) ?? preferred
  const Comp = viz.component

  const sub = useTopicSubscription({
    topic: panel.topic,
    grpcUrl,
    msgType,
    enabled: Boolean(panel.topic && grpcUrl),
    maxHz: vizId.includes('image') || vizId.includes('video') || vizId === 'laser-scan' ? 12 : 20,
  })

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm flex flex-col min-h-[280px] overflow-hidden">
      <header className="flex items-center gap-2 px-2 h-9 border-b border-bus-border shrink-0 bg-bus-bg/50">
        <div className="min-w-0 flex-1">
          <div className="font-mono text-[12px] text-bus-text truncate">{panel.topic}</div>
          <div className="font-mono text-[10px] text-bus-muted truncate">
            {panel.typeName || t('vizUnknownType')}
            {sub.status === 'live' && (
              <span className="text-bus-cyan"> · {sub.hz} Hz · #{sub.msgCount}</span>
            )}
            {sub.status === 'connecting' && <span> · {t('vizConnecting')}</span>}
            {sub.status === 'error' && (
              <span className="text-bus-amber"> · {sub.error || t('vizError')}</span>
            )}
          </div>
        </div>

        <select
          value={viz.id}
          onChange={(e) => {
            setVizId(e.target.value)
            onChangeVisualizer(e.target.value)
          }}
          className="bg-bus-bg border border-bus-border text-bus-text font-mono text-[10px] px-1.5 py-1 rounded-sm max-w-[9rem]"
          title={t('vizVisualizer')}
        >
          {options.map((o) => (
            <option key={o.id} value={o.id}>
              {o.label}
            </option>
          ))}
        </select>

        <button
          type="button"
          onClick={onClose}
          className="p-1 text-bus-muted hover:text-bus-text"
          title={t('vizClosePanel')}
        >
          <X size={14} />
        </button>
      </header>

      <div className="flex-1 min-h-0">
        <Comp
          message={sub.message}
          raw={sub.raw}
          topic={panel.topic}
          typeName={panel.typeName}
        />
      </div>
    </section>
  )
}
