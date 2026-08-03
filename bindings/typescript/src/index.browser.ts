/**
 * Browser entry: gRPC-Web client only.
 * Bundlers resolve this via package.json `exports.browser`.
 */

export {
  GrpcNode,
  GrpcNode as Node,
  GrpcServiceClient,
  GrpcActionClient,
  TypedGrpcServiceClient,
  TypedGrpcActionClient,
  DEFAULT_GRPC_URL,
  type GrpcActionEvent,
} from "./grpc-node.js";

export { encode, decode, type MessageType } from "./typed.js";

export const __version__ = "0.1.3";

export function createPublisher(): never {
  throw new Error(
    "createPublisher is not available in the browser build. Use Node.js native binding.",
  );
}
