/** Shared protobuf message type helpers (typed wrappers). */

export interface ProtoMessage<T = unknown> {
  fromBinary(bytes: Uint8Array): T;
  toBinary(message: T): Uint8Array;
}

/** Minimal message type shape used by typed helpers (@protobuf-ts IMessageType). */
export interface MessageType<T extends object> {
  fromBinary(data: Uint8Array, options?: object): T;
  toBinary(message: T, options?: object): Uint8Array;
  create(value?: Partial<T>): T;
  typeName: string;
}

export function encode<T extends object>(
  msgType: MessageType<T>,
  message: T,
): Uint8Array {
  return msgType.toBinary(message);
}

export function decode<T extends object>(
  msgType: MessageType<T>,
  payload: Uint8Array,
): T | null {
  try {
    return msgType.fromBinary(payload);
  } catch {
    return null;
  }
}
