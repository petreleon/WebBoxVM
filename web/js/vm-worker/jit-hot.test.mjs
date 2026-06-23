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
    memoryGeneration: 4n,
    rawHash: 1n,
    run: () => 0x1004n,
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
  state.jitBlocks.set(0x1000n, cachedEntry());

  assert.equal(await tryRunOrCompileJitBlock(), true);
  assert.equal(pcCalls, 1);
});

test("cached jit hot path can reuse checked emulator reference", () => {
  const previousDescriptor = Object.getOwnPropertyDescriptor(state, "emulator");
  let emulatorReads = 0;
  const emulator = {
    jit_finish_cached_block: () => 0,
    jit_last_error: () => "",
    jit_prepare_cached_block: () => true,
    pc: () => 0x1000n,
  };
  state.jitEnabled = true;
  state.jitBlocks.set(0x1000n, cachedEntry());
  Object.defineProperty(state, "emulator", {
    configurable: true,
    get() {
      emulatorReads += 1;
      return undefined;
    },
  });

  try {
    assert.equal(tryRunOrCompileJitBlock(0, emulator), true);
    assert.equal(emulatorReads, 0);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});

test("cached jit hot path returns synchronously", () => {
  state.jitEnabled = true;
  state.emulator = {
    jit_finish_cached_block: () => 0,
    jit_last_error: () => "",
    jit_prepare_cached_block: () => true,
    pc: () => 0x1000n,
  };
  state.jitBlocks.set(0x1000n, cachedEntry());

  const result = tryRunOrCompileJitBlock();

  assert.equal(result, true);
  assert.equal(result?.then, undefined);
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
  assert.equal(state.jitBlockHits.get(0x1000n), 1);
  assert.equal(await tryRunOrCompileJitBlock(), false);
  assert.equal(pcCalls, 2);
  assert.equal(state.jitBlockHits.has(0x1000n), false);
  assert.equal(state.jitRejectedBlocks.has(0x1000n), true);
});

test("jit compile path remains asynchronous", () => {
  state.jitEnabled = true;
  state.running = true;
  state.wasmExports = { memory: {} };
  state.emulator = {
    current_instruction: () => "",
    jit_compile_current_block: () => [],
    jit_last_error: () => "compile miss",
    pc: () => 0x1000n,
  };

  assert.equal(tryRunOrCompileJitBlock(), false);
  const result = tryRunOrCompileJitBlock();

  assert.equal(typeof result.then, "function");
  return result.then((usedJit) => assert.equal(usedJit, false));
});

test("jit compile success drops warmup hit counter", async () => {
  const originalInstantiate = WebAssembly.instantiate;
  state.jitEnabled = true;
  state.running = true;
  state.wasmExports = { memory: {} };
  state.emulator = {
    jit_compile_current_block: () => new Uint8Array([1]),
    jit_finish_cached_block: () => 0,
    jit_last_block_metadata: () => [2n, 0x1000n, 0x2000n, 0x1004n, 0n, 0n, 1n, 4n, 2n, 3n],
    jit_last_error: () => "",
    jit_prepare_cached_block: () => true,
    jit_state_ptr: () => 0x3000n,
    jit_state_size: () => assert.fail("hot compile path does not need state size"),
    pc: () => 0x1000n,
  };
  WebAssembly.instantiate = async () => ({
    instance: { exports: { run: () => 0x1004n } },
    module: {},
  });

  try {
    assert.equal(tryRunOrCompileJitBlock(), false);
    assert.equal(await tryRunOrCompileJitBlock(), true);

    assert.equal(state.jitBlocks.has(0x1000n), true);
    assert.equal(state.jitBlockHits.has(0x1000n), false);
  } finally {
    WebAssembly.instantiate = originalInstantiate;
  }
});
