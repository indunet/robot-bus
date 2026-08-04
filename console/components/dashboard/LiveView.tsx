'use client'

import { useCallback, useEffect, useRef, useState } from 'react'
import { useI18n } from '@/lib/i18n'
import {
  connectWhep,
  disconnectWhep,
  type DataChannelMessage,
  type WhepSession,
  type WhepStatus,
} from '@/lib/whep-client'

const STORAGE_KEY = 'robot-bus-console-whep-url'
const DEFAULT_WHEP = 'http://127.0.0.1:8090/whep'

export default function LiveView() {
  const { t } = useI18n()
  const [url, setUrl] = useState(DEFAULT_WHEP)
  const [status, setStatus] = useState<WhepStatus>('idle')
  const [error, setError] = useState<string | null>(null)
  const [messages, setMessages] = useState<DataChannelMessage[]>([])
  const sessionRef = useRef<WhepSession | null>(null)
  const videoRef = useRef<HTMLVideoElement | null>(null)

  useEffect(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY)
      if (saved) setUrl(saved)
    } catch {
      /* ignore */
    }
  }, [])

  const persistUrl = useCallback((value: string) => {
    setUrl(value)
    try {
      localStorage.setItem(STORAGE_KEY, value)
    } catch {
      /* ignore */
    }
  }, [])

  const onDisconnect = useCallback(async () => {
    const sess = sessionRef.current
    sessionRef.current = null
    await disconnectWhep(sess)
    if (videoRef.current) {
      videoRef.current.srcObject = null
    }
    setStatus('idle')
    setError(null)
  }, [])

  const onConnect = useCallback(async () => {
    const whepUrl = url.trim()
    if (!whepUrl) return
    await onDisconnect()
    setMessages([])
    setError(null)
    try {
      const session = await connectWhep({
        whepUrl,
        onStatus: (s, err) => {
          setStatus(s)
          if (err) setError(err)
        },
        onData: (msg) => {
          setMessages((prev) => [...prev.slice(-199), msg])
        },
      })
      sessionRef.current = session
      if (videoRef.current) {
        videoRef.current.srcObject = session.stream
        void videoRef.current.play().catch(() => undefined)
      }
    } catch (e) {
      setStatus('failed')
      setError(e instanceof Error ? e.message : String(e))
    }
  }, [url, onDisconnect])

  useEffect(() => {
    return () => {
      void disconnectWhep(sessionRef.current)
      sessionRef.current = null
    }
  }, [])

  const statusLabel =
    status === 'idle'
      ? t('liveStatusIdle')
      : status === 'connecting'
        ? t('liveStatusConnecting')
        : status === 'connected'
          ? t('liveStatusConnected')
          : t('liveStatusFailed')

  return (
    <div className="flex flex-col gap-3 h-full min-h-0" style={{ height: 'calc(100vh - 88px)' }}>
      <div className="flex items-baseline justify-between gap-3 shrink-0">
        <div>
          <h2 className="font-mono text-sm tracking-widest text-bus-text">{t('liveTitle')}</h2>
          <p className="text-xs text-bus-muted mt-0.5">{t('liveSub')}</p>
        </div>
        <span className="font-mono text-[10px] tracking-widest text-bus-muted uppercase">
          {statusLabel}
        </span>
      </div>

      <div className="flex flex-wrap items-center gap-2 shrink-0">
        <label className="font-mono text-[10px] tracking-widest text-bus-muted shrink-0">
          {t('liveWhepUrl')}
        </label>
        <input
          className="flex-1 min-w-[16rem] bg-[#15181c] border border-bus-border rounded px-2 py-1.5 font-mono text-xs text-bus-text outline-none focus:border-bus-cyan/50"
          value={url}
          onChange={(e) => persistUrl(e.target.value)}
          placeholder={DEFAULT_WHEP}
          spellCheck={false}
        />
        <button
          type="button"
          onClick={() => void onConnect()}
          disabled={status === 'connecting'}
          className="px-3 py-1.5 rounded bg-bus-cyan/20 text-bus-cyan font-mono text-xs tracking-widest hover:bg-bus-cyan/30 disabled:opacity-50"
        >
          {t('liveConnect')}
        </button>
        <button
          type="button"
          onClick={() => void onDisconnect()}
          disabled={status === 'idle' || status === 'connecting'}
          className="px-3 py-1.5 rounded border border-bus-border text-bus-muted font-mono text-xs tracking-widest hover:text-bus-text disabled:opacity-50"
        >
          {t('liveDisconnect')}
        </button>
      </div>

      {error && (
        <div className="shrink-0 text-xs text-red-400 font-mono break-all bg-red-500/10 border border-red-500/20 rounded px-2 py-1.5">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-[1.4fr_0.9fr] gap-3 flex-1 min-h-0">
        <div className="bg-[#0e1012] border border-bus-border rounded overflow-hidden flex items-center justify-center min-h-[16rem]">
          <video
            ref={videoRef}
            className="w-full h-full max-h-[calc(100vh-12rem)] object-contain bg-black"
            autoPlay
            playsInline
            controls
          />
        </div>

        <div className="border border-bus-border rounded bg-[#0e1012] flex flex-col min-h-0">
          <div className="px-3 py-2 border-b border-bus-border flex items-baseline justify-between shrink-0">
            <span className="font-mono text-[10px] tracking-widest text-bus-muted">
              {t('liveDataTitle')}
            </span>
            <span className="font-mono text-[10px] text-bus-muted">{messages.length}</span>
          </div>
          <div className="flex-1 overflow-y-auto p-2 font-mono text-[11px] space-y-1.5">
            {messages.length === 0 ? (
              <p className="text-bus-muted px-1 py-2">{t('liveDataEmpty')}</p>
            ) : (
              messages.map((m) => (
                <div key={m.id} className="border-b border-bus-border/40 pb-1.5 last:border-0">
                  <div className="text-bus-cyan/80">
                    {m.label}{' '}
                    <span className="text-bus-muted">
                      {m.byteLength} B · {new Date(m.at).toLocaleTimeString()}
                    </span>
                  </div>
                  <div className="text-bus-text break-all whitespace-pre-wrap">
                    {m.text ?? m.hexPreview}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
