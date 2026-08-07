import TurtleTeleop from '@/components/turtle/TurtleTeleop'

export default function TurtleTeleopPage() {
  return (
    <div className="min-h-screen bg-bus-bg text-bus-text">
      <header className="h-11 border-b border-bus-border bg-bus-panel flex items-center justify-between px-4">
        <div className="font-mono text-xs font-semibold">
          robot-bus <span className="text-bus-muted">/ turtle_teleop</span>
        </div>
        <span className="font-mono text-[10px] text-bus-muted">/turtle1/cmd_vel</span>
      </header>
      <TurtleTeleop autoFocus />
    </div>
  )
}
