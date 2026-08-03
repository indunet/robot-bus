'use client'

import { type TopicInfo, fmtBytes, fmtNum } from '@/lib/mock-data'
import { PanelHeader } from './BrokerOverview'
import { LineChart, Line, ResponsiveContainer } from 'recharts'
import { Eye, Radio } from 'lucide-react'
import { fmtAgeLocalized, useI18n } from '@/lib/i18n'

interface Props {
  topics: TopicInfo[]
  /** Cap list height (overview split with chart). */
  maxBodyHeight?: string
  onVisualize?: (topic: string) => void
}

const COLS = 'grid-cols-[1fr_80px_96px_80px_56px_56px_96px_40px]'

export default function TopicTable({ topics, maxBodyHeight, onVisualize }: Props) {
  const { t } = useI18n()
  const sorted = [...topics].sort((a, b) => b.msgPerSec - a.msgPerSec)
  const headers = [
    t('colTopic'),
    t('colMsgS'),
    t('colBandwidth'),
    t('colTotal'),
    t('colPub'),
    t('colSub'),
    t('colLastSeen'),
    '',
  ]

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm flex flex-col min-h-0 h-full">
      <PanelHeader
        icon={<Radio size={14} />}
        title={t('topicsTitle')}
        sub={t('topicsActive', { n: topics.length })}
      />

      <div className="overflow-x-auto flex-1 min-h-0">
        <div className="min-w-[760px] h-full flex flex-col">
          <div className={`grid ${COLS} items-center px-3 h-8 border-b border-bus-border bg-bus-bg shrink-0`}>
            {headers.map((h, i) => (
              <span
                key={`${h}-${i}`}
                className={`font-mono text-xs text-bus-muted uppercase tracking-wider ${i > 0 ? 'text-right' : ''}`}
              >
                {h}
              </span>
            ))}
          </div>

          <div
            className="overflow-y-auto"
            style={maxBodyHeight ? { maxHeight: maxBodyHeight } : undefined}
          >
            {sorted.length === 0 ? (
              <div className="flex items-center justify-center h-14 text-bus-muted font-mono text-xs">
                —
              </div>
            ) : (
              sorted.map((topic) => (
                <TopicRow key={topic.name} topic={topic} onVisualize={onVisualize} />
              ))
            )}
          </div>
        </div>
      </div>
    </section>
  )
}

function TopicRow({
  topic,
  onVisualize,
}: {
  topic: TopicInfo
  onVisualize?: (topic: string) => void
}) {
  const { t } = useI18n()
  const isIdle = topic.msgPerSec === 0
  const isHot = topic.msgPerSec > 80

  const sparkData = topic.sparkline.map((v, i) => ({ i, v }))

  const rateColor = isIdle ? 'text-bus-muted' : isHot ? 'text-bus-amber' : 'text-bus-cyan'
  const sparkColor = isIdle ? '#2a2f35' : isHot ? '#f59e0b' : '#00d4ff'
  const rowBg = isIdle ? '' : 'hover:bg-[#1f2428]'

  return (
    <div
      className={`grid ${COLS} items-center px-3 h-9 border-b border-[#1f2428] transition-colors cursor-default ${rowBg}`}
    >
      <div className="flex items-center gap-2 min-w-0">
        <span className={`w-2 h-2 rounded-full shrink-0 ${isIdle ? 'bg-bus-muted' : 'bg-bus-green pulse-green'}`} />
        <div className="min-w-0 flex flex-col">
          <span className={`font-mono text-[13px] font-medium truncate ${isIdle ? 'text-bus-muted' : 'text-bus-text'}`}>
            {topic.name}
          </span>
          {topic.typeName && (
            <span className="font-mono text-[10px] text-[#6b8294] truncate">{topic.typeName}</span>
          )}
        </div>
      </div>

      <div className="flex items-center justify-end gap-1.5">
        <div className="w-12 h-5">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={sparkData}>
              <Line
                type="monotone"
                dataKey="v"
                stroke={sparkColor}
                strokeWidth={1.5}
                dot={false}
                isAnimationActive={false}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
        <span className={`font-mono text-[13px] tabular-nums ${rateColor} w-8 text-right`}>
          {topic.msgPerSec}
        </span>
      </div>

      <span className="font-mono text-[13px] text-bus-text tabular-nums text-right">
        {isIdle ? '—' : `${fmtBytes(topic.bytesPerSec)}/s`}
      </span>

      <span className="font-mono text-[13px] text-bus-muted tabular-nums text-right">
        {fmtNum(topic.totalMsgs)}
      </span>

      <span className="font-mono text-[13px] text-bus-muted tabular-nums text-right">
        {topic.publishers}
      </span>

      <span className="font-mono text-[13px] text-bus-muted tabular-nums text-right">
        {topic.subscribers}
      </span>

      <span className={`font-mono text-[13px] tabular-nums text-right ${isIdle ? 'text-bus-amber' : 'text-bus-muted'}`}>
        {fmtAgeLocalized(topic.lastSeen, t)}
      </span>

      <div className="flex justify-end">
        {onVisualize && (
          <button
            type="button"
            onClick={() => onVisualize(topic.name)}
            className="p-1 text-bus-muted hover:text-bus-cyan"
            title={t('vizOpen')}
          >
            <Eye size={14} />
          </button>
        )}
      </div>
    </div>
  )
}
