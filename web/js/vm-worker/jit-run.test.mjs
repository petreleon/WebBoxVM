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
    steps: 4,
  };
}

test("commit preflight falls back without validating or running cached jit", () => {
  let ran = false;
  let synced = false;
  let validated = false;
  state.emulator = {
    jit_can_commit_block_now: () => false,
    jit_last_error: () => "JIT block crosses timer deadline",
    jit_sync_state_from_core: () => {
      synced = true;
      return true;
    },
    jit_validate_block: () => {
      validated = true;
      return true;
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
  assert.equal(validated, false);
  assert.equal(ran, false);
  assert.equal(synced, false);
});

test("cached jit validation receives code page generations", () => {
  let validateArgs;
  state.emulator = {
    jit_can_commit_block_now: () => true,
    jit_commit_state_to_core: () => true,
    jit_helper_failed: () => false,
    jit_last_error: () => "",
    jit_state_ptr: () => 0n,
    jit_sync_state_from_core: () => true,
    jit_validate_block: (...args) => {
      validateArgs = args;
      return true;
    },
    pc: () => 0x1000n,
  };

  const result = runCachedJitBlock(0, "0:1000", cachedEntry());

  assert.equal(result.committed, true);
  assert.deepEqual(validateArgs, [0, 0x1000n, 0x2000n, 1n, 2n, 3n, 4]);
});
