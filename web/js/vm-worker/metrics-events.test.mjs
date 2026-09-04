import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { maybePostMetrics, maybeRequestAutosave, postMetrics } from "./metrics-events.js?v=20260904-virgl-depth-r1";
import { AUTOSAVE_INTERVAL_MS, AUTOSAVE_POLL_MS, METRICS_INTERVAL_MS, state } from "./state.js?v=20260904-virgl-depth-r1";

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
  const previousDescriptor = Object.getOwnPropertyDescriptor(state, "emulator");
  const emulator = metricsEmulator();
  let emulatorReads = 0;
  state.lastMetricsAt = 1000;
  Object.defineProperty(state, "emulator", {
    configurable: true,
    get() {
      emulatorReads += 1;
      return undefined;
    },
  });

  try {
    maybePostMetrics(1000 + METRICS_INTERVAL_MS - 1, emulator);
    assert.deepEqual(messages, []);

    maybePostMetrics(1000 + METRICS_INTERVAL_MS, emulator);
    assert.equal(state.lastMetricsAt, 1000 + METRICS_INTERVAL_MS);
    assert.equal(messages.length, 1);
    assert.equal(messages[0].event, "metrics");
    assert.equal(emulatorReads, 0);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});

test("worker autosave requests use a coarse disk-snapshot cadence", () => {
  assert.equal(AUTOSAVE_INTERVAL_MS, 60_000);
});

test("routine metrics use a low-overhead ui cadence", () => {
  assert.equal(METRICS_INTERVAL_MS, 250);
});

test("routine metrics omit unchanged jit stats payloads", () => {
  state.emulator = metricsEmulator();

  postMetrics({ force: true, now: 1000 });
  postMetrics();

  assert.equal(messages[0].metrics.jitStats.enabled, false);
  assert.equal(messages[1].metrics.jitStats, undefined);
});

test("post metrics samples through one checked emulator reference", () => {
  const previousDescriptor = Object.getOwnPropertyDescriptor(state, "emulator");
  const emulator = metricsEmulator();
  let emulatorReads = 0;
  Object.defineProperty(state, "emulator", {
    configurable: true,
    get() {
      emulatorReads += 1;
      return emulator;
    },
  });

  try {
    postMetrics();

    assert.equal(messages.length, 1);
    assert.equal(messages[0].metrics.totalSteps, 8n);
    assert.equal(emulatorReads, 1);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
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
  const previousDescriptor = Object.getOwnPropertyDescriptor(state, "emulator");
  let emulatorReads = 0;
  let generationPolls = 0;
  const now = 10_000;
  state.lastAutosaveAt = now - AUTOSAVE_INTERVAL_MS - 10;
  state.lastAutosaveGeneration = 0n;
  state.lastAutosavePollAt = now - AUTOSAVE_POLL_MS - 10;
  const emulator = {
    install_disk_generation: () => {
      generationPolls += 1;
      return 1n;
    },
  };
  Object.defineProperty(state, "emulator", {
    configurable: true,
    get() {
      emulatorReads += 1;
      return undefined;
    },
  });

  try {
    maybeRequestAutosave(now, emulator);

    assert.equal(generationPolls, 1);
    assert.equal(state.lastAutosaveGeneration, 1n);
    assert.deepEqual(messages, [{ event: "autosave", installDiskGeneration: 1n }]);
    assert.equal(emulatorReads, 0);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});
