'use client'

import { useMemo, useState } from 'react'
import { Eye, Plus, Radio } from 'lucide-react'
import type { TopicInfo } from '@/lib/mock-data'
import { normalizeGrpcUrl } from '@/lib/bus-client'
import { PanelHeader } from './BrokerOverview'
import TopicPanel, { type TopicPanelModel } from './TopicPanel'
import { preferVisualizer } from '@/lib/visualizers/registry'
import { ensureVisualizersRegistered } from '@/lib/visualizers/register-all'
import { useI18n } from '@/lib/i18n'

ensureVisualizersRegistered()

interface Props {
  topics: TopicInfo[]
  grpcAddr: string
  initialTopic?: string | null
  onConsumedInitial?: () => void
}

let panelSeq = 1

export default function VisualizeWorkspace({
  topics,
  grpcAddr,
  initialTopic,
  onConsumedInitial,
}: Props) {
  const { t } = useI18n()
  const grpcUrl = useMemo(() => normalizeGrpcUrl(grpcAddr), [grpcAddr])
  const [panels, setPanels] = useState<TopicPanelModel[]>([])
  const [filter, setFilter] = useState('')
  const [picked, setPicked] = useState('')

  // Open initial topic from Topics table once.
  useMemo(() => {
    if (!initialTopic) return
    const exists = panels.some((p) => p.topic === initialTopic)
    if (!exists) {
      const info = topics.find((x) => x.name === initialTopic)
      queueMicrotask(() => {
        addPanel(initialTopic, info?.typeName)
        onConsumedInitial?.()
      })
    } else {
      queueMicrotask(() => onConsumedInitial?.())
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [initialTopic])

  const sorted = useMemo(() => {
    const q = filter.trim().toLowerCase()
    return [...topics]
      .filter((x) => !q || x.name.toLowerCase().includes(q) || (x.typeName ?? '').toLowerCase().includes(q))
      .sort((a, b) => b.msgPerSec - a.msgPerSec)
  }, [topics, filter])

  function addPanel(topic: string, typeName?: string) {
    if (!topic) return
    const viz = preferVisualizer(typeName)
    setPanels((prev) => [
      ...prev,
      {
        id: `p-${panelSeq++}`,
        topic,
        typeName,
        visualizerId: viz.id,
      },
    ])
  }

  return (
    <div className="h-full flex gap-3 min-h-0" style={{ height: 'calc(100vh - 88px)' }}>
      <aside className="w-64 shrink-0 border border-bus-border bg-bus-panel rounded-sm flex flex-col min-h-0">
        <PanelHeader
          icon={<Radio size={14} />}
          title={t('vizTopics')}
          sub={t('vizTopicsSub', { n: topics.length })}
        />
        <div className="p-2 border-b border-bus-border space-y-2">
          <input
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder={t('vizFilter')}
            className="w-full bg-bus-bg border border-bus-border rounded-sm px-2 py-1.5 font-mono text-[11px] text-bus-text placeholder:text-bus-muted"
          />
          <div className="flex gap-1">
            <select
              value={picked}
              onChange={(e) => setPicked(e.target.value)}
              className="flex-1 min-w-0 bg-bus-bg border border-bus-border rounded-sm px-1.5 py-1.5 font-mono text-[11px] text-bus-text"
            >
              <option value="">{t('vizSelectTopic')}</option>
              {sorted.map((topic) => (
                <option key={topic.name} value={topic.name}>
                  {topic.name}
                </option>
              ))}
            </select>
            <button
              type="button"
              onClick={() => {
                const info = topics.find((x) => x.name === picked)
                addPanel(picked, info?.typeName)
              }}
              disabled={!picked}
              className="px-2 border border-bus-border rounded-sm text-bus-cyan hover:bg-bus-cyan/10 disabled:opacity-40"
              title={t('vizAddPanel')}
            >
              <Plus size={14} />
            </button>
          </div>
          <div className="font-mono text-[10px] text-bus-muted truncate" title={grpcUrl}>
            gRPC {grpcUrl}
          </div>
        </div>
        <div className="flex-1 overflow-y-auto">
          {sorted.length === 0 ? (
            <div className="p-3 font-mono text-xs text-bus-muted">{t('vizNoTopics')}</div>
          ) : (
            sorted.map((topic) => (
              <button
                key={topic.name}
                type="button"
                onClick={() => addPanel(topic.name, topic.typeName)}
                className="w-full text-left px-2.5 py-2 border-b border-[#1f2428] hover:bg-[#1f2428] transition-colors"
              >
                <div className="font-mono text-[12px] text-bus-text truncate">{topic.name}</div>
                <div className="font-mono text-[10px] text-bus-muted truncate">
                  {topic.typeName || t('vizUnknownType')} · {topic.msgPerSec} msg/s
                </div>
              </button>
            ))
          )}
        </div>
      </aside>

      <div className="flex-1 min-w-0 min-h-0 flex flex-col">
        <div className="mb-2 flex items-center gap-2">
          <Eye size={14} className="text-bus-cyan" />
          <h2 className="font-mono text-xs text-bus-muted uppercase tracking-wider">{t('vizTitle')}</h2>
          <span className="font-mono text-[10px] text-bus-muted">{t('vizPanels', { n: panels.length })}</span>
        </div>

        {panels.length === 0 ? (
          <div className="flex-1 border border-dashed border-bus-border rounded-sm flex items-center justify-center">
            <p className="font-mono text-sm text-bus-muted max-w-sm text-center px-4">{t('vizEmpty')}</p>
          </div>
        ) : (
          <div className="flex-1 min-h-0 overflow-y-auto grid grid-cols-1 xl:grid-cols-2 gap-3 content-start">
            {panels.map((panel) => (
              <TopicPanel
                key={panel.id}
                panel={panel}
                grpcUrl={grpcUrl}
                onClose={() => setPanels((prev) => prev.filter((p) => p.id !== panel.id))}
                onChangeVisualizer={(id) =>
                  setPanels((prev) =>
                    prev.map((p) => (p.id === panel.id ? { ...p, visualizerId: id } : p)),
                  )
                }
              />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
