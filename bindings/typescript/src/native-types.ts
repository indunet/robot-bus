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

export interface SendGoalOptions {
  goalId?: string;
  timeoutSeconds?: number;
  onFeedback?: (feedback: ActionEvent) => void;
}

export declare class ShutdownHandle {
  shutdown(): void;
  isRunning(): boolean;
}

export declare class TimerHandle {}

export declare class SubscriptionHandle {
  readonly id: number | null;
}

export declare class ServiceHandle {
  readonly id: number | null;
  readonly serviceName: string | null;
}

export declare class ActionServerHandle {
  readonly id: number | null;
  readonly actionName: string | null;
}

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
  serviceIsReady(): boolean;
  waitForService(timeout?: number): boolean;
  call(body: Buffer, timeout?: number): Buffer;
}

export declare class ActionClient {
  readonly actionName: string;
  actionServerIsReady(): boolean;
  waitForActionServer(timeout?: number): boolean;
  sendGoal(body: Buffer, options?: SendGoalOptions): GoalHandle;
}

export declare class GoalHandle {
  readonly goalId: string;
  readonly actionName: string;
  result(): Promise<ActionEvent>;
  cancel(body?: Buffer): void;
}

export declare class Context {
  constructor();
}

export declare class Node {
  constructor(
    name: string,
    host?: string,
    transport?: string,
    wsUrl?: string,
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
    wsUrl?: string,
    messageXsub?: string,
    messageXpub?: string,
    serviceFrontend?: string,
    serviceBackend?: string,
    actionBackend?: string,
    actionFrontend?: string,
  ): Node;
  static ws(name: string): Node;
  static wsAt(name: string, url: string): Node;
  static discover(name: string, options?: DiscoverNodeOptions): Node;
  readonly name: string;
  readonly connectionState: string;
  waitForBroker(timeoutSeconds?: number): boolean;
  addOnConnectionEvent(
    callback: (oldState: string, next: string, reason: string) => void,
  ): void;
  createCallbackGroup(kind: JsCallbackGroupType): JsCallbackGroup;
  createPublisher(topic: string, qosDepth?: number): TopicPublisher;
  createSubscription(
    topic: string,
    callback: (payload: Buffer) => void,
    callbackGroup?: JsCallbackGroup,
    qosDepth?: number,
  ): SubscriptionHandle;
  destroySubscription(handle: SubscriptionHandle): void;
  createTimer(
    period: number,
    callback: () => void,
    callbackGroup?: JsCallbackGroup,
  ): TimerHandle;
  createWallTimer(
    period: number,
    callback: () => void,
    callbackGroup?: JsCallbackGroup,
  ): TimerHandle;
  cancelTimer(handle: TimerHandle): void;
  createService(
    serviceName: string,
    handler: (body: Buffer) => Buffer,
    callbackGroup?: JsCallbackGroup,
    qosDepth?: number,
  ): ServiceHandle;
  destroyService(handle: ServiceHandle): void;
  createClient(serviceName: string, qosDepth?: number): ServiceClient;
  createActionServer(
    actionName: string,
    handler: (payload: Buffer) => Array<{ phase: string; body: Buffer }>,
    callbackGroup?: JsCallbackGroup,
    qosDepth?: number,
  ): ActionServerHandle;
  destroyActionServer(handle: ActionServerHandle): void;
  createActionClient(actionName: string, qosDepth?: number): ActionClient;
  connectActionClient(): void;
  shutdownHandle(): ShutdownHandle;
  shutdown(): void;
  spinOnce(timeout?: number): boolean;
  waitForMessage(topic: string, timeout?: number): Buffer | null;
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
    wsUrl?: string,
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
    wsUrl?: string,
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
  /** Broker API base URL, e.g. http://127.0.0.1:15560 */
  apiUrl?: string;
  brokerId?: string;
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
  /** API / WebSocket / console listen address (default 0.0.0.0:15560). */
  apiListen?: string;
  corsOrigins?: string[];
  consoleListen?: string;
  noConsole?: boolean;
  noTank?: boolean;
  noDocs?: boolean;
  brokerId?: string;
  messagePeers?: string[];
  servicePeers?: string[];
  actionPeers?: string[];
  peers?: string[];
  domainId?: number;
  noDiscovery?: boolean;
  advertiseHost?: string;
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
  readonly apiListen: string;
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


