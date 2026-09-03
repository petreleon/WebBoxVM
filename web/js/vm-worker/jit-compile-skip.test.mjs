import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { compileJitBlockEntry } from "./jit-compile.js?v=20260903-virgl-blend-r1";
import { resetJitState, state } from "./state.js?v=20260903-virgl-blend-r1";

afterEach(() => {
  resetJitState();
  state.emulator = undefined;
  state.wasmExports = undefined;
});

test("single-instruction jit blocks are skipped before instantiation", async () => {
  let instantiateCalls = 0;
  let statePtrCalls = 0;
  const originalInstantiate = WebAssembly.instantiate;
  WebAssembly.instantiate = async () => {
    instantiateCalls += 1;
    return { instance: { exports: { run: () => 0x1004n } } };
  };
  const emulator = {
    jit_compile_current_block: () => new Uint8Array([1]),
    jit_last_block_metadata: () => [1n, 0x1000n, 0x2000n, 0x1004n, 0n, 0n, 1n, 4n, 2n, 3n],
    jit_state_ptr: () => {
      statePtrCalls += 1;
      return 0x3000n;
    },
    pc: () => 0x1000n,
  };
  state.emulator = emulator;
  state.wasmExports = { memory: {} };

  try {
    const result = await compileJitBlockEntry(0, 0x1000n, emulator);

    assert.equal(result.compiled, false);
    assert.equal(result.skipped, true);
    assert.match(result.error, /single-instruction/);
    assert.equal(instantiateCalls, 0);
    assert.equal(statePtrCalls, 0);
    assert.equal(state.jitBlocks.size, 0);
  } finally {
    WebAssembly.instantiate = originalInstantiate;
  }
});
