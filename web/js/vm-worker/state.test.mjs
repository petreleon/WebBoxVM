import assert from "node:assert/strict";
import test from "node:test";
import { DEFAULT_JIT_ENABLED, resetJitState, state } from "./state.js";

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
