import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { runCachedJitBlock } from "./jit-run.js";
import { resetJitState, state } from "./state.js";

afterEach(() => {
  resetJitState();
  state.emulator = undefined;
});

test("commit preflight falls back without running cached jit", () => {
  let ran = false;
  let synced = false;
  state.emulator = {
    jit_can_commit_block_now: () => false,
    jit_last_error: () => "JIT block crosses timer deadline",
    jit_sync_state_from_core: () => {
      synced = true;
      return true;
    },
    jit_validate_block: () => true,
    pc: () => 0x1000n,
  };

  const result = runCachedJitBlock(0, "0:1000", {
    alternateExitPc: 0n,
    dynamicExit: false,
    exitPc: 0x1010n,
    instance: {
      exports: {
        run: () => {
          ran = true;
          return 0x1010n;
        },
      },
    },
    rawHash: 1n,
    startPa: 0x2000n,
    startPc: 0x1000n,
    steps: 4,
  });

  assert.equal(result.committed, false);
  assert.equal(result.error, "JIT block crosses timer deadline");
  assert.equal(result.invalidated, undefined);
  assert.equal(ran, false);
  assert.equal(synced, false);
});
