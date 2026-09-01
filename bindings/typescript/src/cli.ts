#!/usr/bin/env node
/**
 * Standalone broker: `npx robot-bus [options]`.
 *
 * Same flags as `python -m robot_bus.broker` / `robot_bus_broker`.
 * Prefer `RobotBusBroker.start()` in application code; this CLI is for demos
 * and a long-running process.
 */

import { loadNative } from "./native.js";

loadNative().runBroker();
