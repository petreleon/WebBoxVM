import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { interpreterStepSlice } from "./pump.js?v=20260904-virgl-depth-compare-r1";
import {
  COOPERATIVE_STEP_SLICE,
  DEFAULT_JIT_ENABLED,
  DEFAULT_STEP_SLICE,
  JIT_PROBE_STEP_SLICE,
  NETWORK_STEP_SLICE,
  NETWORK_TX_POLL_INTERVAL_MS,
  state,
} from "./state.js?v=20260904-virgl-depth-compare-r1";

afterEach(() => {
  state.emulator = undefined;
  state.jitEnabled = DEFAULT_JIT_ENABLED;
  state.lastNetworkActivityAt = 0;
  state.lastNetworkTxPollAt = 0;
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
    assert.equal(interpreterStepSlice(10_000, emulator), COOPERATIVE_STEP_SLICE);
    assert.equal(emulatorReads, 0);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});

test("idle jit probe fallback uses the normal interpreter slice", () => {
  state.jitEnabled = true;
  state.networkStatus = "offline";

  assert.equal(JIT_PROBE_STEP_SLICE, COOPERATIVE_STEP_SLICE);
  assert.equal(interpreterStepSlice(10_000), JIT_PROBE_STEP_SLICE);
});

test("jit probe fallback still yields to pending network transmit", () => {
  state.jitEnabled = true;
  state.networkStatus = "connected";
  state.lastNetworkActivityAt = 0;
  state.lastNetworkTxPollAt = 10_000;
  state.emulator = { network_tx_pending: () => 1 };

  assert.equal(
    interpreterStepSlice(10_000 + NETWORK_TX_POLL_INTERVAL_MS),
    COOPERATIVE_STEP_SLICE,
  );
});

test("idle network pending checks respect tx poll cadence", () => {
  let pendingPolls = 0;
  state.jitEnabled = false;
  state.networkStatus = "connected";
  state.lastNetworkActivityAt = 0;
  state.lastNetworkTxPollAt = 10_000;
  state.stepSlice = 50_000_000;
  state.emulator = {
    network_tx_pending: () => {
      pendingPolls += 1;
      return 1;
    },
  };

  assert.equal(
    interpreterStepSlice(10_000 + NETWORK_TX_POLL_INTERVAL_MS - 1),
    COOPERATIVE_STEP_SLICE,
  );
  assert.equal(pendingPolls, 0);
  assert.equal(
    interpreterStepSlice(10_000 + NETWORK_TX_POLL_INTERVAL_MS),
    COOPERATIVE_STEP_SLICE,
  );
  assert.equal(pendingPolls, 1);
});
