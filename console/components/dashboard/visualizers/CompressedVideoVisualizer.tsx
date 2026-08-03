'use client'

import { useEffect, useRef, useState } from 'react'
import { asRecord, num } from '@/lib/visualizers/utils'
import type { VisualizerProps } from '@/lib/visualizers/registry'

/**
 * Best-effort H.264 Annex-B playback via WebCodecs.
 * Falls back to stats when VideoDecoder is unavailable or decode fails.
 */
export default function CompressedVideoVisualizer({ message }: VisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const decoderRef = useRef<VideoDecoder | null>(null)
  const [info, setInfo] = useState('Waiting for video…')
  const [frames, setFrames] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const configuredRef = useRef(false)

  useEffect(() => {
    return () => {
      try {
        decoderRef.current?.close()
      } catch {
        /* ignore */
      }
      decoderRef.current = null
    }
  }, [])

  useEffect(() => {
    const rec = asRecord(message)
    if (!rec) return
    const data = rec.data instanceof Uint8Array ? rec.data : null
    const format = typeof rec.format === 'string' ? rec.format : 'h264'
    if (!data || data.byteLength === 0) return

    setInfo(`${format} · ${data.byteLength} B`)

    if (typeof VideoDecoder === 'undefined') {
      setError('WebCodecs VideoDecoder not available in this browser')
      return
    }

    const canvas = canvasRef.current
    if (!canvas) return

    const ensureDecoder = () => {
      if (decoderRef.current && decoderRef.current.state !== 'closed') return decoderRef.current
      configuredRef.current = false
      const decoder = new VideoDecoder({
        output: (frame) => {
          canvas.width = frame.displayWidth
          canvas.height = frame.displayHeight
          const ctx = canvas.getContext('2d')
          if (ctx) {
            ctx.drawImage(frame, 0, 0)
          }
          frame.close()
          setFrames((n) => n + 1)
          setError(null)
        },
        error: (e) => {
          setError(e.message || 'decode error')
        },
      })
      decoderRef.current = decoder
      return decoder
    }

    try {
      const decoder = ensureDecoder()
      if (!configuredRef.current) {
        const codec = format.toLowerCase().includes('h265') || format.toLowerCase().includes('hevc')
          ? 'hev1.1.6.L93.B0'
          : 'avc1.42E01E'
        decoder.configure({ codec, optimizeForLatency: true })
        configuredRef.current = true
      }

      const chunk = new EncodedVideoChunk({
        type: 'key',
        timestamp: num(asRecord(rec.timestamp)?.sec) * 1e6 + num(asRecord(rec.timestamp)?.nanosec) / 1e3,
        data,
      })
      decoder.decode(chunk)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }, [message])

  return (
    <div className="h-full flex flex-col items-center justify-center gap-2 p-2 bg-black/40">
      <canvas ref={canvasRef} className="max-w-full max-h-[85%] object-contain" />
      <div className="font-mono text-[10px] text-bus-muted text-center">
        <div>{info}</div>
        <div>frames: {frames}</div>
        {error && <div className="text-bus-amber mt-1 max-w-md">{error}</div>}
      </div>
    </div>
  )
}
