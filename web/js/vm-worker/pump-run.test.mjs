import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { createPumpTaskScheduler, schedulePump } from "./pump.js";
import { DEFAULT_JIT_ENABLED, DEFAULT_STEP_SLICE, state } from "./state.js";

const previousPerformance = globalThis.performance;
const previousPostMessage = globalThis.postMessage;

afterEach(() => {
  globalThis.performance = previousPerformance;
  globalThis.postMessage = previousPostMessage;
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
  let stepSlice;
  let resolveRun;
  const ran = new Promise((resolve) => (resolveRun = resolve));
  globalThis.performance = { now: () => 100 + nowCalls++ };
  globalThis.postMessage = () => {};
  state.jitEnabled = false;
  state.lastAutosavePollAt = 10_000;
  state.lastMetricsAt = 10_000;
  state.lastUartPollAt = 10_000;
  state.running = true;
  state.emulator = {
    run_kernel(slice) {
      stepSlice = slice;
      state.running = false;
      resolveRun();
    },
  };

  schedulePump();
  await ran;

  assert.equal(stepSlice, DEFAULT_STEP_SLICE);
  assert.equal(nowCalls, 2);
});

test("pump scheduler uses message channel when available", () => {
  let port1;
  class FakeMessageChannel {
    constructor() {
      port1 = { addEventListener: (_, listener) => (port1.listener = listener), start() {} };
      this.port1 = port1;
      this.port2 = { postMessage: () => port1.listener() };
    }
  }
  const scheduler = createPumpTaskScheduler({
    MessageChannelCtor: FakeMessageChannel,
    timeout: () => assert.fail("setTimeout fallback should not run"),
  });
  let ran = false;

  scheduler(() => (ran = true));

  assert.equal(ran, true);
});

test("pump scheduler falls back to timeout without message channel", () => {
  let scheduled;
  let delay;
  const scheduler = createPumpTaskScheduler({
    MessageChannelCtor: null,
    timeout: (callback, ms) => {
      scheduled = callback;
      delay = ms;
    },
  });
  let ran = false;

  scheduler(() => (ran = true));
  scheduled();

  assert.equal(delay, 0);
  assert.equal(ran, true);
});
