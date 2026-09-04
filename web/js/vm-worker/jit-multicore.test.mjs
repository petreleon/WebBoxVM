import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { tryRunOrCompileNextJitBlock } from "./jit-hot.js?v=20260904-virgl-depth-r1";
import { DEFAULT_JIT_ENABLED, resetJitState, state } from "./state.js?v=20260904-virgl-depth-r1";

afterEach(() => {
  resetJitState();
  state.emulator = undefined;
  state.jitEnabled = DEFAULT_JIT_ENABLED;
});

test("cooperative multicore jit selects the runnable core and its cache key", () => {
  const selected = [];
  state.jitEnabled = true;
  state.emulator = {
    jit_finish_cached_block: (coreId) => {
      selected.push(["finish", coreId]);
      return 0;
    },
    jit_last_error: () => "",
    jit_prepare_cached_block: (coreId) => {
      selected.push(["block", coreId]);
      return true;
    },
    jit_prepare_next_core: () => {
      selected.push(["scheduler"]);
      return 1;
    },
    pc: () => assert.fail("core-zero PC must not be used for vCPU 1"),
    pc_for_core: (coreId) => {
      selected.push(["pc", coreId]);
      return 0x2000n;
    },
  };
  state.jitBlocks.set("1:2000", cachedEntry());

  assert.equal(tryRunOrCompileNextJitBlock(), true);
  assert.deepEqual(selected, [
    ["scheduler"],
    ["pc", 1],
    ["block", 1],
    ["finish", 1],
  ]);
});

test("cooperative multicore jit falls back when no core is runnable", () => {
  state.jitEnabled = true;
  state.emulator = {
    jit_prepare_next_core: () => -1,
    pc_for_core: () => assert.fail("no PC should be read without a runnable core"),
  };

  assert.equal(tryRunOrCompileNextJitBlock(), false);
});

function cachedEntry() {
  return {
    alternateExitPc: 0n,
    dynamicExit: false,
    exitPc: 0x2004n,
    memoryGeneration: 4n,
    rawHash: 1n,
    run: () => 0x2004n,
    startPageGeneration: 2n,
    endPageGeneration: 3n,
    startPa: 0x2000n,
    startPc: 0x2000n,
    statePtr: 0x3000n,
    steps: 1,
  };
}
