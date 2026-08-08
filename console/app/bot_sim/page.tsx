import BotSimViewer from '@/components/bot/BotSimViewer'

export default function BotSimPage() {
  return (
    <div className="min-h-screen bg-bus-bg text-bus-text">
      <header className="h-11 border-b border-bus-border bg-bus-panel flex items-center justify-between px-4">
        <div className="font-mono text-xs font-semibold">
          robot-bus <span className="text-bus-muted">/ bot_sim_viewer</span>
        </div>
        <span className="font-mono text-[10px] text-bus-muted">/bot1/pose</span>
      </header>
      <BotSimViewer />
    </div>
  )
}
