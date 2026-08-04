/** Minimal WHEP client for Console LIVE (recvonly). */

export type WhepStatus = 'idle' | 'connecting' | 'connected' | 'failed'

export interface DataChannelMessage {
  id: number
  label: string
  at: number
  /** UTF-8 text when decodable; otherwise null. */
  text: string | null
  byteLength: number
  hexPreview: string
}

export interface WhepSession {
  pc: RTCPeerConnection
  resourceUrl: string | null
  stream: MediaStream
}

export interface ConnectWhepOptions {
  whepUrl: string
  onStatus?: (status: WhepStatus, error?: string) => void
  onData?: (msg: DataChannelMessage) => void
}

let msgSeq = 0

function toHexPreview(buf: ArrayBuffer, max = 32): string {
  const bytes = new Uint8Array(buf)
  const n = Math.min(bytes.length, max)
  let out = ''
  for (let i = 0; i < n; i++) {
    out += bytes[i]!.toString(16).padStart(2, '0')
    if (i + 1 < n) out += ' '
  }
  if (bytes.length > max) out += '…'
  return out
}

function tryUtf8(buf: ArrayBuffer): string | null {
  try {
    const text = new TextDecoder('utf-8', { fatal: true }).decode(buf)
    // Prefer printable-ish payloads for the log panel.
    if (/^[\x09\x0a\x0d\x20-\x7e\u0080-\uFFFF]*$/.test(text)) return text
    return null
  } catch {
    return null
  }
}

async function waitIceComplete(pc: RTCPeerConnection): Promise<void> {
  if (pc.iceGatheringState === 'complete') return
  await new Promise<void>((resolve) => {
    const check = () => {
      if (pc.iceGatheringState === 'complete') {
        pc.removeEventListener('icegatheringstatechange', check)
        resolve()
      }
    }
    pc.addEventListener('icegatheringstatechange', check)
  })
}

function resolveResourceUrl(whepUrl: string, locationHeader: string | null): string | null {
  if (!locationHeader) return null
  try {
    return new URL(locationHeader, whepUrl).toString()
  } catch {
    return locationHeader
  }
}

export async function connectWhep(opts: ConnectWhepOptions): Promise<WhepSession> {
  const { whepUrl, onStatus, onData } = opts
  onStatus?.('connecting')

  const pc = new RTCPeerConnection({ iceServers: [] })
  const stream = new MediaStream()

  pc.ontrack = (ev) => {
    for (const track of ev.streams[0]?.getTracks() ?? []) {
      if (!stream.getTracks().some((t) => t.id === track.id)) {
        stream.addTrack(track)
      }
    }
    if (ev.track && !stream.getTracks().some((t) => t.id === ev.track.id)) {
      stream.addTrack(ev.track)
    }
  }

  pc.ondatachannel = (ev) => {
    const dc = ev.channel
    dc.binaryType = 'arraybuffer'
    dc.onmessage = (m) => {
      let bytes: Uint8Array
      if (typeof m.data === 'string') {
        bytes = new TextEncoder().encode(m.data)
      } else if (m.data instanceof ArrayBuffer) {
        bytes = new Uint8Array(m.data)
      } else if (ArrayBuffer.isView(m.data)) {
        bytes = new Uint8Array(m.data.buffer, m.data.byteOffset, m.data.byteLength)
      } else {
        return
      }
      const copy = bytes.slice().buffer
      onData?.({
        id: ++msgSeq,
        label: dc.label,
        at: Date.now(),
        text: tryUtf8(copy),
        byteLength: copy.byteLength,
        hexPreview: toHexPreview(copy),
      })
    }
  }

  pc.addTransceiver('video', { direction: 'recvonly' })
  pc.addTransceiver('audio', { direction: 'recvonly' })

  try {
    const offer = await pc.createOffer()
    await pc.setLocalDescription(offer)
    await waitIceComplete(pc)

    const resp = await fetch(whepUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/sdp' },
      body: pc.localDescription?.sdp ?? offer.sdp,
    })
    if (!resp.ok) {
      const text = await resp.text().catch(() => '')
      throw new Error(`WHEP POST ${resp.status}${text ? `: ${text}` : ''}`)
    }

    const resourceUrl = resolveResourceUrl(whepUrl, resp.headers.get('Location'))
    const answerSdp = await resp.text()
    await pc.setRemoteDescription({ type: 'answer', sdp: answerSdp })
    onStatus?.('connected')
    return { pc, resourceUrl, stream }
  } catch (e) {
    pc.close()
    const msg = e instanceof Error ? e.message : String(e)
    onStatus?.('failed', msg)
    throw e
  }
}

export async function disconnectWhep(session: WhepSession | null): Promise<void> {
  if (!session) return
  if (session.resourceUrl) {
    try {
      await fetch(session.resourceUrl, { method: 'DELETE' })
    } catch {
      /* ignore */
    }
  }
  session.pc.close()
  for (const t of session.stream.getTracks()) {
    t.stop()
  }
}
