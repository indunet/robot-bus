/**
 * Hand-written typings for the napi-rs addon (`robot-bus.*.node`).
 * Keep in sync with `native/src/lib.rs`.
 */

export interface TopicMessage {
  topic: string;
  payload: Buffer;
}

export interface ActionEvent {
  kind: string;
  body: Buffer;
  goalId: string;
  actionName: string;
}

export declare class ShutdownHandle {
  shutdown(): void;
  isRunning(): boolean;
}

export declare class TimerHandle {}

export declare enum JsCallbackGroupType {
  MutuallyExclusive = 0,
  Reentrant = 1,
}

export declare class JsCallbackGroup {
  readonly id: number;
  readonly kind: JsCallbackGroupType;
}

export declare class Publisher {
  constructor(endpoint?: string);
  publish(topic: string, payload: Buffer): void;
  readonly endpoint: string;
}

export declare class Subscriber {
  constructor(endpoint?: string);
  subscribe(topic: string): void;
  unsubscribe(topic: string): void;
  receive(timeout?: number): TopicMessage;
  readonly endpoint: string;
}

export declare class TopicPublisher {
  readonly topic: string;
  publish(payload: Buffer): void;
}

export declare class ServiceClient {
  readonly serviceName: string;
  call(body: Buffer, timeout?: number): Buffer;
}

export declare class ActionClient {
  readonly actionName: string;
  sendGoal(body: Buffer, goalId?: string, timeout?: number): ActionEvent[];
  cancel(goalId: string, body?: Buffer, timeout?: number): ActionEvent;
}

export declare class Context {
  constructor();
}

export declare class Node {
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
  static tcp(name: string, host?: string): Node;
  static ipc(name: string, path?: string): Node;
  static inproc(name: string, prefix?: string): Node;
  static inprocWithContext(context: Context, name: string, prefix?: string): Node;
  static withContext(
    context: Context,
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
  ): Node;
  static grpc(name: string): Node;
  static grpcAt(name: string, url: string): Node;
  static discover(name: string, options?: DiscoverNodeOptions): Node;
  readonly name: string;
  createCallbackGroup(kind: JsCallbackGroupType): JsCallbackGroup;
  createPublisher(topic: string): TopicPublisher;
  createSubscription(
    topic: string,
    callback: (topic: string, payload: Buffer) => void,
    callbackGroup?: JsCallbackGroup,
  ): void;
  createTimer(
    period: number,
    callback: () => void,
    callbackGroup?: JsCallbackGroup,
  ): TimerHandle;
  cancelTimer(handle: TimerHandle): void;
  createService(
    serviceName: string,
    handler: (body: Buffer) => Buffer,
    callbackGroup?: JsCallbackGroup,
  ): void;
  createClient(serviceName: string): ServiceClient;
  createActionServer(
    actionName: string,
    handler: (payload: Buffer) => Array<{ phase: string; body: Buffer }>,
    callbackGroup?: JsCallbackGroup,
  ): void;
  createActionClient(actionName: string): ActionClient;
  connectActionClient(): void;
  shutdownHandle(): ShutdownHandle;
  shutdown(): void;
  spinOnce(timeout?: number): boolean;
  spin(): void;
  start(): void;
  stop(): void;
  wait(): void;
}

export declare class SingleThreadedExecutor {
  constructor(context?: Context);
  addNode(node: Node): void;
  createNode(
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
  ): Node;
  shutdownHandle(): ShutdownHandle;
  shutdown(): void;
  spinOnce(timeout?: number): boolean;
  spin(): void;
  start(): void;
  stop(): void;
  wait(): void;
}

export declare class MultiThreadedExecutor {
  constructor(numThreads?: number, context?: Context);
  addNode(node: Node): void;
  createNode(
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
  ): Node;
  shutdownHandle(): ShutdownHandle;
  shutdown(): void;
  spinOnce(timeout?: number): boolean;
  spin(): void;
}

export interface DiscoverNodeOptions {
  transport?: string;
  domainId?: number;
  brokerId?: string;
  multicastAddr?: string;
  multicastPort?: number;
  timeoutSecs?: number;
}

export interface BrokerStartOptions {
  messageXsubBind?: string;
  messageXpubBind?: string;
  messageSndHwm?: number;
  messageRcvHwm?: number;
  serviceFrontendBind?: string;
  serviceBackendBind?: string;
  serviceSndHwm?: number;
  serviceRcvHwm?: number;
  serviceHeartbeatIntervalMs?: number;
  serviceHeartbeatTimeoutMs?: number;
  actionFrontendBind?: string;
  actionBackendBind?: string;
  actionSndHwm?: number;
  actionRcvHwm?: number;
  actionHeartbeatIntervalMs?: number;
  actionHeartbeatTimeoutMs?: number;
  actionPendingTimeoutMs?: number;
  sndHwm?: number;
  rcvHwm?: number;
  heartbeatIntervalMs?: number;
  heartbeatTimeoutMs?: number;
  tcpOnly?: boolean;
  grpcListen?: string;
  corsOrigins?: string[];
  consoleListen?: string;
  noConsole?: boolean;
  brokerId?: string;
  messagePeers?: string[];
  servicePeers?: string[];
  actionPeers?: string[];
  domainId?: number;
  noDiscovery?: boolean;
  advertiseHost?: string;
  discoveryAddr?: string;
  discoveryPort?: number;
}

export declare class RobotBusBroker {
  static start(options?: BrokerStartOptions, context?: Context): RobotBusBroker;
  stop(): void;
  readonly messageXsubBind: string;
  readonly messageXpubBind: string;
  readonly serviceFrontendBind: string;
  readonly serviceBackendBind: string;
  readonly actionFrontendBind: string;
  readonly actionBackendBind: string;
  readonly grpcListen: string;
  readonly consoleListen: string | null;
}

export declare function messageXsubEndpoint(
  host?: string,
  transport?: string,
): string;
export declare function messageXpubEndpoint(
  host?: string,
  transport?: string,
): string;
export declare function runBroker(): void;
export declare function getVersion(): string;

export declare class TfBuffer {
  constructor();
  clear(): void;
  setTransformMsg(data: Buffer, isStatic: boolean): void;
  lookupTransform(target: string, source: string): Buffer;
  canTransform(target: string, source: string): boolean;
  frames(): string[];
}

export declare class TfListener {
  constructor(node: Node, tfTopic?: string, tfStaticTopic?: string);
  static withDefaults(node: Node): TfListener;
  buffer(): TfBuffer;
}
