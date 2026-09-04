import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { metrics, restoreInstallDisk } from "./lifecycle.js?v=20260904-virgl-mixed-depth-batch-r1";
import { resetJitState, state } from "./state.js?v=20260904-virgl-mixed-depth-batch-r1";

const previousPostMessage = globalThis.postMessage;

afterEach(() => {
  globalThis.postMessage = previousPostMessage;
  state.emulator = undefined;
  state.lastAutosaveGeneration = 0n;
  state.lastMetricsAt = 0;
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

test("metrics can reuse known install disk generation", () => {
  let generationPolls = 0;
  state.emulator = metricsEmulator({
    install_disk_generation: () => {
      generationPolls += 1;
      return 3n;
    },
  });

  const snapshot = metrics({ installDiskGeneration: 11n });

  assert.equal(snapshot.installDiskGeneration, 11n);
  assert.equal(generationPolls, 0);
});

test("metrics samples through one checked emulator reference", () => {
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
    const snapshot = metrics();

    assert.equal(snapshot.totalSteps, 9n);
    assert.equal(emulatorReads, 1);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});

test("restore install disk reuses one metrics snapshot", () => {
  const messages = [];
  const snapshot = new Uint8Array([1, 2]);
  let generationPolls = 0;
  let restoredSnapshot;
  globalThis.postMessage = (message) => messages.push(message);
  state.emulator = metricsEmulator({
    install_disk_generation: () => {
      generationPolls += 1;
      return 12n;
    },
    restore_install_disk: (value) => {
      restoredSnapshot = value;
      return "restored";
    },
  });

  const result = restoreInstallDisk(snapshot);

  assert.equal(result.result, "restored");
  assert.equal(restoredSnapshot, snapshot);
  assert.equal(result.metrics.installDiskGeneration, 12n);
  assert.equal(messages.length, 1);
  assert.equal(messages[0].metrics, result.metrics);
  assert.equal(generationPolls, 1);
  assert.equal(state.lastAutosaveGeneration, 12n);
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
