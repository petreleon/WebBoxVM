import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { compileJitBlock, compiledJitBlockSkipReason } from "./jit-compile.js";
import { state } from "./state.js";

afterEach(() => {
  state.emulator = undefined;
  state.wasmExports = undefined;
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

test("EL0 guest-memory helper blocks are not compile-time skipped", () => {
  assert.equal(
    compiledJitBlockSkipReason({ blockEl: 0, usesGuestHelpers: true }),
    undefined,
  );
});
