import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { createPumpTaskScheduler, schedulePump } from "./pump.js?v=20260718-staged-fast-boot";
import { DEFAULT_JIT_ENABLED, DEFAULT_STEP_SLICE, resetJitState, state } from "./state.js?v=20260718-staged-fast-boot";

const previousPerformance = globalThis.performance;
const previousPostMessage = globalThis.postMessage;

afterEach(() => {
  globalThis.performance = previousPerformance;
  globalThis.postMessage = previousPostMessage;
  state.emulator = undefined;
  state.executionMode = "cooperative";
  state.jitEnabled = DEFAULT_JIT_ENABLED;
  resetJitState();
  state.lastAutosavePollAt = 0;
  state.lastMetricsAt = 0;
  state.lastUartPollAt = 0;
  state.networkStatus = "offline";
  state.pumpScheduled = false;
  state.running = false;
  state.vcpuPool = undefined;
});

test("cached jit pump path does not probe boolean then", async () => {
  const descriptor = Object.getOwnPropertyDescriptor(Boolean.prototype, "then");
  let thenReads = 0;
  let resolveRun;
  const ran = new Promise((resolve) => (resolveRun = resolve));
  Object.defineProperty(Boolean.prototype, "then", {
    configurable: true,
    get() {
      thenReads += 1;
      return undefined;
    },
  });
  globalThis.performance = { now: () => 100 };
  globalThis.postMessage = () => {};
  state.jitEnabled = true;
  state.jitBlocks.set(0x1000n, {
    alternateExitPc: 0n,
    dynamicExit: false,
    exitPc: 0x1004n,
    memoryGeneration: 4n,
    rawHash: 1n,
    run: () => 0x1004n,
    startPageGeneration: 2n,
    endPageGeneration: 3n,
    startPa: 0x2000n,
    startPc: 0x1000n,
    statePtr: 0x3000n,
    steps: 1,
  });
  state.lastAutosavePollAt = 10_000;
  state.lastMetricsAt = 10_000;
  state.lastUartPollAt = 10_000;
  state.running = true;
  state.emulator = {
    jit_finish_cached_block: () => {
      state.running = false;
      resolveRun();
      return 0;
    },
    jit_last_error: () => "",
    jit_prepare_cached_block: () => true,
    pc: () => 0x1000n,
  };

  try {
    schedulePump();
    await ran;

    assert.equal(thenReads, 0);
  } finally {
    if (descriptor) {
      Object.defineProperty(Boolean.prototype, "then", descriptor);
    } else {
      delete Boolean.prototype.then;
    }
  }
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

test("parallel interpreter batches run through the vcpu pool", async () => {
  let resolveRun;
  const ran = new Promise((resolve) => (resolveRun = resolve));
  const emulator = {};
  globalThis.performance = { now: () => 100 };
  globalThis.postMessage = () => {};
  state.executionMode = "parallel-wasm";
  state.lastAutosavePollAt = 10_000;
  state.lastMetricsAt = 10_000;
  state.lastUartPollAt = 10_000;
  state.running = true;
  state.emulator = emulator;
  state.vcpuPool = {
    runRound(actual, slice) {
      assert.equal(actual, emulator);
      assert.equal(slice, DEFAULT_STEP_SLICE);
      state.running = false;
      resolveRun();
    },
  };

  schedulePump();
  await ran;
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
