'use client'

import { useRef, type PointerEvent, type ReactNode } from 'react'
import { X } from 'lucide-react'

export interface WindowPosition {
  x: number
  y: number
}

interface Props {
  title: string
  children: ReactNode
  position: WindowPosition
  width: number
  height: number
  zIndex: number
  onPositionChange: (position: WindowPosition) => void
  onBringToFront: () => void
  onClose: () => void
}

export default function FloatingWindow({
  title,
  children,
  position,
  width,
  height,
  zIndex,
  onPositionChange,
  onBringToFront,
  onClose,
}: Props) {
  const drag = useRef<{ pointerX: number; pointerY: number; x: number; y: number } | null>(null)

  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return
    onBringToFront()
    drag.current = {
      pointerX: event.clientX,
      pointerY: event.clientY,
      x: position.x,
      y: position.y,
    }
    event.currentTarget.setPointerCapture(event.pointerId)
  }

  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (!drag.current) return
    const nextX = drag.current.x + event.clientX - drag.current.pointerX
    const nextY = drag.current.y + event.clientY - drag.current.pointerY
    onPositionChange({
      x: Math.max(0, Math.min(nextX, window.innerWidth - 120)),
      y: Math.max(0, Math.min(nextY, window.innerHeight - 36)),
    })
  }

  return (
    <section
      role="dialog"
      aria-label={title}
      onPointerDown={onBringToFront}
      className="fixed flex flex-col overflow-hidden border border-bus-cyan-dim bg-bus-bg shadow-2xl rounded-sm"
      style={{
        left: position.x,
        top: position.y,
        width,
        height,
        maxWidth: 'calc(100vw - 12px)',
        maxHeight: 'calc(100vh - 12px)',
        zIndex,
      }}
    >
      <div
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={(event) => {
          drag.current = null
          event.currentTarget.releasePointerCapture(event.pointerId)
        }}
        onPointerCancel={() => {
          drag.current = null
        }}
        className="h-8 shrink-0 flex items-center justify-between px-2 border-b border-bus-border bg-[#15191d] cursor-move select-none touch-none"
      >
        <span className="font-mono text-[10px] tracking-wider text-bus-cyan">{title}</span>
        <button
          type="button"
          aria-label={`Close ${title}`}
          onPointerDown={(event) => event.stopPropagation()}
          onClick={onClose}
          className="w-6 h-6 flex items-center justify-center text-bus-muted hover:text-bus-red hover:bg-bus-panel rounded-sm"
        >
          <X size={13} />
        </button>
      </div>
      <div className="flex-1 min-h-0 overflow-auto">{children}</div>
    </section>
  )
}
