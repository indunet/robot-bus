#!/usr/bin/env node
/**
 * TypeScript/Node interop peer — role: pub (typed Imu).
 * Run from repo root with NODE_PATH / cwd that can resolve bindings/typescript.
 */
import { setTimeout as sleep } from "node:timers/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const TOPIC = "/interop/imu";
const EXPECT_Z = 0.42;

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(__dirname, "../..");
const TS_ROOT = path.join(REPO, "bindings/typescript");

function requireEnv(key) {
  const v = process.env[key];
  if (!v) {
    throw new Error(`missing env ${key}`);
  }
  return v;
}

async function main() {
  const role = requireEnv("ROBOT_BUS_INTEROP_ROLE");
  if (role !== "pub") {
    throw new Error(`unknown ROBOT_BUS_INTEROP_ROLE: ${role}`);
  }

  const { Node } = await import(
    pathToFileURL(path.join(TS_ROOT, "dist/index.node.js")).href
  );
  const { Imu } = await import(
    pathToFileURL(
      path.join(TS_ROOT, "generated/sensor_msgs/msg/v1/imu.ts"),
    ).href
  );

  const node = new Node(
    "interop_ts_pub",
    "127.0.0.1",
    "tcp",
    undefined,
    requireEnv("ROBOT_BUS_MESSAGE_XSUB"),
    requireEnv("ROBOT_BUS_MESSAGE_XPUB"),
    requireEnv("ROBOT_BUS_SERVICE_FRONTEND"),
    requireEnv("ROBOT_BUS_SERVICE_BACKEND"),
    requireEnv("ROBOT_BUS_ACTION_BACKEND"),
    requireEnv("ROBOT_BUS_ACTION_FRONTEND"),
  );

  const pub = node.createPublisher(TOPIC, Imu);
  await sleep(400);
  const msg = Imu.create({
    angularVelocity: { x: 0, y: 0, z: EXPECT_Z },
  });
  for (let i = 0; i < 5; i++) {
    pub.publish(msg);
    await sleep(50);
  }
  console.log("READY");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
