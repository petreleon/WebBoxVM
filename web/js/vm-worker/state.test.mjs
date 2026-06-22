import assert from "node:assert/strict";
import test from "node:test";
import { interpreterStepSlice, shouldFlushUart } from "./pump.js";
import { DEFAULT_JIT_ENABLED, NETWORK_STEP_SLICE, resetJitState, state } from "./state.js";

test("browser worker starts with jit disabled for installer safety", () => {
  assert.equal(DEFAULT_JIT_ENABLED, false);
  assert.equal(state.jitEnabled, false);
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
  state.stepSlice = 1_000_000;
});

test("idle connected network allows fast interpreter step slices", () => {
  state.jitEnabled = false;
  state.networkStatus = "connected";
  state.lastNetworkActivityAt = performance.now() - 10_000;
  state.stepSlice = 50_000_000;

  assert.equal(interpreterStepSlice(), 50_000_000);

  state.networkStatus = "offline";
  state.stepSlice = 1_000_000;
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
  state.stepSlice = 1_000_000;
});

test("uart flushing batches small bursts for terminal throughput", () => {
  assert.equal(shouldFlushUart(16, 40, 0), false);
  assert.equal(shouldFlushUart(16, 55, 0), true);
});

test("uart flushing sends large chunks immediately", () => {
  assert.equal(shouldFlushUart(8192, 1, 0), true);
});
