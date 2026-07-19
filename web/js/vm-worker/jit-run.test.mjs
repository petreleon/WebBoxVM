import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { runCachedJitBlock } from "./jit-run.js?v=20260718-staged-fast-boot";
import { resetJitState, state } from "./state.js?v=20260718-staged-fast-boot";

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
  entry.run = () => {
    ran = true;
    return 0x1010n;
  };
  const result = runCachedJitBlock(0, "0:1000", entry);

  assert.equal(result.committed, false);
  assert.equal(result.error, "JIT block crosses timer deadline");
  assert.equal(result.invalidated, undefined);
  assert.deepEqual(prepareArgs, [0, 0x1000n, 0x2000n, 1n, 4n, 2n, 3n, 4]);
  assert.equal(ran, false);
});

test("cached jit prepare receives metadata and run uses cached state pointer", () => {
  let prepareArgs;
  let finishArgs;
  let runStatePtr;
  state.emulator = {
    jit_finish_cached_block: (...args) => {
      finishArgs = args;
      return 0;
    },
    jit_last_error: () => "",
    jit_prepare_cached_block: (...args) => {
      prepareArgs = args;
      return true;
    },
    pc: () => 0x1000n,
  };
  const entry = cachedEntry();
  entry.run = (statePtr) => {
    runStatePtr = statePtr;
    return 0x1010n;
  };

  const result = runCachedJitBlock(0, "0:1000", entry);

  assert.equal(result.committed, true);
  assert.equal(runStatePtr, 0x3000n);
  assert.deepEqual(prepareArgs, [0, 0x1000n, 0x2000n, 1n, 4n, 2n, 3n, 4]);
  assert.deepEqual(finishArgs, [0, 4, 0x1010n, 0x1010n, 0n, false]);
});

test("cached jit run can reuse caller pc without another wasm lookup", () => {
  let pcCalls = 0;
  const previousDescriptor = Object.getOwnPropertyDescriptor(state, "emulator");
  let emulatorReads = 0;
  const emulator = {
    jit_finish_cached_block: () => 0,
    jit_last_error: () => "",
    jit_prepare_cached_block: () => true,
    pc: () => {
      pcCalls += 1;
      return 0x9999n;
    },
  };
  Object.defineProperty(state, "emulator", {
    configurable: true,
    get() {
      emulatorReads += 1;
      return undefined;
    },
  });

  try {
    const result = runCachedJitBlock(0, "0:1000", cachedEntry(), 0x1000n, emulator);

    assert.equal(result.committed, true);
    assert.equal(result.pc, 0x1000n);
    assert.equal(pcCalls, 0);
    assert.equal(emulatorReads, 0);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});

test("finish helper rejection invalidates cached jit block", () => {
  state.emulator = {
    jit_finish_cached_block: () => 1,
    jit_last_error: () => "JIT helper failed",
    jit_prepare_cached_block: () => true,
    pc: () => 0x1000n,
  };

  const result = runCachedJitBlock(0, "0:1000", cachedEntry());

  assert.equal(result.committed, false);
  assert.equal(result.error, "JIT helper failed");
  assert.equal(result.invalidated, true);
  assert.equal(result.rejected, true);
});

test("finish exit rejection deletes cached jit block", () => {
  state.emulator = {
    jit_finish_cached_block: () => 3,
    jit_last_error: () => "JIT block returned 0x1020 instead of 0x1010",
    jit_prepare_cached_block: () => true,
    pc: () => 0x1000n,
  };
  const entry = cachedEntry();
  state.jitBlocks.set("0:1000", entry);

  const result = runCachedJitBlock(0, "0:1000", entry);

  assert.equal(result.committed, false);
  assert.equal(result.invalidated, true);
  assert.equal(state.jitBlocks.has("0:1000"), false);
});

test("finish commit rejection keeps cached jit block", () => {
  state.emulator = {
    jit_finish_cached_block: () => 2,
    jit_last_error: () => "JIT block crosses an unmasked pending IRQ boundary",
    jit_prepare_cached_block: () => true,
    pc: () => 0x1000n,
  };
  const entry = cachedEntry();
  state.jitBlocks.set("0:1000", entry);

  const result = runCachedJitBlock(0, "0:1000", entry);

  assert.equal(result.committed, false);
  assert.equal(result.invalidated, undefined);
  assert.equal(result.steps, 4);
  assert.equal(state.jitBlocks.has("0:1000"), true);
});
