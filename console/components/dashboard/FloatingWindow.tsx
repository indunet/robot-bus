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
      className="fixed flex flex-col overflow-hidden border border-bus-border bg-bus-panel/95 shadow-[0_1px_0_rgb(255_255_255_/10%)_inset,0_-1px_0_rgb(0_0_0_/28%)_inset,0_22px_48px_rgb(0_0_0_/42%),0_8px_18px_rgb(0_0_0_/28%),0_0_0_1px_rgb(0_212_255_/14%)] backdrop-blur-[8px]"
      style={{
        left: position.x,
        top: position.y,
        width,
        height,
        maxWidth: 'calc(100vw - 12px)',
        maxHeight: 'calc(100vh - 12px)',
        zIndex,
        clipPath:
          'polygon(0 0, calc(100% - 12px) 0, 100% 12px, 100% 100%, 12px 100%, 0 calc(100% - 12px))',
        transform: 'translateZ(0)',
      }}
    >
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 z-0 bg-gradient-to-br from-white/[0.03] via-transparent to-black/15"
      />
      <div aria-hidden="true" className="pointer-events-none absolute inset-0 z-20">
        <span className="absolute left-0 top-0 h-[2px] w-28 bg-gradient-to-r from-bus-cyan to-bus-cyan/10" />
        <span className="absolute right-[-2px] top-[5px] h-px w-[17px] rotate-45 bg-bus-cyan" />
        <span className="absolute bottom-[5px] left-[-2px] h-px w-[17px] rotate-45 bg-bus-cyan" />
        <span className="absolute bottom-0 right-5 h-[2px] w-16 bg-gradient-to-l from-bus-cyan/80 to-transparent" />
        <span className="absolute right-0 top-16 h-12 w-[2px] bg-gradient-to-b from-bus-cyan/60 to-transparent" />
        <span className="absolute inset-x-0 top-0 h-px bg-white/30" />
        <span className="absolute inset-y-0 left-0 w-px bg-white/12" />
        <span className="absolute inset-y-0 right-0 w-px bg-black/30" />
        <span className="absolute inset-x-0 bottom-0 h-px bg-black/35" />
      </div>
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
        className="relative z-10 h-8 shrink-0 flex items-center justify-between px-2 border-b border-bus-border bg-bus-bg/90 cursor-move select-none touch-none shadow-[0_1px_0_rgb(255_255_255_/06%)_inset]"
      >
        <span className="font-mono text-[10px] tracking-wider text-bus-cyan drop-shadow-[0_1px_2px_rgb(0_0_0_/55%)]">
          {title}
        </span>
        <button
          type="button"
          aria-label={`Close ${title}`}
          onPointerDown={(event) => event.stopPropagation()}
          onClick={onClose}
          className="w-6 h-6 flex items-center justify-center text-bus-muted hover:text-bus-red hover:bg-white/10 rounded-sm"
        >
          <X size={13} />
        </button>
      </div>
      <div className="relative z-10 flex-1 min-h-0 overflow-auto">{children}</div>
    </section>
  )
}
