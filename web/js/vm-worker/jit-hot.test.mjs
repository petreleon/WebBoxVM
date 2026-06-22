import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { tryRunOrCompileJitBlock } from "./jit-hot.js";
import { DEFAULT_JIT_ENABLED, resetJitState, state } from "./state.js";

afterEach(() => {
  resetJitState();
  state.emulator = undefined;
  state.jitEnabled = DEFAULT_JIT_ENABLED;
  state.running = false;
  state.wasmExports = undefined;
});

function cachedEntry() {
  return {
    alternateExitPc: 0n,
    dynamicExit: false,
    exitPc: 0x1004n,
    instance: { exports: { run: () => 0x1004n } },
    rawHash: 1n,
    startPageGeneration: 2n,
    endPageGeneration: 3n,
    startPa: 0x2000n,
    startPc: 0x1000n,
    statePtr: 0x3000n,
    steps: 1,
  };
}

test("cached jit hot path reads pc once per hit", async () => {
  let pcCalls = 0;
  state.jitEnabled = true;
  state.emulator = {
    jit_finish_cached_block: () => 0,
    jit_last_error: () => "",
    jit_prepare_cached_block: () => true,
    pc: () => {
      pcCalls += 1;
      return 0x1000n;
    },
  };
  state.jitBlocks.set("0:1000", cachedEntry());

  assert.equal(await tryRunOrCompileJitBlock(), true);
  assert.equal(pcCalls, 1);
});

test("jit compile fallback reuses hot-path pc", async () => {
  let pcCalls = 0;
  state.jitEnabled = true;
  state.running = true;
  state.wasmExports = { memory: {} };
  state.emulator = {
    current_instruction: () => "",
    jit_compile_current_block: () => [],
    jit_last_error: () => "compile miss",
    pc: () => {
      pcCalls += 1;
      return 0x1000n;
    },
  };

  assert.equal(await tryRunOrCompileJitBlock(), false);
  assert.equal(await tryRunOrCompileJitBlock(), false);
  assert.equal(pcCalls, 2);
});
