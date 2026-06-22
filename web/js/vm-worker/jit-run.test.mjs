import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { runCachedJitBlock } from "./jit-run.js";
import { resetJitState, state } from "./state.js";

afterEach(() => {
  resetJitState();
  state.emulator = undefined;
});

function cachedEntry() {
  return {
    alternateExitPc: 0n,
    dynamicExit: false,
    exitPc: 0x1010n,
    instance: {
      exports: {
        run: () => 0x1010n,
      },
    },
    rawHash: 1n,
    startPageGeneration: 2n,
    endPageGeneration: 3n,
    startPa: 0x2000n,
    startPc: 0x1000n,
    statePtr: 0x3000n,
    steps: 4,
  };
}

test("prepare failure falls back without running cached jit", () => {
  let ran = false;
  let prepareArgs;
  state.emulator = {
    jit_last_error: () => "JIT block crosses timer deadline",
    jit_prepare_cached_block: (...args) => {
      prepareArgs = args;
      return false;
    },
    pc: () => 0x1000n,
  };

  const entry = cachedEntry();
  entry.instance.exports.run = () => {
    ran = true;
    return 0x1010n;
  };
  const result = runCachedJitBlock(0, "0:1000", entry);

  assert.equal(result.committed, false);
  assert.equal(result.error, "JIT block crosses timer deadline");
  assert.equal(result.invalidated, undefined);
  assert.deepEqual(prepareArgs, [0, 0x1000n, 0x2000n, 1n, 2n, 3n, 4]);
  assert.equal(ran, false);
});

test("cached jit prepare receives metadata and run uses cached state pointer", () => {
  let prepareArgs;
  let runStatePtr;
  state.emulator = {
    jit_commit_state_to_core: () => true,
    jit_helper_failed: () => false,
    jit_last_error: () => "",
    jit_prepare_cached_block: (...args) => {
      prepareArgs = args;
      return true;
    },
    pc: () => 0x1000n,
  };
  const entry = cachedEntry();
  entry.instance.exports.run = (statePtr) => {
    runStatePtr = statePtr;
    return 0x1010n;
  };

  const result = runCachedJitBlock(0, "0:1000", entry);

  assert.equal(result.committed, true);
  assert.equal(runStatePtr, 0x3000n);
  assert.deepEqual(prepareArgs, [0, 0x1000n, 0x2000n, 1n, 2n, 3n, 4]);
});
