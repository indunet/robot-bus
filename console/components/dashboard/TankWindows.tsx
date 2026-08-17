'use client'

import { useEffect, useState } from 'react'
import { Bot } from 'lucide-react'
import TankPanel from '@/components/tank/TankPanel'
import { useI18n } from '@/lib/i18n'
import FloatingWindow, { type WindowPosition } from './FloatingWindow'

const WINDOW_WIDTH = 860
const WINDOW_HEIGHT = 560

interface Props {
  open: boolean
  onClose: () => void
}

function centeredPosition(width: number, height: number): WindowPosition {
  const vw = typeof window === 'undefined' ? 1280 : window.innerWidth
  const vh = typeof window === 'undefined' ? 800 : window.innerHeight
  return {
    x: Math.max(0, Math.round((vw - Math.min(width, vw - 12)) / 2)),
    y: Math.max(0, Math.round((vh - Math.min(height, vh - 12)) / 2)),
  }
}

export default function TankWindows({ open, onClose }: Props) {
  const { t } = useI18n()
  const [position, setPosition] = useState(() =>
    centeredPosition(WINDOW_WIDTH, WINDOW_HEIGHT),
  )

  useEffect(() => {
    if (!open) return
    setPosition(centeredPosition(WINDOW_WIDTH, WINDOW_HEIGHT))
  }, [open])

  if (!open) return null

  return (
    <FloatingWindow
      title={t('tankWindowTitle')}
      titleIcon={<Bot size={14} className="text-bus-cyan" strokeWidth={2} />}
      position={position}
      width={WINDOW_WIDTH}
      height={WINDOW_HEIGHT}
      zIndex={50}
      modal={false}
      onPositionChange={setPosition}
      onBringToFront={() => {}}
      onClose={onClose}
    >
      <TankPanel compact autoFocus />
    </FloatingWindow>
  )
}
