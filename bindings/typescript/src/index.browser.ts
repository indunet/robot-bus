/**
 * Browser entry: gRPC-Web client only.
 * Bundlers resolve this via package.json `exports.browser`.
 */

export {
  GrpcNode,
  GrpcNode as Node,
  GrpcServiceClient,
  GrpcActionClient,
  GrpcGoalHandle,
  GrpcTopicPublisher,
  TypedGrpcTopicPublisher,
  TypedGrpcServiceClient,
  TypedGrpcActionClient,
  DEFAULT_GRPC_URL,
  type GrpcActionEvent,
  type GrpcSendGoalOptions,
} from "./grpc-node.js";

export { encode, decode, type MessageType } from "./typed.js";
export * as consoleTopics from "./console-topics.js";

export const __version__ = "0.1.6";
