/**
 * Node.js entry: napi-rs native binding + typed helpers.
 * Also re-exports GrpcNode for pure gRPC-Web clients on Node without ZMQ.
 */

import { loadNative } from "./native.js";
import { decode, encode, type MessageType } from "./typed.js";
import type {
  ActionClient as NativeActionClient,
  GoalHandle as NativeGoalHandle,
  Node as NativeNode,
  ServiceClient as NativeServiceClient,
  TopicPublisher as NativeTopicPublisher,
} from "./native-types.js";

export * from "./native-types.js";
export { encode, decode, type MessageType } from "./typed.js";
export * as consoleTopics from "./console-topics.js";
export {
  GrpcNode,
  GrpcServiceClient,
  GrpcActionClient,
  GrpcGoalHandle,
  TypedGrpcServiceClient,
  TypedGrpcActionClient,
  DEFAULT_GRPC_URL,
  type GrpcActionEvent,
  type GrpcSendGoalOptions,
} from "./grpc-node.js";

const native = loadNative();

export const Publisher = native.Publisher;
export const Subscriber = native.Subscriber;
export const ShutdownHandle = native.ShutdownHandle;
export const TimerHandle = native.TimerHandle;
export const Context = native.Context;
export const JsCallbackGroupType = native.JsCallbackGroupType;
export const CallbackGroupType = native.JsCallbackGroupType;
export const JsCallbackGroup = native.JsCallbackGroup;
export const CallbackGroup = native.JsCallbackGroup;
export const SingleThreadedExecutor = native.SingleThreadedExecutor;
export const MultiThreadedExecutor = native.MultiThreadedExecutor;
export const RobotBusBroker = native.RobotBusBroker;
export const messageXsubEndpoint = native.messageXsubEndpoint;
export const messageXpubEndpoint = native.messageXpubEndpoint;
export const runBroker = native.runBroker;
export const __version__ = native.getVersion();
/** Create an offline typed TF buffer (no listener). */

export class TypedTopicPublisher<T extends object> {
  constructor(
    private readonly inner: NativeTopicPublisher,
    private readonly msgType: MessageType<T>,
  ) {}

  get topic(): string {
    return this.inner.topic;
  }

  publish(message: T): void {
    this.inner.publish(Buffer.from(encode(this.msgType, message)));
  }
}

export class TypedServiceClient<Req extends object, Res extends object> {
  constructor(
    private readonly inner: NativeServiceClient,
    private readonly requestType: MessageType<Req>,
    private readonly responseType: MessageType<Res>,
  ) {}

  get serviceName(): string {
    return this.inner.serviceName;
  }

  call(request: Req, timeout?: number): Res {
    const raw = this.inner.call(
      Buffer.from(encode(this.requestType, request)),
      timeout,
    );
    const reply = decode(this.responseType, raw);
    if (!reply) {
      throw new Error(`service ${this.serviceName} response decode failed`);
    }
    return reply;
  }
}

export class TypedActionClient<
  Goal extends object,
  Feedback extends object,
  Result extends object,
> {
  constructor(
    private readonly inner: NativeActionClient,
    private readonly goalType: MessageType<Goal>,
    private readonly feedbackType: MessageType<Feedback>,
    private readonly resultType: MessageType<Result>,
  ) {}

  get actionName(): string {
    return this.inner.actionName;
  }

  sendGoal(
    goal: Goal,
    options: TypedSendGoalOptions<Feedback> = {},
  ): TypedActionGoalHandle<Result> {
    const raw = this.inner.sendGoal(Buffer.from(encode(this.goalType, goal)), {
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
    return new TypedActionGoalHandle(raw, this.resultType);
  }
}

export interface TypedSendGoalOptions<Feedback> {
  goalId?: string;
  timeoutSeconds?: number;
  onFeedback?: (feedback: Feedback) => void;
}

export class TypedActionGoalHandle<Result extends object> {
  constructor(
    private readonly inner: NativeGoalHandle,
    private readonly resultType: MessageType<Result>,
  ) {}

  get goalId(): string {
    return this.inner.goalId;
  }

  get actionName(): string {
    return this.inner.actionName;
  }

  async result(): Promise<Result> {
    const event = await this.inner.result();
    const result = decode(this.resultType, event.body);
    if (!result) {
      throw new Error(`action ${this.actionName} result decode failed`);
    }
    return result;
  }

  cancel(body?: Buffer): void {
    this.inner.cancel(body);
  }
}

/** Thin wrapper adding optional typed create_* overloads. */
export class Node {
  private readonly inner: NativeNode;

  constructor(
    name: string,
    host?: string,
    transport?: string,
    grpcUrl?: string,
    messageXsub?: string,
    messageXpub?: string,
    serviceFrontend?: string,
    serviceBackend?: string,
    actionBackend?: string,
    actionFrontend?: string,
  );
  constructor(inner: NativeNode);
  constructor(
    nameOrInner: string | NativeNode,
    host?: string,
    transport?: string,
    grpcUrl?: string,
    messageXsub?: string,
    messageXpub?: string,
    serviceFrontend?: string,
    serviceBackend?: string,
    actionBackend?: string,
    actionFrontend?: string,
  ) {
    if (typeof nameOrInner === "string") {
      this.inner = new native.Node(
        nameOrInner,
        host,
        transport,
        grpcUrl,
        messageXsub,
        messageXpub,
        serviceFrontend,
        serviceBackend,
        actionBackend,
        actionFrontend,
      );
    } else {
      this.inner = nameOrInner;
    }
  }

  static tcp(name: string, host?: string): Node {
    return new Node(native.Node.tcp(name, host));
  }

  static ipc(name: string, path?: string): Node {
    return new Node(native.Node.ipc(name, path));
  }

  static inproc(name: string, prefix?: string): Node {
    return new Node(native.Node.inproc(name, prefix));
  }

  static inprocWithContext(
    context: import("./native-types.js").Context,
    name: string,
    prefix?: string,
  ): Node {
    return new Node(native.Node.inprocWithContext(context, name, prefix));
  }

  static withContext(
    context: import("./native-types.js").Context,
    name: string,
    host?: string,
    transport?: string,
    grpcUrl?: string,
    messageXsub?: string,
    messageXpub?: string,
    serviceFrontend?: string,
    serviceBackend?: string,
    actionBackend?: string,
    actionFrontend?: string,
  ): Node {
    return new Node(
      native.Node.withContext(
        context,
        name,
        host,
        transport,
        grpcUrl,
        messageXsub,
        messageXpub,
        serviceFrontend,
        serviceBackend,
        actionBackend,
        actionFrontend,
      ),
    );
  }

  static grpc(name: string): Node {
    return new Node(native.Node.grpc(name));
  }

  static grpcAt(name: string, url: string): Node {
    return new Node(native.Node.grpcAt(name, url));
  }

  static discover(
    name: string,
    options?: import("./native-types.js").DiscoverNodeOptions,
  ): Node {
    return new Node(native.Node.discover(name, options));
  }

  get name(): string {
    return this.inner.name;
  }

  createCallbackGroup(kind: import("./native-types.js").JsCallbackGroupType) {
    return this.inner.createCallbackGroup(kind);
  }

  createPublisher(topic: string): NativeTopicPublisher;
  createPublisher<T extends object>(
    topic: string,
    msgType: MessageType<T>,
  ): TypedTopicPublisher<T>;
  createPublisher<T extends object>(
    topic: string,
    msgType?: MessageType<T>,
  ): NativeTopicPublisher | TypedTopicPublisher<T> {
    const pub = this.inner.createPublisher(topic);
    if (msgType) {
      return new TypedTopicPublisher(pub, msgType);
    }
    return pub;
  }

  createSubscription(
    topic: string,
    callback: (topic: string, payload: Buffer | object) => void,
    callbackGroup?: import("./native-types.js").JsCallbackGroup,
  ): void;
  createSubscription<T extends object>(
    topic: string,
    callback: (topic: string, msg: T) => void,
    msgType: MessageType<T>,
    callbackGroup?: import("./native-types.js").JsCallbackGroup,
  ): void;
  createSubscription(
    topic: string,
    callback: (topic: string, payload: Buffer | object) => void,
    msgTypeOrGroup?:
      | MessageType<object>
      | import("./native-types.js").JsCallbackGroup,
    maybeGroup?: import("./native-types.js").JsCallbackGroup,
  ): void {
    if (msgTypeOrGroup && "fromBinary" in msgTypeOrGroup) {
      const msgType = msgTypeOrGroup;
      this.inner.createSubscription(
        topic,
        (t, payload) => {
          const decoded = decode(msgType, payload);
          if (decoded) {
            callback(t, decoded);
          }
        },
        maybeGroup,
      );
      return;
    }
    this.inner.createSubscription(
      topic,
      callback as (topic: string, payload: Buffer) => void,
      msgTypeOrGroup as import("./native-types.js").JsCallbackGroup | undefined,
    );
  }

  createTimer(
    period: number,
    callback: () => void,
    callbackGroup?: import("./native-types.js").JsCallbackGroup,
  ) {
    return this.inner.createTimer(period, callback, callbackGroup);
  }

  cancelTimer(handle: import("./native-types.js").TimerHandle) {
    return this.inner.cancelTimer(handle);
  }

  createService(
    serviceName: string,
    handler: (body: Buffer) => Buffer,
    callbackGroup?: import("./native-types.js").JsCallbackGroup,
  ): void {
    this.inner.createService(serviceName, handler, callbackGroup);
  }

  createClient(serviceName: string): NativeServiceClient;
  createClient<Req extends object, Res extends object>(
    serviceName: string,
    requestType: MessageType<Req>,
    responseType: MessageType<Res>,
  ): TypedServiceClient<Req, Res>;
  createClient(
    serviceName: string,
    requestType?: MessageType<object>,
    responseType?: MessageType<object>,
  ): NativeServiceClient | TypedServiceClient<object, object> {
    const client = this.inner.createClient(serviceName);
    if (requestType && responseType) {
      return new TypedServiceClient(client, requestType, responseType);
    }
    return client;
  }

  createActionServer(
    actionName: string,
    handler: (payload: Buffer) => Array<{ phase: string; body: Buffer }>,
    callbackGroup?: import("./native-types.js").JsCallbackGroup,
  ): void {
    this.inner.createActionServer(actionName, handler, callbackGroup);
  }

  createActionClient(actionName: string): NativeActionClient;
  createActionClient<G extends object, F extends object, R extends object>(
    actionName: string,
    goalType: MessageType<G>,
    feedbackType: MessageType<F>,
    resultType: MessageType<R>,
  ): TypedActionClient<G, F, R>;
  createActionClient(
    actionName: string,
    goalType?: MessageType<object>,
    feedbackType?: MessageType<object>,
    resultType?: MessageType<object>,
  ): NativeActionClient | TypedActionClient<object, object, object> {
    const client = this.inner.createActionClient(actionName);
    if (goalType && feedbackType && resultType) {
      return new TypedActionClient(client, goalType, feedbackType, resultType);
    }
    return client;
  }

  connectActionClient(): void {
    this.inner.connectActionClient();
  }

  shutdownHandle() {
    return this.inner.shutdownHandle();
  }

  shutdown(): void {
    this.inner.shutdown();
  }

  spinOnce(timeout?: number): boolean {
    return this.inner.spinOnce(timeout);
  }

  spin(): void {
    this.inner.spin();
  }

  start(): void {
    this.inner.start();
  }

  stop(): void {
    this.inner.stop();
  }

  wait(): void {
    this.inner.wait();
  }

  /** Native napi handle. */
  asNative(): NativeNode {
    return this.inner;
  }
}
