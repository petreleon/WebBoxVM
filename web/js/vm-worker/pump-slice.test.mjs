import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { interpreterStepSlice } from "./pump.js";
import {
  DEFAULT_JIT_ENABLED,
  DEFAULT_STEP_SLICE,
  JIT_PROBE_STEP_SLICE,
  NETWORK_STEP_SLICE,
  state,
} from "./state.js";

afterEach(() => {
  state.emulator = undefined;
  state.jitEnabled = DEFAULT_JIT_ENABLED;
  state.lastNetworkActivityAt = 0;
  state.networkStatus = "offline";
  state.stepSlice = DEFAULT_STEP_SLICE;
});

test("interpreter step slice can reuse checked emulator reference", () => {
  const previousDescriptor = Object.getOwnPropertyDescriptor(state, "emulator");
  let emulatorReads = 0;
  const emulator = { network_tx_pending: () => 1 };
  Object.defineProperty(state, "emulator", {
    configurable: true,
    get() {
      emulatorReads += 1;
      return undefined;
    },
  });
  state.jitEnabled = false;
  state.lastNetworkActivityAt = 0;
  state.networkStatus = "connected";
  state.stepSlice = 50_000_000;

  try {
    assert.equal(interpreterStepSlice(10_000, emulator), NETWORK_STEP_SLICE);
    assert.equal(emulatorReads, 0);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});

test("idle jit probe fallback uses faster default slice", () => {
  state.jitEnabled = true;
  state.networkStatus = "offline";

  assert.equal(JIT_PROBE_STEP_SLICE, DEFAULT_STEP_SLICE);
  assert.equal(interpreterStepSlice(10_000), DEFAULT_STEP_SLICE);
});
