import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { schedulePump } from "./pump.js?v=20260904-virgl-mixed-depth-batch-r1";
import {
  DEFAULT_JIT_ENABLED,
  DEFAULT_STEP_SLICE,
  INTERACTIVE_STEP_SLICE,
  resetJitState,
  state,
} from "./state.js?v=20260904-virgl-mixed-depth-batch-r1";
import { resetUartInput } from "./uart-input.js?v=20260904-virgl-mixed-depth-batch-r1";

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
  state.lastUart = 0;
  state.lastUartFlushAt = 0;
  state.lastUartPollAt = 0;
  state.networkStatus = "offline";
  state.pumpScheduled = false;
  state.running = false;
  state.stepSlice = DEFAULT_STEP_SLICE;
  state.urgentUartWaiters = 0;
  state.vcpuPool = undefined;
  resetUartInput();
});

test("responsive parallel pump exits its frame on the first UART flush", async () => {
  let rounds = 0;
  let resolveRun;
  const ran = new Promise((resolve) => (resolveRun = resolve));
  const messages = [];
  const emulator = {
    install_disk_generation() {
      state.running = false;
      resolveRun();
      return state.lastAutosaveGeneration;
    },
    uart_output_len: () => (rounds >= 3 ? 1 : 0),
    uart_output_since: () => "x",
  };
  globalThis.performance = { now: () => 1000 };
  globalThis.postMessage = (message) => messages.push(message);
  state.executionMode = "parallel-wasm";
  state.jitEnabled = false;
  state.lastMetricsAt = 10_000;
  state.running = true;
  state.uartNeedsGuestService = true;
  state.emulator = emulator;
  state.vcpuPool = {
    runRound(actual, slice) {
      assert.equal(actual, emulator);
      assert.equal(slice, INTERACTIVE_STEP_SLICE);
      rounds += 1;
    },
  };

  schedulePump();
  await ran;

  assert.equal(rounds, 3);
  assert.deepEqual(messages, [{ event: "uart", output: "x" }]);
});

test("responsive cooperative pump executes exactly one round per frame", async () => {
  let rounds = 0;
  let resolveRun;
  const ran = new Promise((resolve) => (resolveRun = resolve));
  globalThis.performance = { now: () => 1000 };
  globalThis.postMessage = () => {};
  state.executionMode = "cooperative";
  state.jitEnabled = false;
  state.lastMetricsAt = 10_000;
  state.running = true;
  state.uartNeedsGuestService = true;
  state.emulator = {
    install_disk_generation() {
      state.running = false;
      resolveRun();
      return state.lastAutosaveGeneration;
    },
    run_kernel(slice) {
      assert.equal(slice, INTERACTIVE_STEP_SLICE);
      rounds += 1;
    },
    uart_output_len: () => 0,
  };

  schedulePump();
  await ran;

  assert.equal(rounds, 1);
});
