import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { maybePostMetrics, maybeRequestAutosave } from "./metrics-events.js";
import { AUTOSAVE_INTERVAL_MS, AUTOSAVE_POLL_MS, METRICS_INTERVAL_MS, state } from "./state.js";

const previousPostMessage = globalThis.postMessage;
let messages = [];

beforeEach(() => {
  messages = [];
  globalThis.postMessage = (message) => messages.push(message);
});

afterEach(() => {
  globalThis.postMessage = previousPostMessage;
  state.emulator = undefined;
  state.lastAutosaveAt = 0;
  state.lastAutosaveGeneration = 0n;
  state.lastAutosavePollAt = 0;
  state.lastMetricsAt = 0;
});

function metricsEmulator() {
  return {
    allocated_pages: () => 1,
    install_disk_allocated_bytes: () => 2n,
    install_disk_generation: () => 3n,
    install_disk_size_bytes: () => 4n,
    network_rx_packets: () => 5n,
    network_tx_packets: () => 6n,
    network_tx_pending: () => 0,
    pc: () => 7n,
    total_steps: () => 8n,
    uart_output_len: () => 9,
  };
}

test("metrics posting can reuse a caller timestamp", () => {
  state.emulator = metricsEmulator();
  state.lastMetricsAt = 1000;

  maybePostMetrics(1000 + METRICS_INTERVAL_MS - 1);
  assert.deepEqual(messages, []);

  maybePostMetrics(1000 + METRICS_INTERVAL_MS);
  assert.equal(state.lastMetricsAt, 1000 + METRICS_INTERVAL_MS);
  assert.equal(messages.length, 1);
  assert.equal(messages[0].event, "metrics");
});

test("routine metrics use a low-overhead ui cadence", () => {
  assert.equal(METRICS_INTERVAL_MS, 250);
});

test("autosave skips disk generation polling inside poll window", () => {
  let generationPolls = 0;
  const now = 10_000;
  state.lastAutosaveAt = now - AUTOSAVE_INTERVAL_MS - 10;
  state.lastAutosaveGeneration = 0n;
  state.lastAutosavePollAt = now;
  state.emulator = {
    install_disk_generation: () => {
      generationPolls += 1;
      return 1n;
    },
  };

  maybeRequestAutosave(now);

  assert.equal(generationPolls, 0);
  assert.deepEqual(messages, []);
});

test("autosave polls generation and requests save after intervals", () => {
  let generationPolls = 0;
  const now = 10_000;
  state.lastAutosaveAt = now - AUTOSAVE_INTERVAL_MS - 10;
  state.lastAutosaveGeneration = 0n;
  state.lastAutosavePollAt = now - AUTOSAVE_POLL_MS - 10;
  state.emulator = {
    install_disk_generation: () => {
      generationPolls += 1;
      return 1n;
    },
  };

  maybeRequestAutosave(now);

  assert.equal(generationPolls, 1);
  assert.equal(state.lastAutosaveGeneration, 1n);
  assert.deepEqual(messages, [{ event: "autosave", installDiskGeneration: 1n }]);
});
