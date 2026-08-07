import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  GrpcNode,
  GrpcTopicPublisher,
  TypedGrpcTopicPublisher,
} from "../src/grpc-node.js";
import { encode, type MessageType } from "../src/typed.js";

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
    assert.equal(GrpcNode.grpc("a").url, "http://127.0.0.1:15770");
    assert.equal(GrpcNode.grpcAt("a", "http://example:15770/").url, "http://example:15770");
  });
});

describe("GrpcNode action client", () => {
  it("returns a handle immediately and delivers feedback in real time", async () => {
    const node = GrpcNode.grpc("test");
    let request: {
      actionName: string;
      goal: Uint8Array;
      goalId: string;
      timeoutMs: number;
    } | undefined;
    let releaseResult: (() => void) | undefined;
    let feedbackDelivered = false;
    (node as unknown as {
      actionClient: {
        sendGoal: (
          input: typeof request,
          options?: { abort?: AbortSignal },
        ) => { responses: AsyncIterable<object> };
      };
    }).actionClient = {
      sendGoal: (input) => {
        request = input;
        return {
          responses: (async function* () {
            yield {
              actionName: input?.actionName ?? "",
              goalId: input?.goalId ?? "",
              kind: 2,
              body: new Uint8Array([1]),
            };
            await new Promise<void>((resolve) => {
              releaseResult = resolve;
            });
            yield {
              actionName: input?.actionName ?? "",
              goalId: input?.goalId ?? "",
              kind: 3,
              body: new Uint8Array([2]),
            };
          })(),
        };
      },
    };

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
    assert.equal(request?.actionName, "/act");
    assert.equal(request?.goalId, "goal-1");
    assert.equal(request?.timeoutMs, 2_000);
    await new Promise((resolve) => setTimeout(resolve, 0));
    assert.equal(feedbackDelivered, true);
    releaseResult?.();
    const result = await handle.result();
    assert.equal(result.kind, "RESULT");
    assert.deepEqual(Array.from(result.body), [2]);
  });

  it("decodes typed feedback and result as they arrive", async () => {
    const node = GrpcNode.grpc("test");
    (node as unknown as {
      actionClient: {
        sendGoal: () => { responses: AsyncIterable<object> };
      };
    }).actionClient = {
      sendGoal: () => ({
        responses: (async function* () {
          yield {
            actionName: "/typed",
            goalId: "typed-1",
            kind: 2,
            body: new Uint8Array([7]),
          };
          yield {
            actionName: "/typed",
            goalId: "typed-1",
            kind: 3,
            body: new Uint8Array([9]),
          };
        })(),
      }),
    };
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
    const node = GrpcNode.grpc("test");
    const signals: AbortSignal[] = [];
    (node as unknown as {
      actionClient: {
        sendGoal: (
          input: object,
          options?: { abort?: AbortSignal },
        ) => { responses: AsyncIterable<object> };
      };
    }).actionClient = {
      sendGoal: (_input, options) => {
        const signal = options?.abort;
        if (signal) signals.push(signal);
        return {
          responses: (async function* () {
            await new Promise<void>((_resolve, reject) => {
              if (signal?.aborted) {
                reject(new Error("aborted"));
                return;
              }
              signal?.addEventListener("abort", () => reject(new Error("aborted")), {
                once: true,
              });
            });
            yield {};
          })(),
        };
      },
    };

    const client = node.createActionClient("/act");
    const first = client.sendGoal(new Uint8Array(), { goalId: "goal-cancel-1" });
    await first.cancel();
    assert.equal(signals[0]?.aborted, true);
    await assert.rejects(first.result(), /aborted/);

    const second = client.sendGoal(new Uint8Array(), { goalId: "goal-cancel-2" });
    await client.cancel("goal-cancel-2");
    assert.equal(signals[1]?.aborted, true);
    await assert.rejects(second.result(), /aborted/);
    await assert.rejects(client.cancel("goal-cancel-2"), /no active goal/);
  });

  it("rejects result when the RPC stream fails", async () => {
    const node = GrpcNode.grpc("test");
    (node as unknown as {
      actionClient: {
        sendGoal: () => { responses: AsyncIterable<object> };
      };
    }).actionClient = {
      sendGoal: () => ({
        responses: (async function* () {
          throw new Error("rpc unavailable");
        })(),
      }),
    };

    const handle = node.createActionClient("/act").sendGoal(new Uint8Array());
    await assert.rejects(handle.result(), /rpc unavailable/);
  });

  it("rejects missing results and safely cleans duplicate and shutdown goals", async () => {
    const node = GrpcNode.grpc("test");
    let signal: AbortSignal | undefined;
    let calls = 0;
    (node as unknown as {
      actionClient: {
        sendGoal: (
          input: object,
          options?: { abort?: AbortSignal },
        ) => { responses: AsyncIterable<object> };
      };
    }).actionClient = {
      sendGoal: (_input, options) => {
        calls += 1;
        signal = options?.abort;
        return {
          responses: calls === 1
            ? (async function* () {})()
            : (async function* () {
                await new Promise<void>((_resolve, reject) => {
                  signal?.addEventListener("abort", () => reject(new Error("shutdown abort")), {
                    once: true,
                  });
                });
              })(),
        };
      },
    };

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
  it("registers, refreshes, and unregisters browser topic endpoints", async () => {
    const originalFetch = globalThis.fetch;
    const requests: Array<{ url: string; body: Record<string, unknown> }> = [];
    globalThis.fetch = (async (input, init) => {
      requests.push({
        url: String(input),
        body: JSON.parse(String(init?.body ?? "{}")) as Record<string, unknown>,
      });
      return new Response(JSON.stringify({ ok: true }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    }) as typeof fetch;

    try {
      const FakeType = {
        typeName: "fake.v1.Msg",
        create: (v?: object) => (v ?? {}) as object,
        toBinary: () => new Uint8Array(),
        fromBinary: () => ({}),
      } as MessageType<object>;
      const node = GrpcNode.grpcAt("web_test", "http://grpc.invalid", {
        consoleUrl: "http://console.test/",
        topologyRefreshMs: 100,
      });
      node.createPublisher("/typed", FakeType);
      node.start();
      node.createSubscription("/cmd", () => {});

      const firstRegisters = requests.filter((request) =>
        request.url.endsWith("/api/v1/topology/register"),
      );
      assert.equal(firstRegisters.length, 2);
      assert.deepEqual(
        firstRegisters.map((request) => [request.body.nodeName, request.body.kind, request.body.topic]),
        [
          ["web_test", "publisher", "/typed"],
          ["web_test", "subscriber", "/cmd"],
        ],
      );
      assert.ok(requests.some((request) =>
        request.url.endsWith("/api/v1/topics/register") &&
        request.body.topic === "/typed" &&
        request.body.typeName === "fake.v1.Msg"
      ));

      await new Promise((resolve) => setTimeout(resolve, 120));
      assert.ok(
        requests.filter((request) => request.url.endsWith("/api/v1/topology/register")).length >= 4,
      );

      node.shutdown();
      assert.equal(
        requests.filter((request) => request.url.endsWith("/api/v1/topology/unregister")).length,
        2,
      );
    } finally {
      globalThis.fetch = originalFetch;
    }
  });

  it("keeps registration failures best-effort", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async () => {
      throw new Error("console unavailable");
    }) as typeof fetch;
    try {
      const node = GrpcNode.grpc("web_test", {
        consoleUrl: "http://console.test",
      });
      node.createPublisher("/topic");
      node.start();
      await new Promise((resolve) => setTimeout(resolve, 0));
      node.shutdown();
    } finally {
      globalThis.fetch = originalFetch;
    }
  });
});
