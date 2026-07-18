import assert from "node:assert/strict";
import test from "node:test";
import { withEmulatorAccess } from "./access.js";
import { handleMessage } from "./messages.js";
import { resetJitState, state } from "./state.js";

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

  state.jitEnabled = false;
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
  state.emulator = undefined;
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
