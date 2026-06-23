import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { compileJitBlock, jitBlockKey } from "./jit-compile.js";
import { state } from "./state.js";

afterEach(() => {
  state.emulator = undefined;
  state.wasmExports = undefined;
  state.jitBlocks = new Map();
  state.jitBlockHits = new Map();
  state.jitImports = undefined;
  state.jitRejectedBlocks = new Set();
  state.jitSkippedBlocks = new Set();
  state.jitStatePtr = undefined;
  state.jitStateSize = undefined;
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

test("jit block keys avoid string allocation for core zero", () => {
  assert.equal(jitBlockKey(0, 0x1000n), 0x1000n);
  assert.equal(jitBlockKey(2, 0x1000n), "2:1000");
});

test("compile jit block wires pair memory helper imports", async () => {
  const originalInstantiate = WebAssembly.instantiate;
  let pairLoadArgs;
  let pairStoreArgs;
  let quadLoadArgs;
  let quadStoreArgs;
  const importObjects = [];
  let statePtrCalls = 0;
  let stateSizeCalls = 0;
  let metadataCalls = 0;
  const unusedMetadata = () => assert.fail("packed JIT metadata should be used");
  const emulator = {
    jit_compile_current_block: () => new Uint8Array([1, 2, 3]),
    jit_last_block_alternate_exit_pc: unusedMetadata,
    jit_last_block_dynamic_exit: unusedMetadata,
    jit_last_block_el: () => assert.fail("unused skip metadata should not be read"),
    jit_last_block_end_page_generation: unusedMetadata,
    jit_last_block_exit_pc: unusedMetadata,
    jit_last_block_metadata: () => {
      metadataCalls += 1;
      return [1n, 0x1000n, 0x4000_1000n, 0x1004n, 0x1004n, 0n, 7n, 9n, 0n, 0n];
    },
    jit_last_block_raw_hash: unusedMetadata,
    jit_last_block_start_pa: unusedMetadata,
    jit_last_block_start_page_generation: unusedMetadata,
    jit_last_block_start_pc: unusedMetadata,
    jit_last_block_steps: unusedMetadata,
    jit_last_block_uses_guest_helpers: () =>
      assert.fail("unused skip metadata should not be read"),
    jit_state_ptr: () => {
      statePtrCalls += 1;
      return 0x2000n;
    },
    jit_state_size: () => {
      stateSizeCalls += 1;
      return 512;
    },
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
    importObjects.push(imports);
    assert.deepEqual(imports.env.jitLoadPairGuest(0x20n, 8), [0xaabbn, 0xccddn]);
    imports.env.jitStorePairGuest(0x10n, 8, 0x1122n, 0x3344n);
    assert.deepEqual(imports.env.jitLoadQuadGuest(0x30n, 8), [1n, 2n, 3n, 4n]);
    imports.env.jitStoreQuadGuest(0x40n, 8, 1n, 2n, 3n, 4n);
    return { instance: { exports: { run: () => 0x1004n } }, module: {} };
  };

  try {
    const result = await compileJitBlock({ coreId: 3, pc: 0x1000n, emulator });
    const cachedResult = await compileJitBlock({ coreId: 3, pc: 0x1000n, emulator });

    assert.equal(result.compiled, true);
    assert.equal(result.statePtr, 0x2000n);
    assert.equal(cachedResult.statePtr, 0x2000n);
    assert.equal(statePtrCalls, 1);
    assert.equal(result.stateSize, 512);
    assert.equal(cachedResult.stateSize, 512);
    assert.equal(importObjects[0], importObjects[1]);
    const entry = state.jitBlocks.get("3:1000");
    assert.equal(entry.instance, undefined);
    assert.equal(entry.module, undefined);
    assert.equal(typeof entry.run, "function");
    assert.equal(entry.memoryGeneration, 9n);
    assert.equal(metadataCalls, 2);
    assert.equal(stateSizeCalls, 1);
    assert.deepEqual(pairLoadArgs, [3, 0x20n, 8]);
    assert.deepEqual(pairStoreArgs, [3, 0x10n, 8, 0x1122n, 0x3344n]);
    assert.deepEqual(quadLoadArgs, [3, 0x30n, 8]);
    assert.deepEqual(quadStoreArgs, [3, 0x40n, 8, 1n, 2n, 3n, 4n]);
  } finally {
    WebAssembly.instantiate = originalInstantiate;
  }
});
