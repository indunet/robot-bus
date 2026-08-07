/**
 * Browser / gRPC-Web client Node facade.
 *
 * Mirrors Rust/Python `Node.grpc` / `Node.grpc_at`: publish, subscribe, service
 * call, action run. Does not support service/action servers or local broker.
 */

import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import type { RpcOptions } from "@protobuf-ts/runtime-rpc";
import { MessageGatewayClient } from "../generated/robot_bus_interface/grpc/v1/message_gateway.client.js";
import { ServiceGatewayClient } from "../generated/robot_bus_interface/grpc/v1/service_gateway.client.js";
import { ActionGatewayClient } from "../generated/robot_bus_interface/grpc/v1/action_gateway.client.js";
import {
  ActionKind,
  type ActionEvent as PbActionEvent,
} from "../generated/robot_bus_interface/grpc/v1/action_gateway.js";
import { decode, encode, type MessageType } from "./typed.js";

export const DEFAULT_GRPC_URL = "http://127.0.0.1:15770";
const DEFAULT_CONSOLE_URL = "http://127.0.0.1:15771";
const DEFAULT_TOPOLOGY_REFRESH_MS = 10_000;

export interface GrpcNodeOptions {
  /** Console HTTP base URL. `null` disables topology and topic-type registration. */
  consoleUrl?: string | null;
  /** Topology lease refresh interval. Defaults to 10 seconds. */
  topologyRefreshMs?: number;
}

type TopologyKind = "publisher" | "subscriber";

interface TopologyEndpoint {
  endpointId: string;
  kind: TopologyKind;
  topic: string;
}

function defaultConsoleUrl(): string {
  if (typeof globalThis.location !== "undefined" && globalThis.location.origin) {
    return globalThis.location.origin;
  }
  return DEFAULT_CONSOLE_URL;
}

function endpointId(): string {
  if (typeof globalThis.crypto !== "undefined" && "randomUUID" in globalThis.crypto) {
    return globalThis.crypto.randomUUID();
  }
  return `web-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

export interface GrpcActionEvent {
  kind: "GOAL" | "FEEDBACK" | "RESULT" | "CANCEL" | "UNSPECIFIED";
  body: Uint8Array;
  goalId: string;
  actionName: string;
}

export interface GrpcSendGoalOptions<Feedback = GrpcActionEvent> {
  goalId?: string;
  timeoutSeconds?: number;
  onFeedback?: (feedback: Feedback) => void;
}

export class GrpcGoalHandle<Result> {
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

  sendGoal(
    body: Uint8Array,
    options: GrpcSendGoalOptions<GrpcActionEvent> = {},
  ): GrpcGoalHandle<GrpcActionEvent> {
    return this.node.sendGoal(this.actionName, body, options);
  }

  /** @deprecated Prefer `handle.cancel()` on the value returned by `sendGoal()`. */
  async cancel(
    goalId: string,
  ): Promise<void> {
    return this.node.cancelGoal(this.actionName, goalId);
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

  sendGoal(
    goal: Goal,
    options: GrpcSendGoalOptions<Feedback> = {},
  ): GrpcGoalHandle<Result> {
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
    return new GrpcGoalHandle(raw.goalId, raw.actionName, result, () => raw.cancel());
  }
}

type SubCallback = (topic: string, payload: Uint8Array) => void;

/** Raw (bytes) publisher over MessageGateway.Publish. */
export class GrpcTopicPublisher {
  constructor(
    private readonly node: GrpcNode,
    readonly topic: string,
  ) {}

  async publish(payload: Uint8Array): Promise<void> {
    await this.node.publishRaw(this.topic, payload);
  }
}

/** Typed publisher: encodes protobuf then Publish. */
export class TypedGrpcTopicPublisher<T extends object> {
  constructor(
    private readonly inner: GrpcTopicPublisher,
    private readonly msgType: MessageType<T>,
  ) {}

  get topic(): string {
    return this.inner.topic;
  }

  async publish(message: T): Promise<void> {
    await this.inner.publish(encode(this.msgType, message));
  }
}

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
  private readonly consoleUrl: string | null;
  private readonly topologyRefreshMs: number;
  private readonly topologyEndpoints = new Map<string, TopologyEndpoint>();
  private readonly topicTypes = new Map<string, string>();
  private topologyTimer: ReturnType<typeof setInterval> | null = null;
  private topologyStarted = false;
  private abort: AbortController | null = null;
  private readonly actionAborts = new Map<
    string,
    { actionName: string; controller: AbortController }
  >();
  private running = false;

  private constructor(name: string, url: string, options: GrpcNodeOptions = {}) {
    this.name = name;
    this.url = url.replace(/\/$/, "");
    this.consoleUrl =
      options.consoleUrl === null
        ? null
        : (options.consoleUrl ?? defaultConsoleUrl()).replace(/\/$/, "");
    this.topologyRefreshMs = Math.max(100, options.topologyRefreshMs ?? DEFAULT_TOPOLOGY_REFRESH_MS);
    this.transport = new GrpcWebFetchTransport({
      baseUrl: this.url,
      format: "binary",
    });
    this.messageClient = new MessageGatewayClient(this.transport);
    this.serviceClient = new ServiceGatewayClient(this.transport);
    this.actionClient = new ActionGatewayClient(this.transport);
  }

  static grpc(name: string, options?: GrpcNodeOptions): GrpcNode {
    return new GrpcNode(name, DEFAULT_GRPC_URL, options);
  }

  static grpcAt(name: string, url: string, options?: GrpcNodeOptions): GrpcNode {
    return new GrpcNode(name, url, options);
  }

  createPublisher(topic: string): GrpcTopicPublisher;
  createPublisher<T extends object>(
    topic: string,
    msgType: MessageType<T>,
  ): TypedGrpcTopicPublisher<T>;
  createPublisher<T extends object>(
    topic: string,
    msgType?: MessageType<T>,
  ): GrpcTopicPublisher | TypedGrpcTopicPublisher<T> {
    const raw = new GrpcTopicPublisher(this, topic);
    this.trackEndpoint("publisher", topic);
    if (msgType) {
      this.topicTypes.set(topic, msgType.typeName);
      if (this.topologyStarted) this.registerTopicType(topic, msgType.typeName);
      return new TypedGrpcTopicPublisher(raw, msgType);
    }
    return raw;
  }

  createService(_serviceName: string, _handler: unknown): never {
    return unsupported("createService");
  }

  createActionServer(_actionName: string, _handler: unknown): never {
    return unsupported("createActionServer");
  }

  /** Unary Publish onto the message bus. */
  async publishRaw(topic: string, payload: Uint8Array): Promise<void> {
    await this.messageClient.publish({ topic, payload }).response;
  }

  /**
   * Subscribe to a topic prefix. Callbacks fire after `spin()` / `start()` begins.
   */
  createSubscription(topic: string, callback: SubCallback): void;
  createSubscription<T extends object>(
    topic: string,
    callback: (topic: string, msg: T) => void,
    msgType: MessageType<T>,
  ): void;
  createSubscription<T extends object>(
    topic: string,
    callback: SubCallback | ((topic: string, msg: T) => void),
    msgType?: MessageType<T>,
  ): void {
    const wrapped: SubCallback = msgType
      ? (t, payload) => {
          const decoded = decode(msgType, payload);
          if (decoded) {
            (callback as (topic: string, msg: T) => void)(t, decoded);
          }
        }
      : (callback as SubCallback);
    const list = this.subscriptions.get(topic) ?? [];
    list.push(wrapped);
    this.subscriptions.set(topic, list);
    this.trackEndpoint("subscriber", topic);
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

  sendGoal(
    actionName: string,
    body: Uint8Array,
    options: GrpcSendGoalOptions<GrpcActionEvent> = {},
  ): GrpcGoalHandle<GrpcActionEvent> {
    const timeoutMs =
      options.timeoutSeconds === undefined
        ? 0
        : Math.max(0, Math.round(options.timeoutSeconds * 1000));
    const id =
      options.goalId ??
      (typeof globalThis.crypto !== "undefined" && "randomUUID" in globalThis.crypto
        ? globalThis.crypto.randomUUID().replace(/-/g, "")
        : `goal-${Date.now()}`);
    if (this.actionAborts.has(id)) {
      throw new Error(`action goal '${id}' is already active`);
    }

    const controller = new AbortController();
    this.actionAborts.set(id, { actionName, controller });
    const result = (async (): Promise<GrpcActionEvent> => {
      try {
        const call = this.actionClient.sendGoal(
          {
            actionName,
            goal: body,
            goalId: id,
            timeoutMs,
          },
          { abort: controller.signal } as RpcOptions,
        );

        for await (const ev of call.responses) {
          const event = mapEvent(ev);
          if (ev.kind === ActionKind.FEEDBACK) {
            try {
              options.onFeedback?.(event);
            } catch (err) {
              console.error(`robot-bus action '${actionName}' feedback callback error`, err);
            }
          }
          if (ev.kind === ActionKind.RESULT) {
            return event;
          }
        }
        throw new Error(`action '${actionName}' goal '${id}' completed without a result`);
      } finally {
        if (this.actionAborts.get(id)?.controller === controller) {
          this.actionAborts.delete(id);
        }
      }
    })();

    return new GrpcGoalHandle(id, actionName, result, () =>
      this.cancelGoal(actionName, id)
    );
  }

  async cancelGoal(
    actionName: string,
    goalId: string,
  ): Promise<void> {
    const active = this.actionAborts.get(goalId);
    if (!active || active.actionName !== actionName) {
      throw new Error(`action '${actionName}' has no active goal '${goalId}'`);
    }
    active.controller.abort();
  }

  /** Start background subscribe streams (non-blocking). */
  start(): void {
    if (this.running) return;
    this.running = true;
    this.startTopologyRegistration();
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
    for (const { controller } of this.actionAborts.values()) {
      controller.abort();
    }
    this.actionAborts.clear();
    this.stopTopologyRegistration();
  }

  private trackEndpoint(kind: TopologyKind, topic: string): void {
    const key = `${kind}:${topic}`;
    if (this.topologyEndpoints.has(key)) return;
    const endpoint = { endpointId: endpointId(), kind, topic };
    this.topologyEndpoints.set(key, endpoint);
    if (this.topologyStarted) this.registerEndpoint(endpoint);
  }

  private startTopologyRegistration(): void {
    if (this.topologyStarted || !this.consoleUrl || typeof fetch === "undefined") return;
    this.topologyStarted = true;
    this.refreshTopology();
    this.topologyTimer = setInterval(() => this.refreshTopology(), this.topologyRefreshMs);
  }

  private stopTopologyRegistration(): void {
    if (!this.topologyStarted) return;
    this.topologyStarted = false;
    if (this.topologyTimer) clearInterval(this.topologyTimer);
    this.topologyTimer = null;
    for (const endpoint of this.topologyEndpoints.values()) {
      this.postConsole("/api/v1/topology/unregister", {
        endpointId: endpoint.endpointId,
      });
    }
  }

  private refreshTopology(): void {
    for (const endpoint of this.topologyEndpoints.values()) {
      this.registerEndpoint(endpoint);
    }
    for (const [topic, typeName] of this.topicTypes) {
      this.registerTopicType(topic, typeName);
    }
  }

  private registerEndpoint(endpoint: TopologyEndpoint): void {
    this.postConsole("/api/v1/topology/register", {
      endpointId: endpoint.endpointId,
      nodeName: this.name,
      kind: endpoint.kind,
      topic: endpoint.topic,
    });
  }

  private registerTopicType(topic: string, typeName: string): void {
    this.postConsole("/api/v1/topics/register", { topic, typeName });
  }

  private postConsole(path: string, body: object): void {
    if (!this.consoleUrl || typeof fetch === "undefined") return;
    void fetch(`${this.consoleUrl}${path}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
      keepalive: true,
    }).catch(() => {
      // Console introspection is best-effort and must not break message traffic.
    });
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
