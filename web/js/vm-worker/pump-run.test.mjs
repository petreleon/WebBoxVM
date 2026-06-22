import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { schedulePump } from "./pump.js";
import { DEFAULT_JIT_ENABLED, DEFAULT_STEP_SLICE, state } from "./state.js";

const previousPerformance = globalThis.performance;
const previousPostMessage = globalThis.postMessage;
const previousSetTimeout = globalThis.setTimeout;

afterEach(() => {
  globalThis.performance = previousPerformance;
  globalThis.postMessage = previousPostMessage;
  globalThis.setTimeout = previousSetTimeout;
  state.emulator = undefined;
  state.jitEnabled = DEFAULT_JIT_ENABLED;
  state.lastAutosavePollAt = 0;
  state.lastMetricsAt = 0;
  state.lastUartPollAt = 0;
  state.networkStatus = "offline";
  state.pumpScheduled = false;
  state.running = false;
});

test("interpreter fallback reuses current batch timestamp before run", async () => {
  let nowCalls = 0;
  let scheduled;
  let stepSlice;
  globalThis.performance = { now: () => 100 + nowCalls++ };
  globalThis.postMessage = () => {};
  globalThis.setTimeout = (callback) => {
    scheduled = callback;
    return 1;
  };
  state.jitEnabled = false;
  state.lastAutosavePollAt = 10_000;
  state.lastMetricsAt = 10_000;
  state.lastUartPollAt = 10_000;
  state.running = true;
  state.emulator = {
    run_kernel(slice) {
      stepSlice = slice;
      state.running = false;
    },
  };

  schedulePump();
  await scheduled();

  assert.equal(stepSlice, DEFAULT_STEP_SLICE);
  assert.equal(nowCalls, 2);
});
