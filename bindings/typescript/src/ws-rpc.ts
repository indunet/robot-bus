/**
 * Browser gRPC-over-WebSocket (V1: one WebSocket = one RPC).
 *
 * Frame layout matches Rust `src/grpc/ws_frame.rs` (little-endian).
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
  | { type: "request"; method: string; payload: Uint8Array }
  | { type: "data"; payload: Uint8Array }
  | { type: "cancel" }
  | { type: "trailer"; status: number; message: string };

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
        u16le(method.length),
        method,
        u32le(frame.payload.length),
        frame.payload,
      ]);
    }
    case "data":
      return concatBytes([
        new Uint8Array([FRAME_DATA]),
        u32le(frame.payload.length),
        frame.payload,
      ]);
    case "cancel":
      return concatBytes([new Uint8Array([FRAME_CANCEL]), u32le(0)]);
    case "trailer": {
      const msg = enc.encode(frame.message);
      const payload = concatBytes([u32le(frame.status), msg]);
      return concatBytes([
        new Uint8Array([FRAME_TRAILER]),
        u32le(payload.length),
        payload,
      ]);
    }
  }
}

export function decodeFrame(bytes: Uint8Array): WsFrame {
  if (bytes.length < 1) throw new Error("truncated websocket frame");
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const ty = bytes[0]!;
  if (ty === FRAME_REQUEST) {
    if (bytes.length < 3) throw new Error("truncated REQUEST");
    const methodLen = view.getUint16(1, true);
    const methodStart = 3;
    const methodEnd = methodStart + methodLen;
    if (bytes.length < methodEnd + 4) throw new Error("truncated REQUEST");
    const method = new TextDecoder().decode(bytes.subarray(methodStart, methodEnd));
    const payloadLen = view.getUint32(methodEnd, true);
    const payloadStart = methodEnd + 4;
    const payloadEnd = payloadStart + payloadLen;
    if (bytes.length < payloadEnd) throw new Error("truncated REQUEST payload");
    return {
      type: "request",
      method,
      payload: bytes.subarray(payloadStart, payloadEnd),
    };
  }
  if (ty === FRAME_DATA) {
    if (bytes.length < 5) throw new Error("truncated DATA");
    const payloadLen = view.getUint32(1, true);
    const payloadStart = 5;
    const payloadEnd = payloadStart + payloadLen;
    if (bytes.length < payloadEnd) throw new Error("truncated DATA payload");
    return { type: "data", payload: bytes.subarray(payloadStart, payloadEnd) };
  }
  if (ty === FRAME_CANCEL) return { type: "cancel" };
  if (ty === FRAME_TRAILER) {
    if (bytes.length < 5) throw new Error("truncated TRAILER");
    const payloadLen = view.getUint32(1, true);
    const payloadStart = 5;
    const payloadEnd = payloadStart + payloadLen;
    if (bytes.length < payloadEnd || payloadLen < 4) {
      throw new Error("truncated TRAILER payload");
    }
    const status = new DataView(
      bytes.buffer,
      bytes.byteOffset + payloadStart,
      4,
    ).getUint32(0, true);
    const message = new TextDecoder().decode(
      bytes.subarray(payloadStart + 4, payloadEnd),
    );
    return { type: "trailer", status, message };
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

function openSocket(wsUrl: string, signal?: AbortSignal): Promise<WebSocket> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(new DOMException("Aborted", "AbortError"));
      return;
    }
    const ws = new WebSocket(wsUrl);
    ws.binaryType = "arraybuffer";
    const onAbort = () => {
      try {
        ws.close();
      } catch {
        /* ignore */
      }
      reject(new DOMException("Aborted", "AbortError"));
    };
    signal?.addEventListener("abort", onAbort, { once: true });
    ws.onopen = () => {
      signal?.removeEventListener("abort", onAbort);
      resolve(ws);
    };
    ws.onerror = () => {
      signal?.removeEventListener("abort", onAbort);
      reject(new Error(`websocket error connecting to ${wsUrl}`));
    };
  });
}

function waitBinary(ws: WebSocket, signal?: AbortSignal): Promise<Uint8Array | null> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(new DOMException("Aborted", "AbortError"));
      return;
    }
    const onAbort = () => {
      cleanup();
      try {
        ws.close();
      } catch {
        /* ignore */
      }
      reject(new DOMException("Aborted", "AbortError"));
    };
    const onMessage = (ev: MessageEvent) => {
      cleanup();
      if (ev.data instanceof ArrayBuffer) {
        resolve(new Uint8Array(ev.data));
      } else if (ev.data instanceof Blob) {
        void ev.data.arrayBuffer().then((buf) => resolve(new Uint8Array(buf)), reject);
      } else {
        reject(new Error("expected binary websocket message"));
      }
    };
    const onClose = () => {
      cleanup();
      resolve(null);
    };
    const onError = () => {
      cleanup();
      reject(new Error("websocket error"));
    };
    const cleanup = () => {
      signal?.removeEventListener("abort", onAbort);
      ws.removeEventListener("message", onMessage);
      ws.removeEventListener("close", onClose);
      ws.removeEventListener("error", onError);
    };
    signal?.addEventListener("abort", onAbort, { once: true });
    ws.addEventListener("message", onMessage);
    ws.addEventListener("close", onClose);
    ws.addEventListener("error", onError);
  });
}

/** Unary RPC: REQUEST → DATA → TRAILER → close. */
export async function wsUnary(
  httpBaseUrl: string,
  method: string,
  requestPayload: Uint8Array,
  signal?: AbortSignal,
): Promise<Uint8Array> {
  const ws = await openSocket(httpUrlToWsRpc(httpBaseUrl), signal);
  try {
    ws.send(
      encodeFrame({ type: "request", method, payload: requestPayload }),
    );
    let data: Uint8Array | undefined;
    for (;;) {
      const bin = await waitBinary(ws, signal);
      if (!bin) {
        if (data) return data;
        throw new Error(`rpc ${method} closed without response`);
      }
      const frame = decodeFrame(bin);
      if (frame.type === "data") {
        data = frame.payload;
      } else if (frame.type === "trailer") {
        if (frame.status !== 0) {
          throw new WsRpcError(frame.status, frame.message);
        }
        if (!data) throw new Error(`rpc ${method} trailer without DATA`);
        return data;
      }
    }
  } finally {
    try {
      ws.close();
    } catch {
      /* ignore */
    }
  }
}

export type ServerStreamHandlers = {
  onData: (payload: Uint8Array) => void;
  onTrailer?: (status: number, message: string) => void;
  /** Soft cancel (SendGoal): send CANCEL, keep reading until RESULT/trailer. */
  onControl?: (control: WsServerStreamControl) => void;
};

export type WsServerStreamControl = {
  /** Soft cancel: send CANCEL frame; do not close the socket. */
  cancel: () => void;
  /** Hard close: tear down the socket (server treats as disconnect). */
  close: () => void;
};

/**
 * Server-streaming RPC.
 *
 * - Soft `control.cancel()`: CANCEL frame only (action intentional cancel).
 * - Hard `control.close()` / `AbortSignal`: close the socket (true disconnect).
 * - Successful completion does not send CANCEL.
 */
export async function wsServerStream(
  httpBaseUrl: string,
  method: string,
  requestPayload: Uint8Array,
  handlers: ServerStreamHandlers,
  signal?: AbortSignal,
): Promise<void> {
  const ws = await openSocket(httpUrlToWsRpc(httpBaseUrl), signal);
  let closed = false;
  const close = () => {
    if (closed) return;
    closed = true;
    try {
      ws.close();
    } catch {
      /* ignore */
    }
  };
  const cancel = () => {
    try {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(encodeFrame({ type: "cancel" }));
      }
    } catch {
      /* ignore */
    }
  };
  handlers.onControl?.({ cancel, close });
  const onAbort = () => close();
  signal?.addEventListener("abort", onAbort, { once: true });

  try {
    ws.send(
      encodeFrame({ type: "request", method, payload: requestPayload }),
    );
    for (;;) {
      const bin = await waitBinary(ws, signal);
      if (!bin) {
        handlers.onTrailer?.(0, "");
        return;
      }
      const frame = decodeFrame(bin);
      if (frame.type === "data") {
        handlers.onData(frame.payload);
      } else if (frame.type === "trailer") {
        handlers.onTrailer?.(frame.status, frame.message);
        if (frame.status !== 0) {
          throw new WsRpcError(frame.status, frame.message);
        }
        return;
      }
    }
  } finally {
    signal?.removeEventListener("abort", onAbort);
    close();
  }
}
