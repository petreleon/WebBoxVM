import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { tryRunOrCompileJitBlock } from "./jit-hot.js?v=20260904-virgl-material-batch-r1";
import { DEFAULT_JIT_ENABLED, JIT_MAX_HIT_SITES, resetJitState, state } from "./state.js?v=20260904-virgl-material-batch-r1";

afterEach(() => {
  resetJitState();
  state.emulator = undefined;
  state.jitEnabled = DEFAULT_JIT_ENABLED;
});

test("jit warmup hits evict oldest one-hit site when capped", () => {
  let pc = 0x1000n;
  state.jitEnabled = true;
  state.emulator = {
    pc: () => {
      const next = pc;
      pc += 4n;
      return next;
    },
  };

  for (let index = 0; index <= JIT_MAX_HIT_SITES; index += 1) {
    assert.equal(tryRunOrCompileJitBlock(), false);
  }

  assert.equal(state.jitBlockHits.size, JIT_MAX_HIT_SITES);
  assert.equal(state.jitBlockHits.has(0x1000n), false);
  assert.equal(state.jitBlockHits.has(0x1004n), true);
});
