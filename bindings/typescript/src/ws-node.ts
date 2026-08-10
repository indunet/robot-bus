/**
 * Browser WebSocket RPC client Node facade.
 *
 * Mirrors Rust/Python `Node.grpc` / `Node.ws_at`: publish, subscribe, service
 * call, action run. Does not support service/action servers or local broker.
 * Transport is multiplexed WebSocket RPC (`/ws`, one connection many streams).
 */

import {
  ActionEvent as PbActionEvent,
  ActionKind,
  GoalCommand,
} from "../generated/robot_bus_interface/grpc/v1/action_gateway.js";
import {
  SubscribeRequest,
  TopicMessage,
} from "../generated/robot_bus_interface/grpc/v1/message_gateway.js";
import {
  ServiceCallRequest,
  ServiceCallResponse,
} from "../generated/robot_bus_interface/grpc/v1/service_gateway.js";
import { decode, encode, type MessageType } from "./typed.js";
import {
  TOPIC_TYPE_REGISTER,
  TOPOLOGY_REGISTER,
  TOPOLOGY_UNREGISTER,
} from "./console-topics.js";
import {
  TopologyRegister,
  TopologyUnregister,
  TopicTypeRegister,
} from "../generated/robot_bus_interface/msg/v1/console_status.js";
import {
  METHOD_CALL,
  METHOD_PUBLISH,
  METHOD_SEND_GOAL,
  METHOD_SUBSCRIBE,
  WsSession,
} from "./ws-rpc.js";

/** Test-only hook (session factory). */
let sessionFactory: ((url: string) => WsSession) | null = null;

/** Test-only: inject a session factory. Pass `null` to restore. */
export function __setWsRpcForTests(factory?: ((url: string) => WsSession) | null): void {
  sessionFactory = factory ?? null;
}

export const DEFAULT_WS_URL = "http://127.0.0.1:15570";
const DEFAULT_TOPOLOGY_REFRESH_MS = 10_000;

export interface WsNodeOptions {
  /**
   * When `null`, disables topology and topic-type registration.
   * Otherwise registration uses the broker control-plane services via WebSocket
   * RPC (same gateway host as this node). The option name is retained for API compat.
   */
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

function endpointId(): string {
  if (typeof globalThis.crypto !== "undefined" && "randomUUID" in globalThis.crypto) {
    return globalThis.crypto.randomUUID();
  }
  return `web-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

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

function kindFromPb(kind: ActionKind): WsActionEvent["kind"] {
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
    `${method} is not available on the browser / WebSocket RPC client. ` +
      "Use the Node.js native binding for publish, servers, and local broker.",
  );
}

export class WsServiceClient {
  constructor(
    private readonly node: WsNode,
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

export class TypedWsServiceClient<Req extends object, Res extends object> {
  constructor(
    private readonly inner: WsServiceClient,
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

export class WsActionClient {
  constructor(
    private readonly node: WsNode,
    readonly actionName: string,
  ) {}

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

type SubCallback = (topic: string, payload: Uint8Array) => void;

/** Raw (bytes) publisher over MessageGateway.Publish. */
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

/**
 * Browser WebSocket RPC node (browser + Node without native addon).
 */
export class WsNode {
  readonly name: string;
  readonly url: string;
  private readonly subscriptions = new Map<string, SubCallback[]>();
  private readonly topologyEnabled: boolean;
  private readonly topologyRefreshMs: number;
  private readonly topologyEndpoints = new Map<string, TopologyEndpoint>();
  private readonly topicTypes = new Map<string, string>();
  private topologyTimer: ReturnType<typeof setInterval> | null = null;
  private topologyStarted = false;
  private abort: AbortController | null = null;
  private readonly actionSessions = new Map<
    string,
    {
      actionName: string;
      /** Soft CANCEL on the open WebSocket (keep waiting for RESULT). */
      cancel: () => void;
      /** Hard socket close / AbortSignal (true disconnect). */
      close: () => void;
      controller: AbortController;
    }
  >();
  private running = false;
  private readonly session: WsSession;

  private constructor(name: string, url: string, options: WsNodeOptions = {}) {
    this.name = name;
    this.url = url.replace(/\/$/, "");
    this.session = (sessionFactory ?? ((u) => new WsSession(u)))(this.url);
    this.topologyEnabled = options.consoleUrl !== null;
    this.topologyRefreshMs = Math.max(100, options.topologyRefreshMs ?? DEFAULT_TOPOLOGY_REFRESH_MS);
  }

  static ws(name: string, options?: WsNodeOptions): WsNode {
    return new WsNode(name, DEFAULT_WS_URL, options);
  }

  static wsAt(name: string, url: string, options?: WsNodeOptions): WsNode {
    return new WsNode(name, url, options);
  }

  createPublisher(topic: string): WsTopicPublisher;
  createPublisher<T extends object>(
    topic: string,
    msgType: MessageType<T>,
  ): TypedWsTopicPublisher<T>;
  createPublisher<T extends object>(
    topic: string,
    msgType?: MessageType<T>,
  ): WsTopicPublisher | TypedWsTopicPublisher<T> {
    const raw = new WsTopicPublisher(this, topic);
    this.trackEndpoint("publisher", topic);
    if (msgType) {
      this.topicTypes.set(topic, msgType.typeName);
      if (this.topologyStarted) this.registerTopicType(topic, msgType.typeName);
      return new TypedWsTopicPublisher(raw, msgType);
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
    const body = TopicMessage.toBinary(
      TopicMessage.create({ topic, payload }),
    );
    await this.session.unary(METHOD_PUBLISH, body);
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
    if (msgType) {
      this.topicTypes.set(topic, msgType.typeName);
      if (this.topologyStarted) this.registerTopicType(topic, msgType.typeName);
    }
  }

  createClient(serviceName: string): WsServiceClient;
  createClient<Req extends object, Res extends object>(
    serviceName: string,
    requestType: MessageType<Req>,
    responseType: MessageType<Res>,
  ): TypedWsServiceClient<Req, Res>;
  createClient(
    serviceName: string,
    requestType?: MessageType<object>,
    responseType?: MessageType<object>,
  ): WsServiceClient | TypedWsServiceClient<object, object> {
    const raw = new WsServiceClient(this, serviceName);
    if (requestType && responseType) {
      return new TypedWsServiceClient(raw, requestType, responseType);
    }
    return raw;
  }

  createActionClient(actionName: string): WsActionClient;
  createActionClient<G extends object, F extends object, R extends object>(
    actionName: string,
    goalType: MessageType<G>,
    feedbackType: MessageType<F>,
    resultType: MessageType<R>,
  ): TypedWsActionClient<G, F, R>;
  createActionClient(
    actionName: string,
    goalType?: MessageType<object>,
    feedbackType?: MessageType<object>,
    resultType?: MessageType<object>,
  ): WsActionClient | TypedWsActionClient<object, object, object> {
    const raw = new WsActionClient(this, actionName);
    if (goalType && feedbackType && resultType) {
      return new TypedWsActionClient(raw, goalType, feedbackType, resultType);
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
    const req = ServiceCallRequest.toBinary(
      ServiceCallRequest.create({
        serviceName,
        request: body,
        requestId: requestId ?? "",
        timeoutMs,
      }),
    );
    const raw = await this.session.unary(METHOD_CALL, req);
    const response = ServiceCallResponse.fromBinary(raw);
    return response.response;
  }

  sendGoal(
    actionName: string,
    body: Uint8Array,
    options: WsSendGoalOptions<WsActionEvent> = {},
  ): WsGoalHandle<WsActionEvent> {
    const timeoutMs =
      options.timeoutSeconds === undefined
        ? 0
        : Math.max(0, Math.round(options.timeoutSeconds * 1000));
    const id =
      options.goalId ??
      (typeof globalThis.crypto !== "undefined" && "randomUUID" in globalThis.crypto
        ? globalThis.crypto.randomUUID().replace(/-/g, "")
        : `goal-${Date.now()}`);
    if (this.actionSessions.has(id)) {
      throw new Error(`action goal '${id}' is already active`);
    }

    const controller = new AbortController();
    let softCancel = () => {
      /* replaced once the WS stream exposes onControl */
    };
    let pendingSoftCancel = false;
    const session = {
      actionName,
      cancel: () => {
        pendingSoftCancel = true;
        softCancel();
      },
      close: () => controller.abort(),
      controller,
    };
    this.actionSessions.set(id, session);
    const result = (async (): Promise<WsActionEvent> => {
      try {
        const req = GoalCommand.toBinary(
          GoalCommand.create({
            actionName,
            goal: body,
            goalId: id,
            timeoutMs,
          }),
        );
        let resultEvent: WsActionEvent | undefined;
        const { control, done } = await this.session.serverStream(
          METHOD_SEND_GOAL,
          req,
          {
            onData: (payload) => {
              const ev = PbActionEvent.fromBinary(payload);
              const event = mapEvent(ev);
              if (ev.kind === ActionKind.FEEDBACK) {
                try {
                  options.onFeedback?.(event);
                } catch (err) {
                  console.error(
                    `robot-bus action '${actionName}' feedback callback error`,
                    err,
                  );
                }
              }
              if (ev.kind === ActionKind.RESULT) {
                resultEvent = event;
              }
            },
          },
        );
        softCancel = control.cancel;
        session.cancel = () => {
          pendingSoftCancel = true;
          softCancel();
        };
        session.close = () => {
          control.close();
          if (!controller.signal.aborted) controller.abort();
        };
        if (pendingSoftCancel) softCancel();
        await done;
        if (!resultEvent) {
          throw new Error(
            `action '${actionName}' goal '${id}' completed without a result`,
          );
        }
        return resultEvent;
      } finally {
        if (this.actionSessions.get(id)?.controller === controller) {
          this.actionSessions.delete(id);
        }
      }
    })();

    return new WsGoalHandle(id, actionName, result, () =>
      this.cancelGoal(actionName, id)
    );
  }

  async cancelGoal(
    actionName: string,
    goalId: string,
  ): Promise<void> {
    const active = this.actionSessions.get(goalId);
    if (!active || active.actionName !== actionName) {
      throw new Error(`action '${actionName}' has no active goal '${goalId}'`);
    }
    // Soft cancel: CANCEL frame on the open WS; do not tear down the connection.
    active.cancel();
  }

  /** Start background subscribe streams (non-blocking). */
  start(): void {
    if (this.running) return;
    this.running = true;
    this.startTopologyRegistration();
    this.abort = new AbortController();
    // Optional prefix coalesce reduces ZMQ SUB sockets; each filter is one WS.
    for (const filter of coalesceSubscribeFilters([
      ...this.subscriptions.keys(),
    ])) {
      void this.pumpTopic(filter, this.abort.signal);
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
    for (const session of this.actionSessions.values()) {
      session.close();
    }
    this.actionSessions.clear();
    this.stopTopologyRegistration();
    this.session.close();
  }

  private trackEndpoint(kind: TopologyKind, topic: string): void {
    const key = `${kind}:${topic}`;
    if (this.topologyEndpoints.has(key)) return;
    const endpoint = { endpointId: endpointId(), kind, topic };
    this.topologyEndpoints.set(key, endpoint);
    if (this.topologyStarted) this.registerEndpoint(endpoint);
  }

  private startTopologyRegistration(): void {
    if (this.topologyStarted || !this.topologyEnabled) return;
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
      this.publishControl(
        TOPOLOGY_UNREGISTER,
        TopologyUnregister.toBinary(
          TopologyUnregister.create({ endpointId: endpoint.endpointId }),
        ),
      );
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
    this.publishControl(
      TOPOLOGY_REGISTER,
      TopologyRegister.toBinary(
        TopologyRegister.create({
          endpointId: endpoint.endpointId,
          nodeName: this.name,
          kind: endpoint.kind,
          topic: endpoint.topic,
        }),
      ),
    );
  }

  private registerTopicType(topic: string, typeName: string): void {
    this.publishControl(
      TOPIC_TYPE_REGISTER,
      TopicTypeRegister.toBinary(
        TopicTypeRegister.create({ topic, typeName }),
      ),
    );
  }

  private publishControl(topic: string, payload: Uint8Array): void {
    if (!this.topologyEnabled) return;
    void this.callService(topic, payload, 2).catch(() => {
      // Console introspection is best-effort and must not break message traffic.
    });
  }

  private async pumpTopic(filter: string, signal: AbortSignal): Promise<void> {
    try {
      const req = SubscribeRequest.toBinary(
        SubscribeRequest.create({ topic: filter }),
      );
      const { control, done } = await this.session.serverStream(
        METHOD_SUBSCRIBE,
        req,
        {
          onData: (payload) => {
            const msg = TopicMessage.fromBinary(payload);
            const cbs: SubCallback[] = [];
            const exact = this.subscriptions.get(msg.topic);
            if (exact) cbs.push(...exact);
            for (const [key, list] of this.subscriptions) {
              if (
                key !== msg.topic &&
                key.endsWith("/") &&
                msg.topic.startsWith(key)
              ) {
                cbs.push(...list);
              }
            }
            for (const cb of cbs) {
              try {
                cb(msg.topic, msg.payload);
              } catch (err) {
                console.error("robot-bus subscription callback error", err);
              }
            }
          },
        },
      );
      const onAbort = () => control.close();
      signal.addEventListener("abort", onAbort, { once: true });
      try {
        await done;
      } finally {
        signal.removeEventListener("abort", onAbort);
      }
    } catch (err) {
      if (signal.aborted) return;
      console.error(`robot-bus subscribe '${filter}' failed`, err);
    }
  }
}

/**
 * Collapse related topic subscriptions onto one Subscribe filter (fewer WS + ZMQ SUBs).
 * Unrelated topics keep one stream each.
 *
 * Exported for unit tests.
 */
export function coalesceSubscribeFilters(topics: string[]): string[] {
  if (topics.length <= 1) return topics.slice();
  let prefix = topics[0] ?? "";
  for (let i = 1; i < topics.length; i += 1) {
    const topic = topics[i] ?? "";
    while (!topic.startsWith(prefix)) {
      prefix = prefix.slice(0, -1);
      if (!prefix) return topics.slice();
    }
  }
  if (prefix.length < 5) return topics.slice();

  // Prefer a directory-style prefix (`…/`) so we do not over-match siblings.
  if (!prefix.endsWith("/")) {
    const cut = prefix.lastIndexOf("/");
    if (cut >= 0) {
      const dir = prefix.slice(0, cut + 1);
      if (dir.length >= 5 && topics.every((t) => t.startsWith(dir))) {
        return [dir];
      }
    }
    if (topics.every((t) => t === prefix)) return [prefix];
    return topics.slice();
  }
  return [prefix];
}

function mapEvent(ev: PbActionEvent): WsActionEvent {
  return {
    kind: kindFromPb(ev.kind),
    body: ev.body,
    goalId: ev.goalId,
    actionName: ev.actionName,
  };
}

/** Browser package entry alias. */
export { WsNode as Node };
