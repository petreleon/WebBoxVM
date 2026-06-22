import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { metrics } from "./lifecycle.js";
import { resetJitState, state } from "./state.js";

afterEach(() => {
  state.emulator = undefined;
  resetJitState();
});

test("routine metrics avoid current instruction decoding", () => {
  let decoded = false;
  state.emulator = metricsEmulator({
    current_instruction: () => {
      decoded = true;
      return "{}";
    },
  });

  const snapshot = metrics();

  assert.equal(decoded, false);
  assert.equal(snapshot.currentInstruction, undefined);
  assert.equal(snapshot.totalSteps, 9n);
});

test("routine metrics omit unchanged jit stats after a full snapshot", () => {
  state.emulator = metricsEmulator();

  const first = metrics();
  const second = metrics({ includeUnchangedJitStats: false });
  state.jitBlockHits.set("0:1000", 1);
  const third = metrics({ includeUnchangedJitStats: false });

  assert.equal(first.jitStats.hitSites, 0);
  assert.equal(second.jitStats, undefined);
  assert.equal(third.jitStats.hitSites, 1);
});

function metricsEmulator(overrides = {}) {
  return {
    allocated_pages: () => 1,
    install_disk_allocated_bytes: () => 2n,
    install_disk_generation: () => 3n,
    install_disk_size_bytes: () => 4n,
    network_rx_packets: () => 5n,
    network_tx_packets: () => 6n,
    network_tx_pending: () => 7,
    pc: () => 8n,
    total_steps: () => 9n,
    uart_output_len: () => 10,
    ...overrides,
  };
}
