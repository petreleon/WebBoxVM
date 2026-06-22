import { Emulator, ensureWasm } from "./wasm.js";
import { changedJitStats, jitStats } from "./jit-stats.js";
import { startNetworkProxy, stopNetworkProxy } from "./network.js";
import { DEFAULT_STEP_SLICE, MAX_STEP_SLICE, state, resetJitState } from "./state.js";

export async function bootIsoWithDisk({ diskSizeBytes, isoImage, numCores }) {
  await ensureWasm();
  freeEmulator();
  state.emulator = new Emulator(numCores);
  state.lastUart = 0;
  state.lastUartFlushAt = 0;
  state.lastUartPollAt = 0;
  state.lastNetworkTxPollAt = 0;
  state.lastMetricsAt = 0;
  state.lastAutosaveAt = performance.now();
  state.lastAutosavePollAt = state.lastAutosaveAt;
  const result = state.emulator.boot_iso_with_disk(isoImage, numCores, diskSizeBytes);
  const installDiskGeneration = state.emulator.install_disk_generation();
  state.lastAutosaveGeneration = installDiskGeneration;
  startNetworkProxy();
  return { metrics: metrics({ installDiskGeneration }), result };
}

export async function bootInstalledDisk({ diskSnapshot, extraBootargs = "", numCores }) {
  await ensureWasm();
  freeEmulator();
  state.emulator = new Emulator(numCores);
  state.lastUart = 0;
  state.lastUartFlushAt = 0;
  state.lastUartPollAt = 0;
  state.lastNetworkTxPollAt = 0;
  state.lastMetricsAt = 0;
  state.lastAutosaveAt = performance.now();
  state.lastAutosavePollAt = state.lastAutosaveAt;
  const result = state.emulator.boot_installed_disk_with_extra_bootargs(
    diskSnapshot,
    numCores,
    extraBootargs,
  );
  const installDiskGeneration = state.emulator.install_disk_generation();
  state.lastAutosaveGeneration = installDiskGeneration;
  startNetworkProxy();
  return { metrics: metrics({ installDiskGeneration }), result };
}

export function restoreInstallDisk(snapshot) {
  requireEmulator();
  const result = state.emulator.restore_install_disk(snapshot);
  const installDiskGeneration = state.emulator.install_disk_generation();
  state.lastAutosaveGeneration = installDiskGeneration;
  state.lastMetricsAt = performance.now();
  const metricsSnapshot = metrics({ installDiskGeneration });
  postMessage({ event: "metrics", metrics: metricsSnapshot });
  return { metrics: metricsSnapshot, result };
}

export function installDiskSnapshot() {
  requireEmulator();
  const snapshot = state.emulator.install_disk_snapshot();
  return {
    transfer: [snapshot.buffer],
    value: { metrics: metrics(), snapshot },
  };
}

export function freeEmulator() {
  state.running = false;
  state.pumpScheduled = false;
  state.lastUart = 0;
  state.lastUartPollAt = 0;
  state.lastNetworkTxPollAt = 0;
  state.lastAutosavePollAt = 0;
  resetJitState();
  stopNetworkProxy();
  if (state.emulator) {
    state.emulator.free();
    state.emulator = undefined;
  }
}

export function metrics({ includeUnchangedJitStats = true, installDiskGeneration } = {}) {
  requireEmulator();
  const snapshot = {
    allocatedPages: state.emulator.allocated_pages(),
    installDiskAllocatedBytes: state.emulator.install_disk_allocated_bytes(),
    installDiskGeneration: installDiskGeneration ?? state.emulator.install_disk_generation(),
    installDiskSizeBytes: state.emulator.install_disk_size_bytes(),
    networkRxPackets: state.emulator.network_rx_packets(),
    networkStatus: state.networkStatus,
    networkTxPackets: state.emulator.network_tx_packets(),
    networkTxPending: state.emulator.network_tx_pending(),
    pc: state.emulator.pc(),
    totalSteps: state.emulator.total_steps(),
    uartOutputLen: state.emulator.uart_output_len(),
  };
  const stats = includeUnchangedJitStats ? jitStats() : changedJitStats();
  if (stats) {
    snapshot.jitStats = stats;
  }
  return snapshot;
}

export function setStepSlice(value) {
  state.stepSlice = Math.max(1000, Math.min(MAX_STEP_SLICE, Number(value) || DEFAULT_STEP_SLICE));
}

export function requireEmulator() {
  if (!state.emulator) {
    throw new Error("Worker VM is not booted");
  }
}
