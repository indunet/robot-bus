import assert from "node:assert/strict";
import { afterEach, describe, it } from "node:test";
import {
  ActionEvent,
  ActionKind,
  GoalCommand,
} from "../generated/robot_bus_interface/grpc/v1/action_gateway.js";
import {
  GrpcNode,
  GrpcTopicPublisher,
  TypedGrpcTopicPublisher,
  __setWsRpcForTests,
  coalesceSubscribeFilters,
} from "../src/grpc-node.js";
import { encode, type MessageType } from "../src/typed.js";
import type { ServerStreamHandlers } from "../src/ws-rpc.js";

afterEach(() => {
  __setWsRpcForTests();
});

describe("coalesceSubscribeFilters", () => {
  it("multiplexes console /robot_bus/* topics onto one prefix stream", () => {
    assert.deepEqual(
      coalesceSubscribeFilters([
        "/robot_bus/status",
        "/robot_bus/topics",
        "/robot_bus/services",
        "/robot_bus/actions",
        "/robot_bus/topology",
        "/robot_bus/events",
      ]),
      ["/robot_bus/"],
    );
  });

  it("keeps unrelated topics on separate streams", () => {
    assert.deepEqual(
      coalesceSubscribeFilters(["/robot1/imu", "/robot_bus/status"]),
      ["/robot1/imu", "/robot_bus/status"],
    );
  });

  it("passes through a single topic", () => {
    assert.deepEqual(coalesceSubscribeFilters(["/robot_bus/bot/pose"]), [
      "/robot_bus/bot/pose",
    ]);
  });

  it("coalesces bot demo topics with other /robot_bus/* snapshots", () => {
    assert.deepEqual(
      coalesceSubscribeFilters(["/robot_bus/bot/pose", "/robot_bus/status"]),
      ["/robot_bus/"],
    );
  });
});

describe("GrpcNode capability guards", () => {
  it("rejects service / action servers", () => {
    const node = GrpcNode.grpc("test");
    assert.throws(() => node.createService("/s", () => new Uint8Array()), /not available/);
    assert.throws(() => node.createActionServer("/a", () => []), /not available/);
  });

  it("createPublisher returns raw and typed publishers", () => {
    const node = GrpcNode.grpc("test");
    const raw = node.createPublisher("/t");
    assert.ok(raw instanceof GrpcTopicPublisher);
    assert.equal(raw.topic, "/t");

    const FakeType = {
      typeName: "fake.v1.Msg",
      create: (v?: object) => (v ?? {}) as object,
      toBinary: () => new Uint8Array([1, 2, 3]),
      fromBinary: () => ({}),
    } as MessageType<object>;
    const typed = node.createPublisher("/typed", FakeType);
    assert.ok(typed instanceof TypedGrpcTopicPublisher);
    assert.equal(typed.topic, "/typed");
    assert.deepEqual(Array.from(encode(FakeType, {})), [1, 2, 3]);
  });

  it("exposes grpc factory urls", () => {
    assert.equal(GrpcNode.grpc("a").url, "http://127.0.0.1:15570");
    assert.equal(GrpcNode.grpcAt("a", "http://example:15570/").url, "http://example:15570");
  });
});

describe("GrpcNode action client", () => {
  it("returns a handle immediately and delivers feedback in real time", async () => {
    let request: GoalCommand | undefined;
    let releaseResult: (() => void) | undefined;
    let feedbackDelivered = false;

    __setWsRpcForTests({
      serverStream: async (_url, _method, req, handlers, _signal) => {
        request = GoalCommand.fromBinary(req);
        handlers.onData(
          ActionEvent.toBinary(
            ActionEvent.create({
              actionName: request.actionName,
              goalId: request.goalId,
              kind: ActionKind.FEEDBACK,
              body: new Uint8Array([1]),
            }),
          ),
        );
        await new Promise<void>((resolve) => {
          releaseResult = resolve;
        });
        handlers.onData(
          ActionEvent.toBinary(
            ActionEvent.create({
              actionName: request.actionName,
              goalId: request.goalId,
              kind: ActionKind.RESULT,
              body: new Uint8Array([2]),
            }),
          ),
        );
        handlers.onTrailer?.(0, "");
      },
    });

    const node = GrpcNode.grpc("test");
    const client = node.createActionClient("/act");
    const handle = client.sendGoal(new Uint8Array([9]), {
      goalId: "goal-1",
      timeoutSeconds: 2,
      onFeedback: (event) => {
        feedbackDelivered = true;
        assert.equal(event.kind, "FEEDBACK");
        assert.deepEqual(Array.from(event.body), [1]);
      },
    });

    assert.equal(handle.goalId, "goal-1");
    assert.equal(handle.actionName, "/act");
    await new Promise((resolve) => setTimeout(resolve, 0));
    assert.equal(request?.actionName, "/act");
    assert.equal(request?.goalId, "goal-1");
    assert.equal(request?.timeoutMs, 2_000);
    assert.equal(feedbackDelivered, true);
    releaseResult?.();
    const result = await handle.result();
    assert.equal(result.kind, "RESULT");
    assert.deepEqual(Array.from(result.body), [2]);
  });

  it("decodes typed feedback and result as they arrive", async () => {
    __setWsRpcForTests({
      serverStream: async (_url, _method, _req, handlers) => {
        handlers.onData(
          ActionEvent.toBinary(
            ActionEvent.create({
              actionName: "/typed",
              goalId: "typed-1",
              kind: ActionKind.FEEDBACK,
              body: new Uint8Array([7]),
            }),
          ),
        );
        handlers.onData(
          ActionEvent.toBinary(
            ActionEvent.create({
              actionName: "/typed",
              goalId: "typed-1",
              kind: ActionKind.RESULT,
              body: new Uint8Array([9]),
            }),
          ),
        );
        handlers.onTrailer?.(0, "");
      },
    });

    const node = GrpcNode.grpc("test");
    const numberType = {
      typeName: "fake.v1.Number",
      create: (value?: { value?: number }) => ({ value: value?.value ?? 0 }),
      toBinary: (value: { value: number }) => new Uint8Array([value.value]),
      fromBinary: (bytes: Uint8Array) => ({ value: bytes[0] ?? 0 }),
    } as MessageType<{ value: number }>;
    const feedback: number[] = [];
    const client = node.createActionClient("/typed", numberType, numberType, numberType);
    const handle = client.sendGoal(
      { value: 1 },
      { goalId: "typed-1", onFeedback: (value) => feedback.push(value.value) },
    );

    const result = await handle.result();
    assert.deepEqual(feedback, [7]);
    assert.deepEqual(result, { value: 9 });
  });

  it("cancels through the handle and deprecated client wrapper", async () => {
    const cancels: Array<() => void> = [];
    __setWsRpcForTests({
      serverStream: async (_url, _method, _req, handlers) => {
        let resolveDone: (() => void) | undefined;
        const done = new Promise<void>((resolve) => {
          resolveDone = resolve;
        });
        handlers.onControl?.({
          cancel: () => {
            cancels.push(() => undefined);
            handlers.onData(
              ActionEvent.toBinary(
                ActionEvent.create({
                  actionName: "/act",
                  goalId: "goal-cancel-1",
                  kind: ActionKind.RESULT,
                  body: new TextEncoder().encode("cancelled"),
                }),
              ),
            );
            handlers.onTrailer?.(0, "");
            resolveDone?.();
          },
          close: () => {
            resolveDone?.();
          },
        });
        await done;
      },
    });

    const node = GrpcNode.grpc("test");
    const client = node.createActionClient("/act");
    const first = client.sendGoal(new Uint8Array(), { goalId: "goal-cancel-1" });
    await new Promise((r) => setTimeout(r, 0));
    await first.cancel();
    assert.equal(cancels.length, 1);
    const result = await first.result();
    assert.equal(result.kind, "RESULT");

    // Second goal: cancel via deprecated client wrapper; mock replies RESULT.
    let secondCancel = 0;
    __setWsRpcForTests({
      serverStream: async (_url, _method, _req, handlers) => {
        let resolveDone: (() => void) | undefined;
        const done = new Promise<void>((resolve) => {
          resolveDone = resolve;
        });
        handlers.onControl?.({
          cancel: () => {
            secondCancel += 1;
            handlers.onData(
              ActionEvent.toBinary(
                ActionEvent.create({
                  actionName: "/act",
                  goalId: "goal-cancel-2",
                  kind: ActionKind.RESULT,
                  body: new Uint8Array(),
                }),
              ),
            );
            handlers.onTrailer?.(0, "");
            resolveDone?.();
          },
          close: () => resolveDone?.(),
        });
        await done;
      },
    });
    const second = client.sendGoal(new Uint8Array(), { goalId: "goal-cancel-2" });
    await new Promise((r) => setTimeout(r, 0));
    await client.cancel("goal-cancel-2");
    assert.equal(secondCancel, 1);
    await second.result();
    await assert.rejects(client.cancel("goal-cancel-2"), /no active goal/);
  });

  it("rejects result when the RPC stream fails", async () => {
    __setWsRpcForTests({
      serverStream: async () => {
        throw new Error("rpc unavailable");
      },
    });
    const node = GrpcNode.grpc("test");
    const handle = node.createActionClient("/act").sendGoal(new Uint8Array());
    await assert.rejects(handle.result(), /rpc unavailable/);
  });

  it("rejects missing results and safely cleans duplicate and shutdown goals", async () => {
    let signal: AbortSignal | undefined;
    let calls = 0;
    __setWsRpcForTests({
      serverStream: async (_url, _method, _req, handlers: ServerStreamHandlers, sig) => {
        calls += 1;
        signal = sig;
        if (calls === 1) {
          handlers.onTrailer?.(0, "");
          return;
        }
        await new Promise<void>((_resolve, reject) => {
          sig?.addEventListener("abort", () => reject(new Error("shutdown abort")), {
            once: true,
          });
        });
      },
    });

    const node = GrpcNode.grpc("test");
    const client = node.createActionClient("/act");
    const missing = client.sendGoal(new Uint8Array(), { goalId: "reusable" });
    await assert.rejects(missing.result(), /without a result/);

    const active = client.sendGoal(new Uint8Array(), { goalId: "reusable" });
    assert.throws(
      () => client.sendGoal(new Uint8Array(), { goalId: "reusable" }),
      /already active/,
    );
    node.shutdown();
    assert.equal(signal?.aborted, true);
    await assert.rejects(active.result(), /shutdown abort/);
  });
});

describe("GrpcNode console registration", () => {
  it("keeps topology registration best-effort when WS fails", async () => {
    let unaryCalls = 0;
    __setWsRpcForTests({
      unary: async () => {
        unaryCalls += 1;
        throw new Error("gateway unavailable");
      },
      serverStream: async () => {
        /* subscribe may also fail; ignore */
      },
    });

    const FakeType = {
      typeName: "fake.v1.Msg",
      create: (v?: object) => (v ?? {}) as object,
      toBinary: () => new Uint8Array(),
      fromBinary: () => ({}),
    } as MessageType<object>;
    const node = GrpcNode.grpcAt("web_test", "http://grpc.invalid", {
      topologyRefreshMs: 100,
    });
    node.createPublisher("/typed", FakeType);
    node.createSubscription("/cmd", () => {}, FakeType);
    node.start();
    await new Promise((resolve) => setTimeout(resolve, 120));
    node.shutdown();
    assert.ok(unaryCalls >= 1);
  });

  it("keeps registration failures best-effort", async () => {
    __setWsRpcForTests({
      unary: async () => {
        throw new Error("gateway unavailable");
      },
      serverStream: async () => {},
    });
    const node = GrpcNode.grpc("web_test");
    node.createPublisher("/topic");
    node.start();
    await new Promise((resolve) => setTimeout(resolve, 0));
    node.shutdown();
  });

  it("can disable topology registration", async () => {
    let unaryCalls = 0;
    __setWsRpcForTests({
      unary: async () => {
        unaryCalls += 1;
        throw new Error("should not be called");
      },
      serverStream: async () => {},
    });
    const node = GrpcNode.grpc("web_test", { consoleUrl: null });
    node.createPublisher("/topic");
    node.start();
    await new Promise((resolve) => setTimeout(resolve, 0));
    node.shutdown();
    assert.equal(unaryCalls, 0);
  });
});
