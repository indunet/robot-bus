'use client'

import { useState } from 'react'
import TurtleSim from '@/components/turtle/TurtleSim'
import TurtleTeleop from '@/components/turtle/TurtleTeleop'
import { useI18n } from '@/lib/i18n'
import FloatingWindow, { type WindowPosition } from './FloatingWindow'

interface Props {
  simOpen: boolean
  teleopOpen: boolean
  onCloseSim: () => void
  onCloseTeleop: () => void
}

function initialSimPosition(): WindowPosition {
  return { x: Math.max(72, (typeof window === 'undefined' ? 1280 : window.innerWidth) - 1060), y: 74 }
}

function initialTeleopPosition(): WindowPosition {
  return { x: Math.max(84, (typeof window === 'undefined' ? 1280 : window.innerWidth) - 350), y: 104 }
}

export default function TurtleWindows({
  simOpen,
  teleopOpen,
  onCloseSim,
  onCloseTeleop,
}: Props) {
  const { t } = useI18n()
  const [simPosition, setSimPosition] = useState(initialSimPosition)
  const [teleopPosition, setTeleopPosition] = useState(initialTeleopPosition)
  const [front, setFront] = useState<'sim' | 'teleop'>('teleop')

  return (
    <>
      {simOpen && (
        <FloatingWindow
          title={t('turtleSimWindowTitle')}
          position={simPosition}
          width={680}
          height={500}
          zIndex={front === 'sim' ? 51 : 50}
          onPositionChange={setSimPosition}
          onBringToFront={() => setFront('sim')}
          onClose={onCloseSim}
        >
          <TurtleSim compact />
        </FloatingWindow>
      )}
      {teleopOpen && (
        <FloatingWindow
          title={t('turtleTeleopWindowTitle')}
          position={teleopPosition}
          width={330}
          height={390}
          zIndex={front === 'teleop' ? 51 : 50}
          onPositionChange={setTeleopPosition}
          onBringToFront={() => setFront('teleop')}
          onClose={onCloseTeleop}
        >
          <TurtleTeleop compact autoFocus />
        </FloatingWindow>
      )}
    </>
  )
}
