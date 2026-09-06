/**
 * Multiplexed WebSocket RPC (V3: one connection, many streams).
 *
 * Frame layout matches Rust `src/ws_gateway/ws_frame.rs` (little-endian):
 * - REQUEST: type | stream_id | opcode | opcode-specific header | body
 * - DATA/CANCEL/TRAILER: type | stream_id | …
 */

export const FRAME_REQUEST = 1;
export const FRAME_DATA = 2;
export const FRAME_CANCEL = 3;
export const FRAME_TRAILER = 4;
export const FRAME_PING = 5;
export const FRAME_PONG = 6;

export const OPCODE_SUBSCRIBE = 1;
export const OPCODE_PUBLISH = 2;
export const OPCODE_CALL = 3;
export const OPCODE_SEND_GOAL = 4;

export const ACTION_KIND_GOAL = 1;
export const ACTION_KIND_FEEDBACK = 2;
export const ACTION_KIND_RESULT = 3;
export const ACTION_KIND_CANCEL = 4;

export const WS_BACKOFF_INITIAL_MS = 200;
export const WS_BACKOFF_MAX_MS = 5000;
export const WS_PING_INTERVAL_MS = 3000;
export const WS_PING_MISS_LIMIT = 2;

/** HTTP path for multiplexed WebSocket RPC on the broker API port. */
export const WS_RPC_PATH = "/ws-rpc";

export type RequestHeader =
  | { opcode: typeof OPCODE_SUBSCRIBE; topic: string; qosDepth: number }
  | { opcode: typeof OPCODE_PUBLISH; topic: string }
  | {
      opcode: typeof OPCODE_CALL;
      serviceName: string;
      timeoutMs: number;
      requestId: string;
    }
  | {
      opcode: typeof OPCODE_SEND_GOAL;
      actionName: string;
      goalId: string;
      timeoutMs: number;
    };

export type WsFrame =
  | { type: "request"; streamId: number; header: RequestHeader; body: Uint8Array }
  | { type: "data"; streamId: number; payload: Uint8Array }
  | { type: "cancel"; streamId: number }
  | { type: "trailer"; streamId: number; status: number; message: string }
  | { type: "ping"; streamId: number }
  | { type: "pong"; streamId: number };

function concatBytes(chunks: Uint8Array[]): Uint8Array {
  let len = 0;
  for (const c of chunks) len += c.length;
  const out = new Uint8Array(len);
  let off = 0;
  for (const c of chunks) {
    out.set(c, off);
    off += c.length;
  }
  return out;
}

function u16le(n: number): Uint8Array {
  const b = new Uint8Array(2);
  new DataView(b.buffer).setUint16(0, n, true);
  return b;
}

function u32le(n: number): Uint8Array {
  const b = new Uint8Array(4);
  new DataView(b.buffer).setUint32(0, n, true);
  return b;
}

function i32le(n: number): Uint8Array {
  const b = new Uint8Array(4);
  new DataView(b.buffer).setInt32(0, n, true);
  return b;
}

function encodeStr(s: string): Uint8Array {
  const enc = new TextEncoder();
  const bytes = enc.encode(s);
  return concatBytes([u16le(bytes.length), bytes]);
}

function encodeRequestHeader(header: RequestHeader, body: Uint8Array): Uint8Array {
  switch (header.opcode) {
    case OPCODE_SUBSCRIBE:
      return concatBytes([
        new Uint8Array([OPCODE_SUBSCRIBE]),
        encodeStr(header.topic),
        i32le(header.qosDepth),
      ]);
    case OPCODE_PUBLISH:
      return concatBytes([
        new Uint8Array([OPCODE_PUBLISH]),
        encodeStr(header.topic),
        body,
      ]);
    case OPCODE_CALL:
      return concatBytes([
        new Uint8Array([OPCODE_CALL]),
        encodeStr(header.serviceName),
        u32le(header.timeoutMs),
        encodeStr(header.requestId),
        body,
      ]);
    case OPCODE_SEND_GOAL:
      return concatBytes([
        new Uint8Array([OPCODE_SEND_GOAL]),
        encodeStr(header.actionName),
        encodeStr(header.goalId),
        u32le(header.timeoutMs),
        body,
      ]);
  }
}

export function encodeSubscribeData(topic: string, payload: Uint8Array): Uint8Array {
  const enc = new TextEncoder();
  const t = enc.encode(topic);
  return concatBytes([u16le(t.length), t, payload]);
}

export function decodeSubscribeData(payload: Uint8Array): {
  topic: string;
  payload: Uint8Array;
} {
  if (payload.length < 2) throw new Error("truncated subscribe DATA");
  const view = new DataView(payload.buffer, payload.byteOffset, payload.byteLength);
  const topicLen = view.getUint16(0, true);
  if (payload.length < 2 + topicLen) throw new Error("truncated subscribe DATA topic");
  const topic = new TextDecoder().decode(payload.subarray(2, 2 + topicLen));
  return { topic, payload: payload.subarray(2 + topicLen) };
}

export function encodeActionData(kind: number, body: Uint8Array): Uint8Array {
  return concatBytes([new Uint8Array([kind]), body]);
}

export function decodeActionData(payload: Uint8Array): {
  kind: number;
  body: Uint8Array;
} {
  if (payload.length < 1) throw new Error("truncated action DATA");
  return { kind: payload[0]!, body: payload.subarray(1) };
}

export function encodeFrame(frame: WsFrame): Uint8Array {
  const enc = new TextEncoder();
  switch (frame.type) {
    case "request":
      return concatBytes([
        new Uint8Array([FRAME_REQUEST]),
        u32le(frame.streamId),
        encodeRequestHeader(frame.header, frame.body),
      ]);
    case "data":
      return concatBytes([
        new Uint8Array([FRAME_DATA]),
        u32le(frame.streamId),
        u32le(frame.payload.length),
        frame.payload,
      ]);
    case "cancel":
      return concatBytes([
        new Uint8Array([FRAME_CANCEL]),
        u32le(frame.streamId),
        u32le(0),
      ]);
    case "ping":
      return concatBytes([
        new Uint8Array([FRAME_PING]),
        u32le(frame.streamId),
        u32le(0),
      ]);
    case "pong":
      return concatBytes([
        new Uint8Array([FRAME_PONG]),
        u32le(frame.streamId),
        u32le(0),
      ]);
    case "trailer": {
      const msg = enc.encode(frame.message);
      const payload = concatBytes([u32le(frame.status), msg]);
      return concatBytes([
        new Uint8Array([FRAME_TRAILER]),
        u32le(frame.streamId),
        u32le(payload.length),
        payload,
      ]);
    }
  }
}

export function decodeFrame(bytes: Uint8Array): WsFrame {
  if (bytes.length < 5) throw new Error("truncated websocket frame");
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const ty = bytes[0]!;
  const streamId = view.getUint32(1, true);
  if (ty === FRAME_REQUEST) {
    if (bytes.length < 6) throw new Error("truncated REQUEST");
    const opcode = bytes[5]!;
    let off = 6;
    const readStr = (): string => {
      if (bytes.length < off + 2) throw new Error("truncated REQUEST string");
      const len = view.getUint16(off, true);
      off += 2;
      if (bytes.length < off + len) throw new Error("truncated REQUEST string body");
      const s = new TextDecoder().decode(bytes.subarray(off, off + len));
      off += len;
      return s;
    };
    let header: RequestHeader;
    if (opcode === OPCODE_SUBSCRIBE) {
      const topic = readStr();
      if (bytes.length < off + 4) throw new Error("truncated Subscribe qos");
      const qosDepth = view.getInt32(off, true);
      off += 4;
      header = { opcode: OPCODE_SUBSCRIBE, topic, qosDepth };
    } else if (opcode === OPCODE_PUBLISH) {
      header = { opcode: OPCODE_PUBLISH, topic: readStr() };
    } else if (opcode === OPCODE_CALL) {
      const serviceName = readStr();
      if (bytes.length < off + 4) throw new Error("truncated Call timeout");
      const timeoutMs = view.getUint32(off, true);
      off += 4;
      const requestId = readStr();
      header = { opcode: OPCODE_CALL, serviceName, timeoutMs, requestId };
    } else if (opcode === OPCODE_SEND_GOAL) {
      const actionName = readStr();
      const goalId = readStr();
      if (bytes.length < off + 4) throw new Error("truncated SendGoal timeout");
      const timeoutMs = view.getUint32(off, true);
      off += 4;
      header = { opcode: OPCODE_SEND_GOAL, actionName, goalId, timeoutMs };
    } else {
      throw new Error(`unknown opcode ${opcode}`);
    }
    return {
      type: "request",
      streamId,
      header,
      body: bytes.subarray(off),
    };
  }
  if (ty === FRAME_DATA) {
    if (bytes.length < 9) throw new Error("truncated DATA");
    const payloadLen = view.getUint32(5, true);
    const payloadStart = 9;
    const payloadEnd = payloadStart + payloadLen;
    if (bytes.length < payloadEnd) throw new Error("truncated DATA payload");
    return {
      type: "data",
      streamId,
      payload: bytes.subarray(payloadStart, payloadEnd),
    };
  }
  if (ty === FRAME_CANCEL) return { type: "cancel", streamId };
  if (ty === FRAME_PING) return { type: "ping", streamId };
  if (ty === FRAME_PONG) return { type: "pong", streamId };
  if (ty === FRAME_TRAILER) {
    if (bytes.length < 9) throw new Error("truncated TRAILER");
    const payloadLen = view.getUint32(5, true);
    const payloadStart = 9;
    const payloadEnd = payloadStart + payloadLen;
    if (bytes.length < payloadEnd || payloadLen < 4) {
      throw new Error("truncated TRAILER payload");
    }
    const status = view.getUint32(payloadStart, true);
    const message = new TextDecoder().decode(
      bytes.subarray(payloadStart + 4, payloadEnd),
    );
    return { type: "trailer", streamId, status, message };
  }
  throw new Error(`unknown frame type ${ty}`);
}

/** Convert `http(s)://host:port` → `ws(s)://host:port/ws-rpc`. */
export function httpUrlToWsRpc(url: string): string {
  const trimmed = url.replace(/\/$/, "");
  let asWs: string;
  if (trimmed.startsWith("ws://") || trimmed.startsWith("wss://")) {
    asWs = trimmed;
  } else if (trimmed.startsWith("https://")) {
    asWs = `wss://${trimmed.slice("https://".length)}`;
  } else if (trimmed.startsWith("http://")) {
    asWs = `ws://${trimmed.slice("http://".length)}`;
  } else {
    asWs = `ws://${trimmed}`;
  }
  if (asWs.endsWith(WS_RPC_PATH)) return asWs;
  if (asWs.endsWith("/ws")) return `${asWs.slice(0, -"/ws".length)}${WS_RPC_PATH}`;
  return `${asWs}${WS_RPC_PATH}`;
}

export class WsRpcError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message || `rpc status ${status}`);
    this.name = "WsRpcError";
  }
}

type StreamHandlers = {
  onData?: (payload: Uint8Array) => void;
  onTrailer?: (status: number, message: string) => void;
  resolve?: (payload: Uint8Array | undefined) => void;
  reject?: (err: Error) => void;
};

export type WsSessionEvent = "connecting" | "open" | "reconnecting" | "close";

export type WsSessionOptions = {
  backoffInitialMs?: number;
  backoffMaxMs?: number;
  pingIntervalMs?: number;
  pingMissLimit?: number;
  webSocket?: { new (url: string): WebSocket };
};

let websocketCtor: { new (url: string): WebSocket } | undefined;

/** Test-only: inject a WebSocket constructor. Pass `undefined` to restore. */
export function __setWebSocketForTests(
  ctor?: { new (url: string): WebSocket },
): void {
  websocketCtor = ctor;
}

export type WsConnectionListener = (event: WsSessionEvent, reason: string) => void;

/** One multiplexed WebSocket session (shared by a WsNode). */
export class WsSession {
  private ws: WebSocket | null = null;
  private connecting: Promise<WebSocket> | null = null;
  private nextStreamId = 1;
  private readonly streams = new Map<number, StreamHandlers>();
  private closed = false;
  private loopStarted = false;
  private backoffMs: number;
  private readonly backoffInitialMs: number;
  private readonly backoffMaxMs: number;
  private readonly pingIntervalMs: number;
  private readonly pingMissLimit: number;
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  private awaitingPong = false;
  private pingMisses = 0;
  private heartbeat = true;
  private readonly listeners: WsConnectionListener[] = [];
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private readonly ctor: { new (url: string): WebSocket };

  constructor(
    readonly httpBaseUrl: string,
    options: WsSessionOptions = {},
  ) {
    this.backoffInitialMs = options.backoffInitialMs ?? WS_BACKOFF_INITIAL_MS;
    this.backoffMaxMs = options.backoffMaxMs ?? WS_BACKOFF_MAX_MS;
    this.pingIntervalMs = options.pingIntervalMs ?? WS_PING_INTERVAL_MS;
    this.pingMissLimit = options.pingMissLimit ?? WS_PING_MISS_LIMIT;
    this.backoffMs = this.backoffInitialMs;
    this.ctor =
      options.webSocket ??
      websocketCtor ??
      (globalThis.WebSocket as { new (url: string): WebSocket });
  }

  /** Begin (or resume) the reconnect loop. Idempotent. */
  start(): void {
    this.startLoop();
  }

  onConnection(listener: WsConnectionListener): () => void {
    this.listeners.push(listener);
    return () => {
      const i = this.listeners.indexOf(listener);
      if (i >= 0) this.listeners.splice(i, 1);
    };
  }

  private emit(event: WsSessionEvent, reason: string): void {
    for (const cb of this.listeners) {
      try {
        cb(event, reason);
      } catch (err) {
        console.error("robot-bus ws session event error", err);
      }
    }
  }

  private socketOpen(): boolean {
    return this.ws !== null && this.ws.readyState === 1;
  }

  private startLoop(): void {
    if (this.loopStarted || this.closed) return;
    this.loopStarted = true;
    void this.connectLoop();
  }

  private async connectLoop(): Promise<void> {
    while (!this.closed) {
      this.emit("connecting", "ws connect");
      try {
        await this.openOnce();
        this.backoffMs = this.backoffInitialMs;
        this.emit("open", "ws open");
        await this.waitUntilSocketClosed();
        if (this.closed) break;
        this.failStreams(new Error("websocket closed"));
        this.emit("reconnecting", "ws closed");
      } catch (err) {
        if (this.closed) break;
        this.emit("reconnecting", err instanceof Error ? err.message : "ws connect failed");
      }
      if (this.closed) break;
      await this.sleepBackoff();
      this.backoffMs = Math.min(this.backoffMs * 2, this.backoffMaxMs);
    }
    this.emit("close", "session closed");
  }

  private sleepBackoff(): Promise<void> {
    return new Promise((resolve) => {
      this.reconnectTimer = setTimeout(() => {
        this.reconnectTimer = null;
        resolve();
      }, this.backoffMs);
    });
  }

  private openOnce(): Promise<WebSocket> {
    if (this.connecting) return this.connecting;
    const wsUrl = httpUrlToWsRpc(this.httpBaseUrl);
    this.connecting = new Promise<WebSocket>((resolve, reject) => {
      const ws = new this.ctor(wsUrl);
      ws.binaryType = "arraybuffer";
      const settleOpen = () => {
        this.ws = ws;
        this.connecting = null;
        this.startHeartbeat(ws);
        resolve(ws);
      };
      const settleFail = (err: Error) => {
        this.connecting = null;
        this.stopHeartbeat();
        this.ws = null;
        reject(err);
      };
      ws.onopen = () => settleOpen();
      ws.onerror = () => {
        if (!this.socketOpen()) {
          settleFail(new Error(`websocket error connecting to ${wsUrl}`));
        }
      };
      ws.onclose = () => {
        this.stopHeartbeat();
        const pendingConnect = this.connecting !== null && this.ws !== ws;
        this.ws = null;
        if (pendingConnect) {
          settleFail(new Error("websocket closed during connect"));
        }
        this.socketClosedWaiter?.();
        this.socketClosedWaiter = null;
      };
      ws.onmessage = (ev) => {
        const bin =
          ev.data instanceof ArrayBuffer
            ? new Uint8Array(ev.data)
            : ev.data instanceof Blob
              ? null
              : null;
        if (bin) {
          this.onBinary(bin);
          return;
        }
        if (ev.data instanceof Blob) {
          void ev.data.arrayBuffer().then((buf) => this.onBinary(new Uint8Array(buf)));
        }
      };
    });
    return this.connecting;
  }

  private socketClosedWaiter: (() => void) | null = null;

  private waitUntilSocketClosed(): Promise<void> {
    if (!this.socketOpen()) return Promise.resolve();
    return new Promise((resolve) => {
      this.socketClosedWaiter = resolve;
    });
  }

  private startHeartbeat(ws: WebSocket): void {
    this.stopHeartbeat();
    this.heartbeat = true;
    this.awaitingPong = false;
    this.pingMisses = 0;
    this.pingTimer = setInterval(() => {
      if (!this.heartbeat || ws.readyState !== 1) return;
      if (this.awaitingPong) {
        this.pingMisses += 1;
        if (this.pingMisses >= this.pingMissLimit) {
          try {
            ws.close();
          } catch {
            /* ignore */
          }
          return;
        }
      }
      this.awaitingPong = true;
      try {
        ws.send(encodeFrame({ type: "ping", streamId: 0 }));
      } catch {
        try {
          ws.close();
        } catch {
          /* ignore */
        }
      }
    }, this.pingIntervalMs);
  }

  private stopHeartbeat(): void {
    if (this.pingTimer !== null) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  private failStreams(err: Error): void {
    for (const [, h] of this.streams) {
      h.reject?.(err);
    }
    this.streams.clear();
  }

  private async ensureSocket(): Promise<WebSocket> {
    if (this.closed) throw new Error("websocket session closed");
    if (this.socketOpen() && this.ws) return this.ws;
    this.startLoop();
    if (this.connecting) return this.connecting;
    return new Promise<WebSocket>((resolve, reject) => {
      const unsub = this.onConnection((event, reason) => {
        if (event === "open" && this.ws && this.socketOpen()) {
          unsub();
          resolve(this.ws);
        } else if (this.closed) {
          unsub();
          reject(new Error("websocket session closed"));
        } else if (event === "close") {
          unsub();
          reject(new Error(reason));
        }
      });
    });
  }

  private onBinary(bin: Uint8Array): void {
    let frame: WsFrame;
    try {
      frame = decodeFrame(bin);
    } catch {
      return;
    }
    if (frame.type === "pong") {
      this.awaitingPong = false;
      this.pingMisses = 0;
      return;
    }
    if (frame.type === "ping") {
      if (this.socketOpen() && this.ws) {
        try {
          this.ws.send(encodeFrame({ type: "pong", streamId: frame.streamId }));
        } catch {
          /* ignore */
        }
      }
      return;
    }
    if (frame.type === "trailer" && frame.streamId === 0) {
      this.heartbeat = false;
      this.awaitingPong = false;
      return;
    }
    const handlers = this.streams.get(frame.streamId);
    if (!handlers) return;
    if (frame.type === "data") {
      handlers.onData?.(frame.payload);
      (handlers as { _data?: Uint8Array })._data = frame.payload;
    } else if (frame.type === "trailer") {
      this.streams.delete(frame.streamId);
      handlers.onTrailer?.(frame.status, frame.message);
      if (frame.status !== 0) {
        handlers.reject?.(new WsRpcError(frame.status, frame.message));
      } else {
        handlers.resolve?.((handlers as { _data?: Uint8Array })._data);
      }
    }
  }

  async waitUntilOpen(timeoutMs?: number): Promise<boolean> {
    if (this.closed) return false;
    if (this.socketOpen()) return true;
    this.startLoop();
    return new Promise((resolve) => {
      const timer =
        timeoutMs === undefined
          ? undefined
          : setTimeout(() => {
              unsub();
              resolve(false);
            }, timeoutMs);
      const unsub = this.onConnection((event) => {
        if (event === "open") {
          if (timer) clearTimeout(timer);
          unsub();
          resolve(true);
        } else if (this.closed || event === "close") {
          if (timer) clearTimeout(timer);
          unsub();
          resolve(false);
        }
      });
    });
  }

  async unary(header: RequestHeader, requestBody: Uint8Array): Promise<Uint8Array> {
    const ws = await this.ensureSocket();
    const streamId = this.allocStreamId();
    return new Promise<Uint8Array>((resolve, reject) => {
      this.streams.set(streamId, {
        resolve: (data) => resolve(data ?? new Uint8Array()),
        reject,
      });
      ws.send(
        encodeFrame({
          type: "request",
          streamId,
          header,
          body: requestBody,
        }),
      );
    });
  }

  async serverStream(
    header: RequestHeader,
    requestBody: Uint8Array,
    handlers: {
      onData: (payload: Uint8Array) => void;
      onTrailer?: (status: number, message: string) => void;
    },
  ): Promise<{
    control: WsServerStreamControl;
    done: Promise<void>;
  }> {
    const ws = await this.ensureSocket();
    const streamId = this.allocStreamId();
    let settleDone!: () => void;
    let settleErr!: (err: Error) => void;
    const done = new Promise<void>((resolve, reject) => {
      settleDone = resolve;
      settleErr = reject;
    });
    this.streams.set(streamId, {
      onData: handlers.onData,
      onTrailer: (status, message) => {
        handlers.onTrailer?.(status, message);
        if (status !== 0) {
          settleErr(new WsRpcError(status, message));
        } else {
          settleDone();
        }
      },
      reject: (err) => settleErr(err),
      resolve: () => settleDone(),
    });
    ws.send(
      encodeFrame({
        type: "request",
        streamId,
        header,
        body: requestBody,
      }),
    );
    const control: WsServerStreamControl = {
      cancel: () => {
        if (ws.readyState === 1) {
          ws.send(encodeFrame({ type: "cancel", streamId }));
        }
      },
      close: () => {
        this.streams.delete(streamId);
        if (ws.readyState === 1) {
          ws.send(encodeFrame({ type: "cancel", streamId }));
        }
        settleDone();
      },
    };
    return { control, done };
  }

  close(): void {
    this.closed = true;
    this.stopHeartbeat();
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.failStreams(new Error("websocket session closed"));
    try {
      this.ws?.close();
    } catch {
      /* ignore */
    }
    this.ws = null;
    this.socketClosedWaiter?.();
    this.socketClosedWaiter = null;
  }

  private allocStreamId(): number {
    const id = this.nextStreamId;
    this.nextStreamId += 2;
    return id;
  }
}

export type WsServerStreamControl = {
  cancel: () => void;
  close: () => void;
};

/** @deprecated Prefer WsSession; kept for tests that open one-shot RPCs. */
export async function wsUnary(
  httpBaseUrl: string,
  header: RequestHeader,
  requestBody: Uint8Array,
  _signal?: AbortSignal,
): Promise<Uint8Array> {
  const session = new WsSession(httpBaseUrl);
  try {
    return await session.unary(header, requestBody);
  } finally {
    session.close();
  }
}

export type ServerStreamHandlers = {
  onData: (payload: Uint8Array) => void;
  onTrailer?: (status: number, message: string) => void;
  onControl?: (control: WsServerStreamControl) => void;
};

/** @deprecated Prefer WsSession.serverStream. */
export async function wsServerStream(
  httpBaseUrl: string,
  header: RequestHeader,
  requestBody: Uint8Array,
  handlers: ServerStreamHandlers,
  _signal?: AbortSignal,
): Promise<void> {
  const session = new WsSession(httpBaseUrl);
  try {
    const { control, done } = await session.serverStream(
      header,
      requestBody,
      handlers,
    );
    handlers.onControl?.(control);
    await done;
  } finally {
    session.close();
  }
}
