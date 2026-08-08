'use client'

import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react'
import { createPortal } from 'react-dom'

const SHOW_DELAY_MS = 80

/**
 * Truncated text with a fast hover tip (native `title` waits ~1s).
 * Tip is portaled so overflow:auto parents do not clip it.
 */
export default function TruncateTip({
  text,
  className,
  children,
}: {
  text?: string
  className?: string
  children?: ReactNode
}) {
  const ref = useRef<HTMLSpanElement>(null)
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null)

  const clear = useCallback(() => {
    if (timer.current) {
      clearTimeout(timer.current)
      timer.current = null
    }
    setPos(null)
  }, [])

  const onEnter = useCallback(() => {
    const el = ref.current
    if (!el || !text) return
    // Skip tip when the label fully fits.
    if (el.scrollWidth <= el.clientWidth + 1) return
    const rect = el.getBoundingClientRect()
    timer.current = setTimeout(() => {
      setPos({ left: rect.left, top: rect.top })
    }, SHOW_DELAY_MS)
  }, [text])

  useEffect(() => () => clear(), [clear])

  if (!text) {
    return <span className={className}>{children ?? '—'}</span>
  }

  return (
    <>
      <span
        ref={ref}
        className={`block min-w-0 max-w-full truncate ${className ?? ''}`}
        onMouseEnter={onEnter}
        onMouseLeave={clear}
      >
        {children ?? text}
      </span>
      {pos &&
        typeof document !== 'undefined' &&
        createPortal(
          <span
            role="tooltip"
            className="pointer-events-none fixed z-[100] max-w-[min(28rem,80vw)] break-all rounded-sm border border-bus-border bg-[#1a1d20] px-2 py-1 font-mono text-[11px] leading-snug text-bus-text shadow-lg"
            style={{
              left: Math.min(pos.left, window.innerWidth - 24),
              top: pos.top - 6,
              transform: 'translateY(-100%)',
            }}
          >
            {text}
          </span>,
          document.body,
        )}
    </>
  )
}
