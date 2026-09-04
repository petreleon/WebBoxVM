import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { tryRunCachedJitBlock } from "./jit-run.js?v=20260904-virgl-depth-compare-r1";
import { resetJitState, state } from "./state.js?v=20260904-virgl-depth-compare-r1";

afterEach(() => {
  resetJitState();
  state.emulator = undefined;
});

function cachedEntry() {
  return {
    alternateExitPc: 0n,
    dynamicExit: false,
    exitPc: 0x1010n,
    memoryGeneration: 4n,
    rawHash: 1n,
    run: () => 0x1010n,
    startPageGeneration: 2n,
    endPageGeneration: 3n,
    startPa: 0x2000n,
    startPc: 0x1000n,
    statePtr: 0x3000n,
    steps: 4,
  };
}

test("cached jit fast path returns true on commit", () => {
  let runStatePtr;
  state.emulator = {
    jit_finish_cached_block: () => 0,
    jit_last_error: () => "",
    jit_prepare_cached_block: () => true,
    pc: () => 0x1000n,
  };
  const entry = cachedEntry();
  entry.run = (statePtr) => {
    runStatePtr = statePtr;
    return 0x1010n;
  };

  const result = tryRunCachedJitBlock(0, 0x1000n, entry, 0x1000n);

  assert.equal(result, true);
  assert.equal(runStatePtr, 0x3000n);
});

test("cached jit fast path still returns failure details", () => {
  state.emulator = {
    jit_finish_cached_block: () => 2,
    jit_last_error: () => "JIT block crosses an unmasked pending IRQ boundary",
    jit_prepare_cached_block: () => true,
    pc: () => 0x1000n,
  };

  const result = tryRunCachedJitBlock(0, 0x1000n, cachedEntry(), 0x1000n);

  assert.equal(result.committed, false);
  assert.equal(result.error, "JIT block crosses an unmasked pending IRQ boundary");
  assert.equal(result.steps, 4);
});
