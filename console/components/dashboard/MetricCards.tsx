'use client'

import { useRef, useEffect, useState } from 'react'
import { type BrokerInfo, fmtBytes, fmtNum, fmtUptime } from '@/lib/mock-data'
import { Activity, Database, Clock, Hash, Wifi, AlertCircle } from 'lucide-react'

interface Props {
  broker: BrokerInfo
}

// 切角尺寸（px）
const C = 10

export default function MetricCards({ broker }: Props) {
  const cards = [
    {
      icon: <Activity size={13} />,
      label: 'MSG/S',
      value: fmtNum(broker.msgPerSec),
      sub: '实时消息速率',
      variant: 'accent' as const,
    },
    {
      icon: <Wifi size={13} />,
      label: 'BANDWIDTH',
      value: `${fmtBytes(broker.bytesPerSec)}/s`,
      sub: '实时带宽',
      variant: 'accent' as const,
    },
    {
      icon: <Database size={13} />,
      label: 'TOTAL MSGS',
      value: fmtNum(broker.totalMessages),
      sub: '累计消息数',
      variant: 'normal' as const,
    },
    {
      icon: <Clock size={13} />,
      label: 'UPTIME',
      value: fmtUptime(broker.uptime),
      sub: '连续运行时长',
      variant: 'normal' as const,
    },
    {
      icon: <Hash size={13} />,
      label: 'PID',
      value: String(broker.pid),
      sub: `v${broker.version}`,
      variant: 'normal' as const,
    },
    {
      icon: <AlertCircle size={13} />,
      label: 'ERRORS',
      value: String(broker.totalErrors),
      sub: '累计错误数',
      variant: broker.totalErrors > 0 ? ('error' as const) : ('normal' as const),
    },
  ]

  return (
    <div className="grid grid-cols-3 gap-2">
      {cards.map((c) => (
        <MetricCard key={c.label} {...c} />
      ))}
    </div>
  )
}

/** 用 ResizeObserver 获取实际像素尺寸，再画 SVG 切角边框 */
function MetricCard({
  icon,
  label,
  value,
  sub,
  variant,
}: {
  icon: React.ReactNode
  label: string
  value: string
  sub: string
  variant: 'accent' | 'normal' | 'error'
}) {
  const wrapRef = useRef<HTMLDivElement>(null)
  const [size, setSize] = useState({ w: 0, h: 0 })

  useEffect(() => {
    const el = wrapRef.current
    if (!el) return
    const ro = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect
      setSize({ w: Math.round(width), h: Math.round(height) })
    })
    ro.observe(el)
    return () => ro.disconnect()
  }, [])

  const { w, h } = size

  const strokeColor =
    variant === 'accent' ? '#00d4ff'
    : variant === 'error' ? '#ef4444'
    : '#2a2f35'

  const strokeOpacity =
    variant === 'accent' ? 0.6
    : variant === 'error' ? 0.55
    : 1

  const valueColor =
    variant === 'accent' ? '#00d4ff'
    : variant === 'error' ? '#ef4444'
    : '#c8ced6'

  const iconColor =
    variant === 'accent' ? '#00d4ff'
    : variant === 'error' ? '#ef4444'
    : '#5a6370'

  // clip-path：accent 卡双切角（左上 + 右下），其余只右下切
  const clipPath =
    variant === 'accent'
      ? `polygon(${C}px 0, 100% 0, 100% calc(100% - ${C}px), calc(100% - ${C}px) 100%, 0 100%, 0 ${C}px)`
      : `polygon(0 0, 100% 0, 100% calc(100% - ${C}px), calc(100% - ${C}px) 100%, 0 100%)`

  // SVG 切角路径点（根据实际像素尺寸计算）
  const borderPath = w && h
    ? (variant === 'accent'
        ? `M${C},0 L${w},0 L${w},${h - C} L${w - C},${h} L0,${h} L0,${C} Z`
        : `M0,0 L${w},0 L${w},${h - C} L${w - C},${h} L0,${h} Z`)
    : ''

  return (
    <div
      ref={wrapRef}
      className="relative"
      style={{ clipPath }}
    >
      {/* 背景 */}
      <div className="absolute inset-0 bg-[#1a1d20]" />

      {/* SVG 切角边框描边 */}
      {borderPath && (
        <svg
          className="absolute inset-0 pointer-events-none overflow-visible"
          width={w}
          height={h}
          viewBox={`0 0 ${w} ${h}`}
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d={borderPath}
            fill="none"
            stroke={strokeColor}
            strokeWidth="1"
            strokeOpacity={strokeOpacity}
          />
          {/* accent 卡：顶边加亮线 */}
          {variant === 'accent' && (
            <line
              x1={C}
              y1="0.5"
              x2={w}
              y2="0.5"
              stroke="#00d4ff"
              strokeWidth="1.5"
              strokeOpacity="0.9"
            />
          )}
        </svg>
      )}

      {/* 卡片内容 */}
      <div className="relative px-3 py-2.5">
        <div className="flex items-center gap-1.5 mb-1.5">
          <span style={{ color: iconColor }}>{icon}</span>
          <span className="font-mono text-[10px] text-[#5a6370] uppercase tracking-wider leading-none">
            {label}
          </span>
        </div>
        <div
          className="font-mono text-base font-bold tabular-nums leading-none"
          style={{ color: valueColor }}
        >
          {value}
        </div>
        <div className="font-mono text-[10px] text-[#5a6370] mt-1 leading-none">{sub}</div>
      </div>
    </div>
  )
}
