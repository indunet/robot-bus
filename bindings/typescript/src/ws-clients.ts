/**
 * WebSocket RPC client handles: publishers, service clients, action clients.
 */

import { decode, encode, type MessageType } from "./typed.js";
import type { WsNode } from "./ws-node.js";

export interface WsActionEvent {
  kind: "GOAL" | "FEEDBACK" | "RESULT" | "CANCEL" | "UNSPECIFIED";
  body: Uint8Array;
  goalId: string;
  actionName: string;
}

export interface WsSendGoalOptions<Feedback = WsActionEvent> {
  goalId?: string;
  timeoutSeconds?: number;
  onFeedback?: (feedback: Feedback) => void;
}

export class WsGoalHandle<Result> {
  constructor(
    readonly goalId: string,
    readonly actionName: string,
    private readonly resultPromise: Promise<Result>,
    private readonly cancelGoal: () => Promise<void>,
  ) {}

  result(): Promise<Result> {
    return this.resultPromise;
  }

  cancel(): Promise<void> {
    return this.cancelGoal();
  }
}

export class WsServiceClient {
  constructor(
    private readonly node: WsNode,
    readonly serviceName: string,
  ) {}

  async serviceIsReady(): Promise<boolean> {
    return this.node.entityHasWorkers("services", this.serviceName);
  }

  async waitForService(timeoutSeconds?: number): Promise<boolean> {
    return this.node.waitUntilWorkers("services", this.serviceName, timeoutSeconds);
  }

  async call(
    body: Uint8Array,
    timeoutSeconds?: number,
    requestId?: string,
  ): Promise<Uint8Array> {
    return this.node.callService(this.serviceName, body, timeoutSeconds, requestId);
  }
}

export class TypedWsServiceClient<Req extends object, Res extends object> {
  constructor(
    private readonly inner: WsServiceClient,
    private readonly requestType: MessageType<Req>,
    private readonly responseType: MessageType<Res>,
  ) {}

  get serviceName(): string {
    return this.inner.serviceName;
  }

  serviceIsReady(): Promise<boolean> {
    return this.inner.serviceIsReady();
  }

  waitForService(timeoutSeconds?: number): Promise<boolean> {
    return this.inner.waitForService(timeoutSeconds);
  }

  async call(request: Req, timeoutSeconds?: number): Promise<Res> {
    const raw = await this.inner.call(encode(this.requestType, request), timeoutSeconds);
    const reply = decode(this.responseType, raw);
    if (!reply) {
      throw new Error(`service ${this.serviceName} response decode failed`);
    }
    return reply;
  }
}

export class WsActionClient {
  constructor(
    private readonly node: WsNode,
    readonly actionName: string,
  ) {}

  async actionServerIsReady(): Promise<boolean> {
    return this.node.entityHasWorkers("actions", this.actionName);
  }

  async waitForActionServer(timeoutSeconds?: number): Promise<boolean> {
    return this.node.waitUntilWorkers("actions", this.actionName, timeoutSeconds);
  }

  sendGoal(
    body: Uint8Array,
    options: WsSendGoalOptions<WsActionEvent> = {},
  ): WsGoalHandle<WsActionEvent> {
    return this.node.sendGoal(this.actionName, body, options);
  }

  /** @deprecated Prefer `handle.cancel()` on the value returned by `sendGoal()`. */
  async cancel(
    goalId: string,
  ): Promise<void> {
    return this.node.cancelGoal(this.actionName, goalId);
  }
}

export class TypedWsActionClient<
  Goal extends object,
  Feedback extends object,
  Result extends object,
> {
  constructor(
    private readonly inner: WsActionClient,
    private readonly goalType: MessageType<Goal>,
    private readonly feedbackType: MessageType<Feedback>,
    private readonly resultType: MessageType<Result>,
  ) {}

  get actionName(): string {
    return this.inner.actionName;
  }

  actionServerIsReady(): Promise<boolean> {
    return this.inner.actionServerIsReady();
  }

  waitForActionServer(timeoutSeconds?: number): Promise<boolean> {
    return this.inner.waitForActionServer(timeoutSeconds);
  }

  sendGoal(
    goal: Goal,
    options: WsSendGoalOptions<Feedback> = {},
  ): WsGoalHandle<Result> {
    const raw = this.inner.sendGoal(encode(this.goalType, goal), {
      goalId: options.goalId,
      timeoutSeconds: options.timeoutSeconds,
      onFeedback: options.onFeedback
        ? (event) => {
            const feedback = decode(this.feedbackType, event.body);
            if (!feedback) {
              throw new Error(`action ${this.actionName} feedback decode failed`);
            }
            options.onFeedback?.(feedback);
          }
        : undefined,
    });
    const result = raw.result().then((event) => {
      const decoded = decode(this.resultType, event.body);
      if (!decoded) {
        throw new Error(`action ${this.actionName} result decode failed`);
      }
      return decoded;
    });
    return new WsGoalHandle(raw.goalId, raw.actionName, result, () => raw.cancel());
  }
}

/** Raw (bytes) publisher over Publish. */
export class WsTopicPublisher {
  constructor(
    private readonly node: WsNode,
    readonly topic: string,
  ) {}

  async publish(payload: Uint8Array): Promise<void> {
    await this.node.publishRaw(this.topic, payload);
  }
}

/** Typed publisher: encodes protobuf then Publish. */
export class TypedWsTopicPublisher<T extends object> {
  constructor(
    private readonly inner: WsTopicPublisher,
    private readonly msgType: MessageType<T>,
  ) {}

  get topic(): string {
    return this.inner.topic;
  }

  async publish(message: T): Promise<void> {
    await this.inner.publish(encode(this.msgType, message));
  }
}
