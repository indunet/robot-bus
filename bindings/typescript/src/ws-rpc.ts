/**
 * Multiplexed WebSocket RPC (V2: one connection, many streams).
 *
 * Frame layout matches Rust `src/grpc/ws_frame.rs` (little-endian):
 * - REQUEST: type | stream_id | method_len | method | payload_len | payload
 * - DATA/CANCEL/TRAILER: type | stream_id | …
 */

export const FRAME_REQUEST = 1;
export const FRAME_DATA = 2;
export const FRAME_CANCEL = 3;
export const FRAME_TRAILER = 4;
export const FRAME_PING = 5;
export const FRAME_PONG = 6;

export const WS_BACKOFF_INITIAL_MS = 200;
export const WS_BACKOFF_MAX_MS = 5000;
export const WS_PING_INTERVAL_MS = 3000;
export const WS_PING_MISS_LIMIT = 2;

export const METHOD_SUBSCRIBE =
  "robot_bus_interfaces.grpc.v1.MessageGateway/Subscribe";
export const METHOD_PUBLISH =
  "robot_bus_interfaces.grpc.v1.MessageGateway/Publish";
export const METHOD_CALL = "robot_bus_interfaces.grpc.v1.ServiceGateway/Call";
export const METHOD_SEND_GOAL =
  "robot_bus_interfaces.grpc.v1.ActionGateway/SendGoal";

export type WsFrame =
  | { type: "request"; streamId: number; method: string; payload: Uint8Array }
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

export function encodeFrame(frame: WsFrame): Uint8Array {
  const enc = new TextEncoder();
  switch (frame.type) {
    case "request": {
      const method = enc.encode(frame.method);
      return concatBytes([
        new Uint8Array([FRAME_REQUEST]),
        u32le(frame.streamId),
        u16le(method.length),
        method,
        u32le(frame.payload.length),
        frame.payload,
      ]);
    }
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
    if (bytes.length < 7) throw new Error("truncated REQUEST");
    const methodLen = view.getUint16(5, true);
    const methodStart = 7;
    const methodEnd = methodStart + methodLen;
    if (bytes.length < methodEnd + 4) throw new Error("truncated REQUEST");
    const method = new TextDecoder().decode(bytes.subarray(methodStart, methodEnd));
    const payloadLen = view.getUint32(methodEnd, true);
    const payloadStart = methodEnd + 4;
    const payloadEnd = payloadStart + payloadLen;
    if (bytes.length < payloadEnd) throw new Error("truncated REQUEST payload");
    return {
      type: "request",
      streamId,
      method,
      payload: bytes.subarray(payloadStart, payloadEnd),
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

/** Convert `http(s)://host:port` → `ws(s)://host:port/ws`. */
export function httpUrlToWsRpc(url: string): string {
  const trimmed = url.replace(/\/$/, "");
  if (trimmed.startsWith("ws://") || trimmed.startsWith("wss://")) {
    return trimmed.endsWith("/ws") ? trimmed : `${trimmed}/ws`;
  }
  if (trimmed.startsWith("https://")) {
    return `wss://${trimmed.slice("https://".length)}/ws`;
  }
  if (trimmed.startsWith("http://")) {
    return `ws://${trimmed.slice("http://".length)}/ws`;
  }
  return `ws://${trimmed}/ws`;
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

  async unary(method: string, requestPayload: Uint8Array): Promise<Uint8Array> {
    const ws = await this.ensureSocket();
    const streamId = this.allocStreamId();
    return new Promise<Uint8Array>((resolve, reject) => {
      this.streams.set(streamId, {
        resolve: (data) => {
          if (!data) reject(new Error(`rpc ${method} trailer without DATA`));
          else resolve(data);
        },
        reject,
      });
      ws.send(
        encodeFrame({
          type: "request",
          streamId,
          method,
          payload: requestPayload,
        }),
      );
    });
  }

  async serverStream(
    method: string,
    requestPayload: Uint8Array,
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
        method,
        payload: requestPayload,
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
  method: string,
  requestPayload: Uint8Array,
  _signal?: AbortSignal,
): Promise<Uint8Array> {
  const session = new WsSession(httpBaseUrl);
  try {
    return await session.unary(method, requestPayload);
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
  method: string,
  requestPayload: Uint8Array,
  handlers: ServerStreamHandlers,
  _signal?: AbortSignal,
): Promise<void> {
  const session = new WsSession(httpBaseUrl);
  try {
    const { control, done } = await session.serverStream(
      method,
      requestPayload,
      handlers,
    );
    handlers.onControl?.(control);
    await done;
  } finally {
    session.close();
  }
}
