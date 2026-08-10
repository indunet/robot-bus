'use client'

import { type TopicInfo, fmtBytes, fmtNum } from '@/lib/mock-data'
import { PanelHeader } from './BrokerOverview'
import TruncateTip from './TruncateTip'
import { LineChart, Line, ResponsiveContainer } from 'recharts'
import { Radio } from 'lucide-react'
import { fmtAgeLocalized, useI18n } from '@/lib/i18n'

interface Props {
  topics: TopicInfo[]
  /** Cap list height (overview split with chart). */
  maxBodyHeight?: string
}

const COLS = 'grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)_80px_96px_80px_56px_56px_96px]'

export default function TopicTable({ topics, maxBodyHeight }: Props) {
  const { t } = useI18n()
  const sorted = [...topics].sort((a, b) => b.msgPerSec - a.msgPerSec)
  const headers = [
    t('colTopic'),
    t('colType'),
    t('colMsgS'),
    t('colBandwidth'),
    t('colTotal'),
    t('colPub'),
    t('colSub'),
    t('colLastSeen'),
  ]

  return (
    <section className="border border-bus-border bg-bus-panel rounded-sm flex flex-col min-h-0 h-full">
      <PanelHeader
        icon={<Radio size={14} />}
        title={t('topicsTitle')}
        sub={t('topicsActive', { n: topics.length })}
        subClassName="text-bus-cyan"
      />

      <div className="overflow-x-auto bus-scroll flex-1 min-h-0">
        <div className="min-w-[860px] h-full flex flex-col">
          <div className={`grid ${COLS} items-center px-3 h-8 border-b border-bus-border bg-bus-bg shrink-0`}>
            {headers.map((h, i) => (
              <span
                key={`${h}-${i}`}
                className={`font-mono text-xs text-bus-muted uppercase tracking-wider ${i > 1 ? 'text-right' : ''}`}
              >
                {h}
              </span>
            ))}
          </div>

          <div
            className="overflow-y-auto bus-scroll"
            style={maxBodyHeight ? { maxHeight: maxBodyHeight } : undefined}
          >
            {sorted.length === 0 ? (
              <div className="flex items-center justify-center h-14 text-bus-muted font-mono text-xs">
                —
              </div>
            ) : (
              sorted.map((topic) => (
                <TopicRow key={topic.name} topic={topic} />
              ))
            )}
          </div>
        </div>
      </div>
    </section>
  )
}

function TopicRow({ topic }: { topic: TopicInfo }) {
  const { t } = useI18n()
  const isIdle = topic.msgPerSec === 0
  const isHot = topic.msgPerSec > 80

  const sparkData = topic.sparkline.map((v, i) => ({ i, v }))

  const rateColor = isIdle ? 'text-bus-muted' : isHot ? 'text-bus-amber' : 'text-bus-cyan'
  const sparkColor = isIdle ? '#2a2f35' : isHot ? '#f59e0b' : '#00d4ff'

  return (
    <div
      className={`grid ${COLS} items-center px-3 h-9 border-b border-[#1f2428] transition-colors cursor-default hover:bg-[#1f2428]`}
    >
      <div className="flex items-center gap-2 min-w-0">
        <span className={`w-2 h-2 rounded-full shrink-0 ${isIdle ? 'bg-bus-muted' : 'bg-bus-green pulse-green'}`} />
        <TruncateTip
          text={topic.name}
          className={`font-mono text-[13px] font-medium ${isIdle ? 'text-bus-muted' : 'text-bus-text'}`}
        />
      </div>

      <TruncateTip
        text={topic.typeName || undefined}
        className="font-mono text-[12px] text-[#6b8294]"
      />

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
    </div>
  )
}
