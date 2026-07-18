'use client'

import { LayoutDashboard, Radio, Cpu, Zap, ScrollText, Settings } from 'lucide-react'

type Tab = 'overview' | 'topics' | 'services' | 'actions' | 'logs'

interface Props {
  active: Tab
  onSelect: (tab: Tab) => void
}

const NAV_ITEMS: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: 'overview',  label: 'OVERVIEW',  icon: <LayoutDashboard size={14} /> },
  { id: 'topics',    label: 'TOPICS',    icon: <Radio size={14} /> },
  { id: 'services',  label: 'SERVICES',  icon: <Cpu size={14} /> },
  { id: 'actions',   label: 'ACTIONS',   icon: <Zap size={14} /> },
  { id: 'logs',      label: 'EVENTS',    icon: <ScrollText size={14} /> },
]

export default function Sidebar({ active, onSelect }: Props) {
  return (
    <aside className="w-14 bg-[#0e1012] border-r border-[#2a2f35] flex flex-col items-center py-3 gap-1 shrink-0">
      {NAV_ITEMS.map((item) => (
        <button
          key={item.id}
          onClick={() => onSelect(item.id)}
          title={item.label}
          className={`w-10 h-10 flex flex-col items-center justify-center gap-0.5 rounded transition-colors group ${
            active === item.id
              ? 'bg-[#00d4ff]/15 text-[#00d4ff]'
              : 'text-[#5a6370] hover:text-[#c8ced6] hover:bg-[#1a1d20]'
          }`}
        >
          {item.icon}
          <span className="font-mono text-[7px] tracking-widest">{item.label.slice(0, 3)}</span>
        </button>
      ))}

      <div className="flex-1" />

      <button
        title="设置"
        className="w-10 h-10 flex flex-col items-center justify-center gap-0.5 rounded text-[#5a6370] hover:text-[#c8ced6] hover:bg-[#1a1d20] transition-colors"
      >
        <Settings size={14} />
        <span className="font-mono text-[7px] tracking-widest">CFG</span>
      </button>
    </aside>
  )
}
