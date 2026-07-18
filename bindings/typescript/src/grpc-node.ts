/**
 * Browser / gRPC-Web client Node facade.
 *
 * Mirrors Rust/Python `Node.grpc` / `Node.grpc_at`: subscribe, service call,
 * action run. Does not support publish, service/action servers, or local broker.
 */

import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import type { RpcOptions } from "@protobuf-ts/runtime-rpc";
import { MessageGatewayClient } from "../generated/robot_bus/grpc/v1/message_gateway.client.js";
import { ServiceGatewayClient } from "../generated/robot_bus/grpc/v1/service_gateway.client.js";
import { ActionGatewayClient } from "../generated/robot_bus/grpc/v1/action_gateway.client.js";
import {
  ActionKind,
  type ActionEvent as PbActionEvent,
} from "../generated/robot_bus/grpc/v1/action_gateway.js";
import { decode, encode, type MessageType } from "./typed.js";

export const DEFAULT_GRPC_URL = "http://127.0.0.1:15770";

export interface GrpcActionEvent {
  kind: "GOAL" | "FEEDBACK" | "RESULT" | "CANCEL" | "UNSPECIFIED";
  body: Uint8Array;
  goalId: string;
  actionName: string;
}

function kindFromPb(kind: ActionKind): GrpcActionEvent["kind"] {
  switch (kind) {
    case ActionKind.GOAL:
      return "GOAL";
    case ActionKind.FEEDBACK:
      return "FEEDBACK";
    case ActionKind.RESULT:
      return "RESULT";
    case ActionKind.CANCEL:
      return "CANCEL";
    default:
      return "UNSPECIFIED";
  }
}

function unsupported(method: string): never {
  throw new Error(
    `${method} is not available on the browser / gRPC-Web client. ` +
      "Use the Node.js native binding for publish, servers, and local broker.",
  );
}

export class GrpcServiceClient {
  constructor(
    private readonly node: GrpcNode,
    readonly serviceName: string,
  ) {}

  async call(
    body: Uint8Array,
    timeoutSeconds?: number,
    requestId?: string,
  ): Promise<Uint8Array> {
    return this.node.callService(this.serviceName, body, timeoutSeconds, requestId);
  }
}

export class TypedGrpcServiceClient<Req extends object, Res extends object> {
  constructor(
    private readonly inner: GrpcServiceClient,
    private readonly requestType: MessageType<Req>,
    private readonly responseType: MessageType<Res>,
  ) {}

  get serviceName(): string {
    return this.inner.serviceName;
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

export class GrpcActionClient {
  constructor(
    private readonly node: GrpcNode,
    readonly actionName: string,
  ) {}

  async sendGoal(
    body: Uint8Array,
    goalId?: string,
    timeoutSeconds?: number,
  ): Promise<GrpcActionEvent[]> {
    return this.node.sendGoal(this.actionName, body, goalId, timeoutSeconds);
  }

  async cancel(
    goalId: string,
    body: Uint8Array = new Uint8Array(),
  ): Promise<GrpcActionEvent> {
    return this.node.cancelGoal(this.actionName, goalId, body);
  }
}

export class TypedGrpcActionClient<
  Goal extends object,
  Feedback extends object,
  Result extends object,
> {
  constructor(
    private readonly inner: GrpcActionClient,
    private readonly goalType: MessageType<Goal>,
    private readonly feedbackType: MessageType<Feedback>,
    private readonly resultType: MessageType<Result>,
  ) {}

  get actionName(): string {
    return this.inner.actionName;
  }

  async sendGoal(
    goal: Goal,
    goalId?: string,
    timeoutSeconds?: number,
  ): Promise<{
    events: GrpcActionEvent[];
    feedback: Feedback[];
    result: Result | null;
  }> {
    const events = await this.inner.sendGoal(
      encode(this.goalType, goal),
      goalId,
      timeoutSeconds,
    );
    const feedback: Feedback[] = [];
    let result: Result | null = null;
    for (const ev of events) {
      if (ev.kind === "FEEDBACK") {
        const fb = decode(this.feedbackType, ev.body);
        if (fb) feedback.push(fb);
      } else if (ev.kind === "RESULT") {
        result = decode(this.resultType, ev.body);
      }
    }
    return { events, feedback, result };
  }
}

type SubCallback = (topic: string, payload: Uint8Array) => void;

/**
 * gRPC-Web client node (browser + Node without native addon).
 */
export class GrpcNode {
  readonly name: string;
  readonly url: string;
  private readonly transport: GrpcWebFetchTransport;
  private readonly messageClient: MessageGatewayClient;
  private readonly serviceClient: ServiceGatewayClient;
  private readonly actionClient: ActionGatewayClient;
  private readonly subscriptions = new Map<string, SubCallback[]>();
  private abort: AbortController | null = null;
  private running = false;

  private constructor(name: string, url: string) {
    this.name = name;
    this.url = url.replace(/\/$/, "");
    this.transport = new GrpcWebFetchTransport({
      baseUrl: this.url,
      format: "binary",
    });
    this.messageClient = new MessageGatewayClient(this.transport);
    this.serviceClient = new ServiceGatewayClient(this.transport);
    this.actionClient = new ActionGatewayClient(this.transport);
  }

  static grpc(name: string): GrpcNode {
    return new GrpcNode(name, DEFAULT_GRPC_URL);
  }

  static grpcAt(name: string, url: string): GrpcNode {
    return new GrpcNode(name, url);
  }

  createPublisher(_topic: string): never {
    return unsupported("createPublisher");
  }

  createService(_serviceName: string, _handler: unknown): never {
    return unsupported("createService");
  }

  createActionServer(_actionName: string, _handler: unknown): never {
    return unsupported("createActionServer");
  }

  /**
   * Subscribe to a topic prefix. Callbacks fire after `spin()` / `start()` begins.
   */
  createSubscription(
    topic: string,
    callback: SubCallback,
    msgType?: MessageType<object>,
  ): void {
    const wrapped: SubCallback = msgType
      ? (t, payload) => {
          const decoded = decode(msgType, payload);
          if (decoded) {
            (callback as (topic: string, msg: object) => void)(t, decoded);
          }
        }
      : callback;
    const list = this.subscriptions.get(topic) ?? [];
    list.push(wrapped);
    this.subscriptions.set(topic, list);
  }

  createClient(serviceName: string): GrpcServiceClient;
  createClient<Req extends object, Res extends object>(
    serviceName: string,
    requestType: MessageType<Req>,
    responseType: MessageType<Res>,
  ): TypedGrpcServiceClient<Req, Res>;
  createClient(
    serviceName: string,
    requestType?: MessageType<object>,
    responseType?: MessageType<object>,
  ): GrpcServiceClient | TypedGrpcServiceClient<object, object> {
    const raw = new GrpcServiceClient(this, serviceName);
    if (requestType && responseType) {
      return new TypedGrpcServiceClient(raw, requestType, responseType);
    }
    return raw;
  }

  createActionClient(actionName: string): GrpcActionClient;
  createActionClient<G extends object, F extends object, R extends object>(
    actionName: string,
    goalType: MessageType<G>,
    feedbackType: MessageType<F>,
    resultType: MessageType<R>,
  ): TypedGrpcActionClient<G, F, R>;
  createActionClient(
    actionName: string,
    goalType?: MessageType<object>,
    feedbackType?: MessageType<object>,
    resultType?: MessageType<object>,
  ): GrpcActionClient | TypedGrpcActionClient<object, object, object> {
    const raw = new GrpcActionClient(this, actionName);
    if (goalType && feedbackType && resultType) {
      return new TypedGrpcActionClient(raw, goalType, feedbackType, resultType);
    }
    return raw;
  }

  async callService(
    serviceName: string,
    body: Uint8Array,
    timeoutSeconds?: number,
    requestId?: string,
  ): Promise<Uint8Array> {
    const timeoutMs =
      timeoutSeconds === undefined ? 0 : Math.max(0, Math.round(timeoutSeconds * 1000));
    const call = this.serviceClient.call({
      serviceName,
      request: body,
      requestId: requestId ?? "",
      timeoutMs,
    });
    const response = await call.response;
    return response.response;
  }

  async sendGoal(
    actionName: string,
    body: Uint8Array,
    goalId?: string,
    timeoutSeconds?: number,
  ): Promise<GrpcActionEvent[]> {
    const timeoutMs =
      timeoutSeconds === undefined ? 0 : Math.max(0, Math.round(timeoutSeconds * 1000));
    const call = this.actionClient.run({
      abort: undefined as unknown as AbortSignal,
    } as RpcOptions);

    const id =
      goalId ??
      (typeof crypto !== "undefined" && "randomUUID" in crypto
        ? crypto.randomUUID().replace(/-/g, "")
        : `goal-${Date.now()}`);

    await call.requests.send({
      msg: {
        oneofKind: "goal",
        goal: {
          actionName,
          goal: body,
          goalId: id,
          timeoutMs,
        },
      },
    });
    call.requests.complete();

    const out: GrpcActionEvent[] = [];
    for await (const ev of call.responses) {
      out.push(mapEvent(ev));
      if (ev.kind === ActionKind.RESULT) {
        break;
      }
    }
    return out;
  }

  async cancelGoal(
    actionName: string,
    goalId: string,
    body: Uint8Array = new Uint8Array(),
  ): Promise<GrpcActionEvent> {
    const call = this.actionClient.run({} as RpcOptions);
    await call.requests.send({
      msg: {
        oneofKind: "cancel",
        cancel: {
          actionName,
          goalId,
          body,
        },
      },
    });
    call.requests.complete();
    for await (const ev of call.responses) {
      return mapEvent(ev);
    }
    throw new Error(`action '${actionName}' cancel produced no events`);
  }

  /** Start background subscribe streams (non-blocking). */
  start(): void {
    if (this.running) return;
    this.running = true;
    this.abort = new AbortController();
    for (const topic of this.subscriptions.keys()) {
      void this.pumpTopic(topic, this.abort.signal);
    }
  }

  /** Alias of `start` for API familiarity with ZMQ Node.spin(). */
  spin(): void {
    this.start();
  }

  shutdown(): void {
    this.running = false;
    this.abort?.abort();
    this.abort = null;
  }

  private async pumpTopic(topic: string, signal: AbortSignal): Promise<void> {
    try {
      const call = this.messageClient.subscribe(
        { topic },
        { abort: signal },
      );
      for await (const msg of call.responses) {
        const cbs = this.subscriptions.get(topic) ?? [];
        for (const cb of cbs) {
          try {
            cb(msg.topic, msg.payload);
          } catch (err) {
            console.error("robot-bus subscription callback error", err);
          }
        }
      }
    } catch (err) {
      if (signal.aborted) return;
      console.error(`robot-bus subscribe '${topic}' failed`, err);
    }
  }
}

function mapEvent(ev: PbActionEvent): GrpcActionEvent {
  return {
    kind: kindFromPb(ev.kind),
    body: ev.body,
    goalId: ev.goalId,
    actionName: ev.actionName,
  };
}

/** Browser package entry alias. */
export { GrpcNode as Node };
