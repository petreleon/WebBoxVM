import assert from "node:assert/strict";
import test from "node:test";
import { drainUart, interpreterStepSlice, shouldContinuePumpFrame, shouldFlushUart, shouldPollUart } from "./pump.js";
import {
  DEFAULT_JIT_ENABLED,
  DEFAULT_STEP_SLICE,
  MAX_FRAME_BATCHES,
  MAX_FRAME_MS,
  NETWORK_IDLE_FAST_MS,
  NETWORK_STEP_SLICE,
  UART_POLL_INTERVAL_MS,
  resetJitState,
  state,
} from "./state.js";

test("browser worker starts with jit disabled for installer safety", () => {
  assert.equal(DEFAULT_JIT_ENABLED, false);
  assert.equal(state.jitEnabled, false);
  assert.equal(state.stepSlice, DEFAULT_STEP_SLICE);
});

test("jit cache reset preserves an explicit manual jit toggle", () => {
  state.jitBlocks.set("0:1000", {});
  state.jitBlockHits.set("0:1000", 3);
  state.jitRejectedBlocks.add("0:2000");
  state.jitSkippedBlocks.add("0:3000");
  state.jitEnabled = true;

  resetJitState();

  assert.equal(state.jitEnabled, true);
  assert.equal(state.jitBlocks.size, 0);
  assert.equal(state.jitBlockHits.size, 0);
  assert.equal(state.jitRejectedBlocks.size, 0);
  assert.equal(state.jitSkippedBlocks.size, 0);
  state.jitEnabled = DEFAULT_JIT_ENABLED;
});

test("connected network caps interpreter step slices for TCP responsiveness", () => {
  state.jitEnabled = false;
  state.networkStatus = "connected";
  state.stepSlice = 50_000_000;
  state.lastNetworkActivityAt = performance.now();

  assert.equal(interpreterStepSlice(), NETWORK_STEP_SLICE);

  state.networkStatus = "offline";
  state.stepSlice = DEFAULT_STEP_SLICE;
});

test("recent network activity skips pending tx polling", () => {
  let pendingPolls = 0;
  state.jitEnabled = false;
  state.networkStatus = "connected";
  state.stepSlice = 50_000_000;
  state.lastNetworkActivityAt = performance.now();
  state.emulator = {
    network_tx_pending: () => {
      pendingPolls += 1;
      return 0;
    },
  };

  assert.equal(interpreterStepSlice(), NETWORK_STEP_SLICE);
  assert.equal(pendingPolls, 0);

  state.emulator = undefined;
  state.networkStatus = "offline";
  state.stepSlice = DEFAULT_STEP_SLICE;
});

test("network responsiveness can reuse a caller timestamp", () => {
  let pendingPolls = 0;
  state.jitEnabled = false;
  state.networkStatus = "connected";
  state.stepSlice = 50_000_000;
  state.lastNetworkActivityAt = 1000;
  state.emulator = {
    network_tx_pending: () => {
      pendingPolls += 1;
      return 0;
    },
  };

  assert.equal(interpreterStepSlice(1000 + NETWORK_IDLE_FAST_MS - 1), NETWORK_STEP_SLICE);
  assert.equal(pendingPolls, 0);

  state.emulator = undefined;
  state.networkStatus = "offline";
  state.stepSlice = DEFAULT_STEP_SLICE;
});

test("idle connected network allows fast interpreter step slices", () => {
  state.jitEnabled = false;
  state.networkStatus = "connected";
  state.lastNetworkActivityAt = performance.now() - 10_000;
  state.stepSlice = 50_000_000;

  assert.equal(interpreterStepSlice(), 50_000_000);

  state.networkStatus = "offline";
  state.stepSlice = DEFAULT_STEP_SLICE;
});

test("pending network transmit keeps responsive interpreter slices", () => {
  state.jitEnabled = false;
  state.networkStatus = "connected";
  state.lastNetworkActivityAt = performance.now() - 10_000;
  state.stepSlice = 50_000_000;
  state.emulator = { network_tx_pending: () => 1 };

  assert.equal(interpreterStepSlice(), NETWORK_STEP_SLICE);

  state.emulator = undefined;
  state.networkStatus = "offline";
  state.stepSlice = DEFAULT_STEP_SLICE;
});

test("pump allows more cached jit batches inside the frame budget", () => {
  assert.equal(MAX_FRAME_BATCHES, 32);
  assert.equal(shouldContinuePumpFrame(100, 100 + MAX_FRAME_MS - 1, 31), true);
});

test("pump yields on frame time or batch cap", () => {
  assert.equal(shouldContinuePumpFrame(100, 100 + MAX_FRAME_MS, 0), false);
  assert.equal(shouldContinuePumpFrame(100, 100, MAX_FRAME_BATCHES), false);
});

test("uart flushing batches small bursts for terminal throughput", () => {
  assert.equal(shouldFlushUart(16, 40, 0), false);
  assert.equal(shouldFlushUart(16, 55, 0), true);
});

test("uart flushing sends large chunks immediately", () => {
  assert.equal(shouldFlushUart(8192, 1, 0), true);
});

test("uart polling runs immediately then respects poll cadence", () => {
  assert.equal(shouldPollUart(5, 0), true);
  assert.equal(shouldPollUart(1000 + UART_POLL_INTERVAL_MS - 1, 1000), false);
  assert.equal(shouldPollUart(1000 + UART_POLL_INTERVAL_MS, 1000), true);
});

test("uart drain skips emulator length calls inside poll window", () => {
  let lenCalls = 0;
  state.lastUart = 0;
  state.lastUartPollAt = 1000;
  state.emulator = {
    uart_output_len: () => {
      lenCalls += 1;
      return 0;
    },
  };

  drainUart(1000 + UART_POLL_INTERVAL_MS - 1);

  assert.equal(lenCalls, 0);
  assert.equal(state.lastUartPollAt, 1000);

  state.emulator = undefined;
  state.lastUartPollAt = 0;
});
