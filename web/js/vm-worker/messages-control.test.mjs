import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { withEmulatorAccess } from "./access.js?v=20260904-virgl-readback-pool-r1";
import { handleMessage } from "./messages.js?v=20260904-virgl-readback-pool-r1";
import { resetJitState, state } from "./state.js?v=20260904-virgl-readback-pool-r1";

afterEach(() => {
  state.emulator = undefined;
  state.executionMode = "cooperative";
  state.jitEnabled = false;
  state.numCores = 0;
  state.parallelTransitionDeferred = false;
  state.pumpScheduled = false;
  state.running = false;
  state.threadedWasm = undefined;
  state.vcpuPool = undefined;
  resetJitState();
});

test("setJitEnabled resets cached jit state and stale telemetry", async () => {
  resetJitState();
  state.jitBlockHits.set("0:1000", 2);
  state.jitBlocks.set("0:1000", {});
  state.jitFallbackCount = 1;
  state.jitLastFallback = { error: "old fallback" };
  state.jitRejectedBlocks.add("0:2000");
  state.jitRejectLog.push({ error: "old reject" });
  state.jitSkippedBlocks.add("0:3000");
  state.jitSkipLog.push({ error: "old skip" });

  const messages = [];
  await withPostMessage(messages, () =>
    handleMessage({ id: 7, payload: { enabled: true }, type: "setJitEnabled" }),
  );

  assert.equal(state.jitEnabled, true);
  assert.equal(state.jitBlockHits.size, 0);
  assert.equal(state.jitBlocks.size, 0);
  assert.equal(state.jitFallbackCount, 0);
  assert.equal(state.jitLastFallback, undefined);
  assert.equal(state.jitRejectedBlocks.size, 0);
  assert.equal(state.jitRejectLog.length, 0);
  assert.equal(state.jitSkippedBlocks.size, 0);
  assert.equal(state.jitSkipLog.length, 0);
  assert.deepEqual(messages, [{ id: 7, ok: true, value: {} }]);
});

test("gpu3dAck reaches its normal and readback wasm completion boundaries", async () => {
  const acknowledgments = [];
  state.emulator = { gpu_3d_complete: (...values) => { acknowledgments.push(values); return true; } };
  await withPostMessage([], () =>
    handleMessage({ payload: { sequence: 19, success: false }, type: "gpu3dAck" }),
  );
  assert.deepEqual(acknowledgments, [[19, false]]);
  const pixels = new Uint8Array([1, 2, 3, 4]);
  await withPostMessage([], () => handleMessage({
    payload: { readback: { format: 1, pixels }, sequence: 20, success: true }, type: "gpu3dAck",
  }));
  assert.deepEqual(acknowledgments, [[19, false], [20, true]]);
  state.emulator.gpu_3d_complete_readback = (...values) => { acknowledgments.push(values); return true; };
  await withPostMessage([], () => handleMessage({
    payload: { readback: { format: 1, pixels }, sequence: 21, success: true }, type: "gpu3dAck",
  }));
  assert.deepEqual(acknowledgments, [[19, false], [20, true], [21, 1, pixels]]);
});

test("gpu3dAck reports an unaccepted completion without mutating Rust state", async () => {
  state.emulator = { gpu_3d_complete: () => false };
  const messages = [];
  await withPostMessage(messages, () =>
    handleMessage({ id: 20, payload: { sequence: 21, success: true }, type: "gpu3dAck" }),
  );
  assert.deepEqual(messages, [{ id: 20, ok: true, value: { accepted: false } }]);
});

test("a stale one-way gpu3dAck after device reset is not a global worker error", async () => {
  state.emulator = { gpu_3d_complete: () => false };
  const messages = [];
  await withPostMessage(messages, () =>
    handleMessage({ payload: { sequence: 21, success: true }, type: "gpu3dAck" }),
  );
  assert.deepEqual(messages, []);
});

test("gpu3dAck fails closed when the wasm completion export is missing", async () => {
  state.emulator = {};
  const messages = [];
  await withPostMessage(messages, () =>
    handleMessage({ id: 18, payload: { sequence: 20, success: true }, type: "gpu3dAck" }),
  );
  assert.deepEqual(messages, [{
    error: "Worker VM wasm export gpu_3d_complete is unavailable",
    id: 18,
    ok: false,
  }]);
});

test("requests wait until a parallel pump relinquishes emulator access", async () => {
  let releasePump;
  let markPumpStarted;
  const pumpStarted = new Promise((resolve) => {
    markPumpStarted = resolve;
  });
  const pump = withEmulatorAccess(async () => {
    markPumpStarted();
    await new Promise((resolve) => {
      releasePump = resolve;
    });
  });
  await pumpStarted;
  let reads = 0;
  state.emulator = {
    current_instruction: () => {
      reads += 1;
      return "instruction";
    },
  };
  const messages = [];
  const request = withPostMessage(messages, () =>
    handleMessage({ id: 8, payload: { coreId: 0 }, type: "currentInstruction" }),
  );

  await Promise.resolve();
  assert.equal(reads, 0);
  releasePump();
  await Promise.all([pump, request]);

  assert.equal(reads, 1);
  assert.deepEqual(messages, [{ id: 8, ok: true, value: "instruction" }]);
});

test("parallel transition waits for pump ownership to become quiescent", async () => {
  let releasePump;
  let markPumpStarted;
  const pumpStarted = new Promise((resolve) => {
    markPumpStarted = resolve;
  });
  const pump = withEmulatorAccess(async () => {
    markPumpStarted();
    await new Promise((resolve) => {
      releasePump = resolve;
    });
  });
  await pumpStarted;
  state.executionMode = "cooperative-jit";
  state.jitEnabled = true;
  state.numCores = 2;
  state.parallelTransitionDeferred = true;
  state.threadedWasm = undefined;
  const messages = [];

  const transition = withPostMessage(messages, () =>
    handleMessage({ id: 9, type: "transitionToParallel" }),
  );
  await Promise.resolve();

  assert.deepEqual(messages, []);
  assert.equal(state.parallelTransitionDeferred, true);
  releasePump();
  await Promise.all([pump, transition]);

  assert.equal(state.executionMode, "cooperative-jit");
  assert.equal(state.jitEnabled, true);
  assert.equal(state.parallelTransitionDeferred, false);
  assert.equal(messages.length, 1);
  assert.equal(messages[0].id, 9);
  assert.equal(messages[0].ok, true);
});

async function withPostMessage(messages, run) {
  const previousPostMessage = globalThis.postMessage;
  globalThis.postMessage = (message) => messages.push(message);
  try {
    return await run();
  } finally {
    globalThis.postMessage = previousPostMessage;
  }
}
