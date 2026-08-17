'use client'

import { useEffect, useRef, type PointerEvent, type ReactNode } from 'react'
import { X } from 'lucide-react'

export interface WindowPosition {
  x: number
  y: number
}

interface Props {
  title: string
  titleIcon?: ReactNode
  children: ReactNode
  position: WindowPosition
  width: number
  height: number
  zIndex: number
  /** When false, no dim overlay — dashboard stays clickable and outside clicks do not close. */
  modal?: boolean
  onPositionChange: (position: WindowPosition) => void
  onBringToFront: () => void
  onClose: () => void
}

function setDocumentDragLock(locked: boolean) {
  document.body.style.userSelect = locked ? 'none' : ''
  document.body.style.webkitUserSelect = locked ? 'none' : ''
  if (locked) window.getSelection()?.removeAllRanges()
}

export default function FloatingWindow({
  title,
  titleIcon,
  children,
  position,
  width,
  height,
  zIndex,
  modal = true,
  onPositionChange,
  onBringToFront,
  onClose,
}: Props) {
  const drag = useRef<{ pointerX: number; pointerY: number; x: number; y: number } | null>(null)

  useEffect(() => {
    return () => setDocumentDragLock(false)
  }, [])

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return
      event.preventDefault()
      onClose()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [onClose])

  const endDrag = (event?: PointerEvent<HTMLDivElement>) => {
    if (!drag.current) return
    drag.current = null
    setDocumentDragLock(false)
    if (event) {
      try {
        event.currentTarget.releasePointerCapture(event.pointerId)
      } catch {
        // capture may already be released
      }
    }
  }

  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return
    // Prevent native text/image selection while the window is dragged.
    event.preventDefault()
    onBringToFront()
    drag.current = {
      pointerX: event.clientX,
      pointerY: event.clientY,
      x: position.x,
      y: position.y,
    }
    setDocumentDragLock(true)
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
    <>
      {modal ? (
        <div
          aria-hidden="true"
          className="fixed inset-0 bg-black/40"
          style={{ zIndex: Math.max(1, zIndex - 1) }}
          onPointerDown={(event) => {
            event.preventDefault()
            onClose()
          }}
        />
      ) : null}
      <section
        role="dialog"
        aria-label={title}
        aria-modal={modal}
        onPointerDown={onBringToFront}
        className="fixed flex flex-col overflow-hidden rounded-lg border border-[#3a8fa3]/50 bg-bus-panel/90 shadow-[0_1px_0_rgb(255_255_255_/06%)_inset,0_0_0_1px_rgb(0_183_216_/18%),0_0_32px_rgb(0_183_216_/10%),0_28px_56px_rgb(0_0_0_/55%),0_10px_20px_rgb(0_0_0_/35%)] backdrop-blur-md"
        style={{
          left: position.x,
          top: position.y,
          width,
          height,
          maxWidth: 'calc(100vw - 12px)',
          maxHeight: 'calc(100vh - 12px)',
          zIndex,
          transform: 'translateZ(0)',
        }}
      >
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-0 z-0 rounded-[inherit] bg-gradient-to-br from-bus-cyan/[0.04] via-transparent to-black/15"
        />
        <div
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={(event) => endDrag(event)}
          onPointerCancel={() => endDrag()}
          className="relative z-10 h-8 shrink-0 flex items-center justify-between px-2.5 border-b border-[#3a8fa3]/28 bg-bus-bg/70 cursor-move select-none touch-none shadow-[0_1px_0_rgb(255_255_255_/05%)_inset]"
        >
          <div className="flex min-w-0 items-center gap-1.5">
            {titleIcon ? (
              <span className="flex h-4 w-4 shrink-0 items-center justify-center" aria-hidden="true">
                {titleIcon}
              </span>
            ) : null}
            <span className="truncate font-mono text-[10px] tracking-wider text-bus-cyan drop-shadow-[0_1px_2px_rgb(0_0_0_/55%)]">
              {title}
            </span>
          </div>
          <button
            type="button"
            aria-label={`Close ${title}`}
            onPointerDown={(event) => event.stopPropagation()}
            onClick={onClose}
            className="w-6 h-6 flex items-center justify-center text-bus-muted hover:text-bus-red hover:bg-white/10 rounded-md"
          >
            <X size={13} />
          </button>
        </div>
        <div className="relative z-10 flex-1 min-h-0 overflow-hidden">{children}</div>
      </section>
    </>
  )
}
