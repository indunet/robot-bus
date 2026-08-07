'use client'

import {
  LayoutDashboard,
  Radio,
  ScrollText,
  Cpu,
  Zap,
  Network,
  Bot,
} from 'lucide-react'
import { useI18n } from '@/lib/i18n'

export type Tab =
  | 'overview'
  | 'topics'
  | 'services'
  | 'actions'
  | 'logs'
  | 'topology'

interface Props {
  active: Tab
  onSelect: (tab: Tab) => void
  onOpenBot: () => void
}

export default function Sidebar({ active, onSelect, onOpenBot }: Props) {
  const { t } = useI18n()

  const items: { id: Tab; label: string; short: string; icon: React.ReactNode }[] = [
    { id: 'overview', label: t('navOverview'), short: t('navOverviewShort'), icon: <LayoutDashboard size={16} /> },
    { id: 'topology', label: t('navTopology'), short: t('navTopologyShort'), icon: <Network size={16} /> },
    { id: 'topics', label: t('navTopics'), short: t('navTopicsShort'), icon: <Radio size={16} /> },
    { id: 'services', label: t('navServices'), short: t('navServicesShort'), icon: <Cpu size={16} /> },
    { id: 'actions', label: t('navActions'), short: t('navActionsShort'), icon: <Zap size={16} /> },
    { id: 'logs', label: t('navEvents'), short: t('navEventsShort'), icon: <ScrollText size={16} /> },
  ]

  return (
    <aside className="w-16 bg-[#0e1012] border-r border-bus-border flex flex-col items-center py-3 gap-1.5 shrink-0">
      {items.map((item) => (
        <button
          key={item.id}
          onClick={() => onSelect(item.id)}
          title={item.label}
          className={`w-12 h-12 flex flex-col items-center justify-center gap-0.5 rounded transition-colors group ${
            active === item.id
              ? 'bg-bus-cyan/15 text-bus-cyan'
              : 'text-bus-muted hover:text-bus-text hover:bg-bus-panel'
          }`}
        >
          {item.icon}
          <span className="font-mono text-[9px] tracking-widest">{item.short}</span>
        </button>
      ))}
      <div className="w-8 border-t border-bus-border my-1" />
      <button
        type="button"
        onClick={onOpenBot}
        title={t('navBotWindows')}
        className="relative w-12 h-12 flex flex-col items-center justify-center gap-0.5 rounded transition-colors text-bus-muted hover:text-bus-cyan hover:bg-bus-panel"
      >
        <Bot size={17} />
        <span className="font-mono text-[9px] tracking-widest">{t('navBotSimShort')}</span>
      </button>
    </aside>
  )
}
