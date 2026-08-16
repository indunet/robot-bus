/**
 * Browser entry: WebSocket RPC client only.
 * Bundlers resolve this via package.json `exports.browser`.
 */

export {
  WsNode,
  WsNode as Node,
  WsServiceClient,
  WsActionClient,
  WsGoalHandle,
  WsTopicPublisher,
  TypedWsTopicPublisher,
  TypedWsServiceClient,
  TypedWsActionClient,
  DEFAULT_WS_URL,
  type WsActionEvent,
  type WsSendGoalOptions,
} from "./ws-node.js";

export { encode, decode, type MessageType } from "./typed.js";
export * as consoleTopics from "./console-topics.js";

export const __version__ = "1.0.0";
