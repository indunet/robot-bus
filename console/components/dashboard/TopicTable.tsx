'use client'

import { type TopicInfo, fmtBytes, fmtNum, fmtAge } from '@/lib/mock-data'
import { PanelHeader } from './BrokerOverview'
import { LineChart, Line, ResponsiveContainer } from 'recharts'
import { Radio } from 'lucide-react'

interface Props {
  topics: TopicInfo[]
}

export default function TopicTable({ topics }: Props) {
  // 按 msg/s 降序
  const sorted = [...topics].sort((a, b) => b.msgPerSec - a.msgPerSec)

  return (
    <section className="border border-[#2a2f35] bg-[#1a1d20] rounded-sm flex flex-col min-h-0">
      <PanelHeader icon={<Radio size={12} />} title="TOPICS" sub={`${topics.length} 活跃`} />

      {/* 表头 */}
      <div className="grid grid-cols-[1fr_70px_80px_70px_50px_60px_80px] items-center px-3 h-7 border-b border-[#2a2f35] bg-[#141618]">
        {['TOPIC', 'MSG/S', 'BANDWIDTH', 'TOTAL', 'PUB', 'SUB', 'LAST SEEN'].map((h, i) => (
          <span
            key={h}
            className={`font-mono text-[10px] text-[#5a6370] uppercase tracking-wider ${i > 0 ? 'text-right' : ''}`}
          >
            {h}
          </span>
        ))}
      </div>

      {/* 数据行 */}
      <div className="flex-1 overflow-y-auto">
        {sorted.map((t) => (
          <TopicRow key={t.name} topic={t} />
        ))}
      </div>
    </section>
  )
}

function TopicRow({ topic }: { topic: TopicInfo }) {
  const isIdle = topic.msgPerSec === 0
  const isHot  = topic.msgPerSec > 80

  const sparkData = topic.sparkline.map((v, i) => ({ i, v }))

  const rateColor = isIdle ? 'text-[#5a6370]' : isHot ? 'text-[#f59e0b]' : 'text-[#00d4ff]'
  const sparkColor = isIdle ? '#2a2f35' : isHot ? '#f59e0b' : '#00d4ff'
  const rowBg = isIdle ? '' : 'hover:bg-[#1f2428]'

  return (
    <div
      className={`grid grid-cols-[1fr_70px_80px_70px_50px_60px_80px] items-center px-3 h-8 border-b border-[#1f2428] transition-colors cursor-default ${rowBg}`}
    >
      {/* Topic 名称 */}
      <div className="flex items-center gap-2 min-w-0">
        <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${isIdle ? 'bg-[#5a6370]' : 'bg-[#22c55e] pulse-green'}`} />
        <span className={`font-mono text-[11px] truncate ${isIdle ? 'text-[#5a6370]' : 'text-[#c8ced6]'}`}>
          {topic.name}
        </span>
      </div>

      {/* Sparkline + msg/s */}
      <div className="flex items-center justify-end gap-1.5">
        <div className="w-10 h-4">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={sparkData}>
              <Line
                type="monotone"
                dataKey="v"
                stroke={sparkColor}
                strokeWidth={1}
                dot={false}
                isAnimationActive={false}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
        <span className={`font-mono text-[11px] tabular-nums ${rateColor} w-7 text-right`}>
          {topic.msgPerSec}
        </span>
      </div>

      {/* Bandwidth */}
      <span className="font-mono text-[11px] text-[#c8ced6] tabular-nums text-right">
        {isIdle ? '—' : `${fmtBytes(topic.bytesPerSec)}/s`}
      </span>

      {/* Total */}
      <span className="font-mono text-[11px] text-[#5a6370] tabular-nums text-right">
        {fmtNum(topic.totalMsgs)}
      </span>

      {/* Publishers */}
      <span className="font-mono text-[11px] text-[#5a6370] tabular-nums text-right">
        {topic.publishers}
      </span>

      {/* Subscribers */}
      <span className="font-mono text-[11px] text-[#5a6370] tabular-nums text-right">
        {topic.subscribers}
      </span>

      {/* Last seen */}
      <span className={`font-mono text-[11px] tabular-nums text-right ${isIdle ? 'text-[#f59e0b]' : 'text-[#5a6370]'}`}>
        {fmtAge(topic.lastSeen)}
      </span>
    </div>
  )
}
