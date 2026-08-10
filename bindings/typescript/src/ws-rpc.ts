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

export const METHOD_SUBSCRIBE =
  "robot_bus_interface.grpc.v1.MessageGateway/Subscribe";
export const METHOD_PUBLISH =
  "robot_bus_interface.grpc.v1.MessageGateway/Publish";
export const METHOD_CALL = "robot_bus_interface.grpc.v1.ServiceGateway/Call";
export const METHOD_SEND_GOAL =
  "robot_bus_interface.grpc.v1.ActionGateway/SendGoal";

export type WsFrame =
  | { type: "request"; streamId: number; method: string; payload: Uint8Array }
  | { type: "data"; streamId: number; payload: Uint8Array }
  | { type: "cancel"; streamId: number }
  | { type: "trailer"; streamId: number; status: number; message: string };

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

/** One multiplexed WebSocket session (shared by a WsNode). */
export class WsSession {
  private ws: WebSocket | null = null;
  private connecting: Promise<WebSocket> | null = null;
  private nextStreamId = 1;
  private readonly streams = new Map<number, StreamHandlers>();
  private closed = false;

  constructor(readonly httpBaseUrl: string) {}

  private allocStreamId(): number {
    const id = this.nextStreamId;
    this.nextStreamId += 2;
    return id;
  }

  private async ensureSocket(): Promise<WebSocket> {
    if (this.closed) throw new Error("websocket session closed");
    if (this.ws && this.ws.readyState === WebSocket.OPEN) return this.ws;
    if (this.connecting) return this.connecting;
    const wsUrl = httpUrlToWsRpc(this.httpBaseUrl);
    this.connecting = new Promise<WebSocket>((resolve, reject) => {
      const ws = new WebSocket(wsUrl);
      ws.binaryType = "arraybuffer";
      ws.onopen = () => {
        this.ws = ws;
        this.connecting = null;
        resolve(ws);
      };
      ws.onerror = () => {
        this.connecting = null;
        reject(new Error(`websocket error connecting to ${wsUrl}`));
      };
      ws.onclose = () => {
        this.ws = null;
        for (const [, h] of this.streams) {
          h.reject?.(new Error("websocket closed"));
        }
        this.streams.clear();
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

  private onBinary(bin: Uint8Array): void {
    let frame: WsFrame;
    try {
      frame = decodeFrame(bin);
    } catch {
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
        handlers.resolve?.(
          (handlers as { _data?: Uint8Array })._data,
        );
      }
    }
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
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(encodeFrame({ type: "cancel", streamId }));
        }
      },
      close: () => {
        this.streams.delete(streamId);
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(encodeFrame({ type: "cancel", streamId }));
        }
        settleDone();
      },
    };
    return { control, done };
  }

  close(): void {
    this.closed = true;
    try {
      this.ws?.close();
    } catch {
      /* ignore */
    }
    this.ws = null;
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
