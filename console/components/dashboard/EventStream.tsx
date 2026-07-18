'use client'

import { useState, useRef, useEffect } from 'react'
import { type LogEntry, type LogLevel } from '@/lib/mock-data'
import { PanelHeader } from './BrokerOverview'
import { Filter, Pause, Play, Trash2, Terminal } from 'lucide-react'

interface Props {
  logs: LogEntry[]
}

const LEVEL_COLORS: Record<LogLevel, string> = {
  DEBUG: 'text-[#5a6370]',
  INFO:  'text-[#00d4ff]',
  WARN:  'text-[#f59e0b]',
  ERROR: 'text-[#ef4444]',
}
const LEVEL_BG: Record<LogLevel, string> = {
  DEBUG: '',
  INFO:  '',
  WARN:  'bg-[#f59e0b]/5',
  ERROR: 'bg-[#ef4444]/8',
}

export default function EventStream({ logs }: Props) {
  const [filter, setFilter] = useState<LogLevel | 'ALL'>('ALL')
  const [paused, setPaused] = useState(false)
  const [entries, setEntries] = useState<LogEntry[]>(logs)
  const bottomRef = useRef<HTMLDivElement>(null)

  // 接收外部 logs 更新（实时新消息追加）
  useEffect(() => {
    if (!paused) setEntries(logs)
  }, [logs, paused])

  // 自动滚动到底部（在日志容器内，不影响页面滚动）
  const scrollContainerRef = useRef<HTMLDivElement>(null)
  useEffect(() => {
    if (!paused && scrollContainerRef.current) {
      scrollContainerRef.current.scrollTop = scrollContainerRef.current.scrollHeight
    }
  }, [entries, paused])

  const visible = filter === 'ALL' ? entries : entries.filter((e) => e.level === filter)

  return (
    <section className="border border-[#2a2f35] bg-[#1a1d20] rounded-sm flex flex-col min-h-0">
      <div className="flex items-center justify-between px-3 h-8 border-b border-[#2a2f35] shrink-0">
        <div className="flex items-center gap-2">
          <Terminal size={12} className="text-[#00d4ff]" />
          <span className="font-mono text-[11px] font-bold text-[#c8ced6] tracking-widest uppercase">EVENTS</span>
          <span className="font-mono text-[10px] text-[#5a6370]">实时日志流</span>
        </div>
        <div className="flex items-center gap-2">
          {/* 级别过滤 */}
          <Filter size={11} className="text-[#5a6370]" />
          {(['ALL', 'DEBUG', 'INFO', 'WARN', 'ERROR'] as const).map((lvl) => (
            <button
              key={lvl}
              onClick={() => setFilter(lvl)}
              className={`font-mono text-[10px] px-1.5 py-px rounded border transition-colors ${
                filter === lvl
                  ? 'border-[#00d4ff] text-[#00d4ff] bg-[#00d4ff]/10'
                  : 'border-[#2a2f35] text-[#5a6370] hover:text-[#c8ced6] hover:border-[#3a4045]'
              }`}
            >
              {lvl}
            </button>
          ))}
          <div className="w-px h-4 bg-[#2a2f35]" />
          {/* 暂停 */}
          <button
            onClick={() => setPaused((p) => !p)}
            className={`flex items-center gap-1 font-mono text-[10px] px-1.5 py-px rounded border transition-colors ${
              paused
                ? 'border-[#f59e0b] text-[#f59e0b] bg-[#f59e0b]/10'
                : 'border-[#2a2f35] text-[#5a6370] hover:text-[#c8ced6]'
            }`}
          >
            {paused ? <Play size={9} /> : <Pause size={9} />}
            {paused ? 'RESUME' : 'PAUSE'}
          </button>
          {/* 清空 */}
          <button
            onClick={() => setEntries([])}
            className="flex items-center gap-1 font-mono text-[10px] px-1.5 py-px rounded border border-[#2a2f35] text-[#5a6370] hover:text-[#ef4444] hover:border-[#ef4444] transition-colors"
          >
            <Trash2 size={9} />
            CLR
          </button>
        </div>
      </div>

      {/* 日志列 */}
      <div ref={scrollContainerRef} className="flex-1 overflow-y-auto min-h-0 font-mono text-[11px]">
        {visible.length === 0 ? (
          <div className="flex items-center justify-center h-16 text-[#5a6370] text-[11px]">
            无日志条目
          </div>
        ) : (
          visible.map((entry) => (
            <LogLine key={entry.id} entry={entry} />
          ))
        )}
        <div ref={bottomRef} />
      </div>

      {/* 状态栏 */}
      <div className="h-6 border-t border-[#2a2f35] flex items-center px-3 gap-3 shrink-0 bg-[#141618]">
        <span className="font-mono text-[10px] text-[#5a6370]">{visible.length} 条目</span>
        {paused && (
          <span className="font-mono text-[10px] text-[#f59e0b] animate-pulse">● 已暂停</span>
        )}
        {!paused && (
          <span className="font-mono text-[10px] text-[#22c55e]">● 实时</span>
        )}
      </div>
    </section>
  )
}

function LogLine({ entry }: { entry: LogEntry }) {
  const ts = new Date(entry.ts)
  const timeStr = ts.toLocaleTimeString('zh-CN', {
    hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
  }) + '.' + String(ts.getMilliseconds()).padStart(3, '0')

  return (
    <div className={`flex items-start gap-2 px-3 py-0.5 border-b border-[#1a1d20] hover:bg-[#1f2428] ${LEVEL_BG[entry.level]}`}>
      <span className="text-[#5a6370] tabular-nums shrink-0 w-[88px]">{timeStr}</span>
      <span className={`${LEVEL_COLORS[entry.level]} font-bold shrink-0 w-[38px]`}>{entry.level}</span>
      <span className="text-[#007fa0] shrink-0 w-[100px] truncate">{entry.source}</span>
      <span className="text-[#c8ced6] leading-relaxed">{entry.message}</span>
    </div>
  )
}
