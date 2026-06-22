import assert from "node:assert/strict";
import test from "node:test";
import {
  currentJitInstruction,
  jitStats,
  recordJitFallback,
  recordJitReject,
  recordJitSkip,
} from "./jit-stats.js";
import { resetJitState, state } from "./state.js";

test("jit reject logs include parsed current instruction snapshots", () => {
  resetTelemetry();
  state.emulator = {
    current_instruction(coreId) {
      assert.equal(coreId, 2);
      return JSON.stringify({
        el: 1,
        opcode: "SimdUminp",
        pc: "0xffff800080123450",
        raw: "0x6e20ac00",
      });
    },
  };

  recordJitReject("2:ffff800080123450", 0xffff800080123450n, "bad block", 2);

  assert.deepEqual(state.jitRejectLog, [
    {
      error: "bad block",
      instruction: {
        el: 1,
        opcode: "SimdUminp",
        pc: "0xffff800080123450",
        raw: "0x6e20ac00",
      },
      key: "2:ffff800080123450",
      pc: "ffff800080123450",
    },
  ]);
  resetTelemetry();
});

test("jit skip and fallback logs tolerate missing or text instruction snapshots", () => {
  resetTelemetry();
  state.emulator = {
    current_instruction() {
      return "not-json";
    },
  };

  recordJitSkip("0:1000", 0x1000n, "", 0);
  recordJitFallback("0:2000", 0x2000n, "commit failed", 0);

  assert.deepEqual(state.jitSkipLog[0], {
    error: "unknown JIT skip",
    instruction: { text: "not-json" },
    key: "0:1000",
    pc: "1000",
  });
  assert.equal(state.jitFallbackCount, 1);
  assert.deepEqual(state.jitLastFallback, {
    error: "commit failed",
    instruction: { text: "not-json" },
    key: "0:2000",
    pc: "2000",
  });
  resetTelemetry();
});

test("current jit instruction is omitted when the emulator has no snapshot", () => {
  resetTelemetry();

  assert.equal(currentJitInstruction(), undefined);
});

test("jit stats reuse already bounded telemetry logs", () => {
  resetTelemetry();
  for (let index = 0; index < 20; index += 1) {
    recordJitReject(`0:${index}`, BigInt(index), `reject ${index}`, 0);
    recordJitSkip(`1:${index}`, BigInt(index), `skip ${index}`, 1);
  }

  const stats = jitStats();

  assert.equal(state.jitRejectLog.length, 16);
  assert.equal(state.jitSkipLog.length, 16);
  assert.equal(stats.recentRejects, state.jitRejectLog);
  assert.equal(stats.recentSkips, state.jitSkipLog);
  assert.equal(stats.recentRejects[0].error, "reject 4");
  assert.equal(stats.recentSkips[0].error, "skip 4");
  resetTelemetry();
});

function resetTelemetry() {
  resetJitState();
  state.emulator = undefined;
}
