import TankPanel from '@/components/tank/TankPanel'

export default function TankPage() {
  return (
    <div className="min-h-screen bg-bus-bg text-bus-text">
      <header className="h-11 border-b border-bus-border bg-bus-panel flex items-center justify-between px-4">
        <div className="font-mono text-xs font-semibold">
          robot-bus <span className="text-bus-muted">/ tank_viz</span>
        </div>
        <span className="font-mono text-[10px] text-bus-muted">TANK</span>
      </header>
      <TankPanel autoFocus />
    </div>
  )
}
