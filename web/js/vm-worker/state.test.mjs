import assert from "node:assert/strict";
import test from "node:test";
import { interpreterStepSlice, shouldContinuePumpFrame } from "./pump.js?v=20260904-virgl-depth-vertex-color-r1";
import {
  COOPERATIVE_STEP_SLICE,
  DEFAULT_JIT_ENABLED,
  DEFAULT_STEP_SLICE,
  JIT_PROBE_STEP_SLICE,
  MAX_FRAME_BATCHES,
  MAX_FRAME_MS,
  NETWORK_IDLE_FAST_MS,
  NETWORK_STEP_SLICE,
  resetJitState,
  state,
} from "./state.js?v=20260904-virgl-depth-vertex-color-r1";

test("browser worker starts with jit disabled for installer safety", () => {
  assert.equal(DEFAULT_JIT_ENABLED, false);
  assert.equal(state.jitEnabled, false);
  assert.equal(state.stepSlice, DEFAULT_STEP_SLICE);
  assert.equal(DEFAULT_STEP_SLICE, 5_000_000);
  assert.equal(JIT_PROBE_STEP_SLICE, COOPERATIVE_STEP_SLICE);
  assert.equal(NETWORK_STEP_SLICE, 1_000_000);
});

test("jit cache reset preserves an explicit manual jit toggle", () => {
  state.jitBlocks.set("0:1000", {});
  state.jitBlockHits.set("0:1000", 3);
  state.jitImports = {};
  state.jitRejectedBlocks.add("0:2000");
  state.jitSkippedBlocks.add("0:3000");
  state.jitStatePtr = 0x2000n;
  state.jitStateSize = 512;
  state.jitEnabled = true;

  resetJitState();

  assert.equal(state.jitEnabled, true);
  assert.equal(state.jitBlocks.size, 0);
  assert.equal(state.jitBlockHits.size, 0);
  assert.equal(state.jitImports, undefined);
  assert.equal(state.jitRejectedBlocks.size, 0);
  assert.equal(state.jitSkippedBlocks.size, 0);
  assert.equal(state.jitStatePtr, undefined);
  assert.equal(state.jitStateSize, undefined);
  state.jitEnabled = DEFAULT_JIT_ENABLED;
});

test("connected network caps interpreter step slices for TCP responsiveness", () => {
  state.jitEnabled = false;
  state.networkStatus = "connected";
  state.stepSlice = 50_000_000;
  state.lastNetworkActivityAt = performance.now();

  assert.equal(interpreterStepSlice(), COOPERATIVE_STEP_SLICE);

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

  assert.equal(interpreterStepSlice(), COOPERATIVE_STEP_SLICE);
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

  assert.equal(
    interpreterStepSlice(1000 + NETWORK_IDLE_FAST_MS - 1),
    COOPERATIVE_STEP_SLICE,
  );
  assert.equal(pendingPolls, 0);

  state.emulator = undefined;
  state.networkStatus = "offline";
  state.stepSlice = DEFAULT_STEP_SLICE;
});

test("idle connected network keeps the cooperative latency bound", () => {
  state.jitEnabled = false;
  state.networkStatus = "connected";
  state.lastNetworkActivityAt = performance.now() - 10_000;
  state.stepSlice = 50_000_000;

  assert.equal(interpreterStepSlice(), COOPERATIVE_STEP_SLICE);

  state.networkStatus = "offline";
  state.stepSlice = DEFAULT_STEP_SLICE;
});

test("pending network transmit keeps responsive interpreter slices", () => {
  state.jitEnabled = false;
  state.networkStatus = "connected";
  state.lastNetworkActivityAt = performance.now() - 10_000;
  state.stepSlice = 50_000_000;
  state.emulator = { network_tx_pending: () => 1 };

  assert.equal(interpreterStepSlice(), COOPERATIVE_STEP_SLICE);

  state.emulator = undefined;
  state.networkStatus = "offline";
  state.stepSlice = DEFAULT_STEP_SLICE;
});

test("pump bounds steady-state cached jit residency", () => {
  assert.equal(MAX_FRAME_MS, 32);
  assert.equal(MAX_FRAME_BATCHES, 128);
  assert.equal(shouldContinuePumpFrame(100, 100 + MAX_FRAME_MS - 1, 127), true);
});

test("pump yields on frame time or batch cap", () => {
  assert.equal(shouldContinuePumpFrame(100, 100 + MAX_FRAME_MS, 0), false);
  assert.equal(shouldContinuePumpFrame(100, 100, MAX_FRAME_BATCHES), false);
});

test("interactive parallel pump can service more guest work inside one bounded frame", () => {
  assert.equal(shouldContinuePumpFrame(100, 101, 1, true, "parallel-wasm"), true);
  assert.equal(shouldContinuePumpFrame(100, 101, 1, true, "cooperative"), false);
});
