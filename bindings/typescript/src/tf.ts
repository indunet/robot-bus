/**
 * Typed TF helpers (Node.js / napi only). Browser entry does not export these.
 */

import type {
  TfBuffer as NativeTfBuffer,
  TopicPublisher as NativeTopicPublisher,
} from "./native-types.js";
import { decode, encode, type MessageType } from "./typed.js";

/** Typed facade over the native TF buffer (protobuf in/out). */
export class TfBuffer {
  /** @internal */
  constructor(readonly native: NativeTfBuffer) {}

  clear(): void {
    this.native.clear();
  }

  /** Ingest raw `TFMessage` bytes. */
  setTransformMsgBytes(data: Buffer, isStatic: boolean): void {
    this.native.setTransformMsg(data, isStatic);
  }

  /** Ingest a typed `TFMessage` instance. */
  setTransformMsg<T extends object>(
    msgType: MessageType<T>,
    message: T,
    isStatic: boolean,
  ): void {
    this.native.setTransformMsg(
      Buffer.from(encode(msgType, message)),
      isStatic,
    );
  }

  /** Lookup as raw `TransformStamped` bytes. */
  lookupTransformBytes(target: string, source: string): Buffer {
    return this.native.lookupTransform(target, source);
  }

  /** Lookup and decode as typed `TransformStamped`. */
  lookupTransform<T extends object>(
    target: string,
    source: string,
    msgType: MessageType<T>,
  ): T {
    const decoded = decode(msgType, this.native.lookupTransform(target, source));
    if (!decoded) {
      throw new Error("TransformStamped decode failed");
    }
    return decoded;
  }

  canTransform(target: string, source: string): boolean {
    return this.native.canTransform(target, source);
  }

  frames(): string[] {
    return this.native.frames();
  }
}

/** Thin helper over a typed or raw `TFMessage` publisher. */
export class TransformBroadcaster<T extends object> {
  constructor(
    private readonly publisher: {
      publish(message: T | Buffer): void;
    },
    private readonly msgType?: MessageType<T>,
  ) {}

  static fromRaw(publisher: NativeTopicPublisher): TransformBroadcaster<object> {
    return new TransformBroadcaster({
      publish: (payload) => {
        if (!Buffer.isBuffer(payload)) {
          throw new Error("raw TransformBroadcaster expects Buffer");
        }
        publisher.publish(payload);
      },
    });
  }

  static fromTyped<T extends object>(
    publisher: { publish(message: T): void },
    msgType: MessageType<T>,
  ): TransformBroadcaster<T> {
    return new TransformBroadcaster(publisher, msgType);
  }

  send(message: T): void {
    if (this.msgType) {
      this.publisher.publish(message);
      return;
    }
    throw new Error("TransformBroadcaster.send(message) requires a typed publisher");
  }

  sendBytes(payload: Buffer): void {
    (this.publisher as { publish(p: Buffer): void }).publish(payload);
  }
}
