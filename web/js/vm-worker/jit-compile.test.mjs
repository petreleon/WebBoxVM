import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { compileJitBlock, compiledJitBlockSkipReason } from "./jit-compile.js";
import { state } from "./state.js";

afterEach(() => {
  state.emulator = undefined;
  state.wasmExports = undefined;
  state.jitBlocks = new Map();
  state.jitBlockHits = new Map();
  state.jitRejectedBlocks = new Set();
  state.jitSkippedBlocks = new Set();
});

test("compile jit block reuses caller pc on fallback", async () => {
  let pcCalls = 0;
  state.wasmExports = { memory: {} };
  state.emulator = {
    jit_compile_current_block: () => [],
    jit_last_error: () => "no block",
    pc: () => {
      pcCalls += 1;
      return 0x9999n;
    },
  };

  const result = await compileJitBlock({ coreId: 0, pc: 0x1000n });

  assert.equal(result.compiled, false);
  assert.equal(result.pc, 0x1000n);
  assert.equal(pcCalls, 0);
});

test("compile jit block can reuse checked emulator reference", async () => {
  const previousDescriptor = Object.getOwnPropertyDescriptor(state, "emulator");
  let emulatorReads = 0;
  const emulator = {
    jit_compile_current_block: () => [],
    jit_last_error: () => "no block",
    pc: () => 0x9999n,
  };
  state.wasmExports = { memory: {} };
  Object.defineProperty(state, "emulator", {
    configurable: true,
    get() {
      emulatorReads += 1;
      return undefined;
    },
  });

  try {
    const result = await compileJitBlock({ coreId: 0, pc: 0x1000n, emulator });

    assert.equal(result.compiled, false);
    assert.equal(result.pc, 0x1000n);
    assert.equal(emulatorReads, 0);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});

test("compile jit block wires pair memory helper imports", async () => {
  const originalInstantiate = WebAssembly.instantiate;
  let pairLoadArgs;
  let pairStoreArgs;
  let quadLoadArgs;
  let quadStoreArgs;
  const emulator = {
    jit_compile_current_block: () => new Uint8Array([1, 2, 3]),
    jit_last_block_alternate_exit_pc: () => 0x1004n,
    jit_last_block_dynamic_exit: () => false,
    jit_last_block_el: () => 1,
    jit_last_block_end_page_generation: () => 0n,
    jit_last_block_exit_pc: () => 0x1004n,
    jit_last_block_raw_hash: () => 7n,
    jit_last_block_start_pa: () => 0x4000_1000n,
    jit_last_block_start_page_generation: () => 0n,
    jit_last_block_start_pc: () => 0x1000n,
    jit_last_block_steps: () => 1,
    jit_last_block_uses_guest_helpers: () => true,
    jit_state_ptr: () => 0x2000n,
    jit_state_size: () => 512,
    jit_load_pair_guest: (...args) => {
      pairLoadArgs = args;
      return [0xaabbn, 0xccddn];
    },
    jit_store_pair_guest: (...args) => {
      pairStoreArgs = args;
    },
    jit_load_quad_guest: (...args) => {
      quadLoadArgs = args;
      return [1n, 2n, 3n, 4n];
    },
    jit_store_quad_guest: (...args) => {
      quadStoreArgs = args;
    },
    pc: () => 0x1000n,
  };
  state.emulator = emulator;
  state.wasmExports = { memory: {} };
  WebAssembly.instantiate = async (_bytes, imports) => {
    assert.deepEqual(imports.env.jitLoadPairGuest(0x20n, 8), [0xaabbn, 0xccddn]);
    imports.env.jitStorePairGuest(0x10n, 8, 0x1122n, 0x3344n);
    assert.deepEqual(imports.env.jitLoadQuadGuest(0x30n, 8), [1n, 2n, 3n, 4n]);
    imports.env.jitStoreQuadGuest(0x40n, 8, 1n, 2n, 3n, 4n);
    return { instance: {}, module: {} };
  };

  try {
    const result = await compileJitBlock({ coreId: 3, pc: 0x1000n, emulator });

    assert.equal(result.compiled, true);
    assert.deepEqual(pairLoadArgs, [3, 0x20n, 8]);
    assert.deepEqual(pairStoreArgs, [3, 0x10n, 8, 0x1122n, 0x3344n]);
    assert.deepEqual(quadLoadArgs, [3, 0x30n, 8]);
    assert.deepEqual(quadStoreArgs, [3, 0x40n, 8, 1n, 2n, 3n, 4n]);
  } finally {
    WebAssembly.instantiate = originalInstantiate;
  }
});

test("EL0 guest-memory helper blocks are not compile-time skipped", () => {
  assert.equal(
    compiledJitBlockSkipReason({ blockEl: 0, usesGuestHelpers: true }),
    undefined,
  );
});
