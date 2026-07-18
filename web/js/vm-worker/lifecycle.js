import { Emulator, ensureWasm } from "./wasm.js";
import { VcpuPool } from "./vcpu-pool.js";
import { changedJitStats, jitStats } from "./jit-stats.js";
import { startNetworkProxy, stopNetworkProxy } from "./network.js";
import { DEFAULT_STEP_SLICE, MAX_STEP_SLICE, state, resetJitState } from "./state.js";

export async function bootIsoWithDisk({ diskSizeBytes, isoImage, numCores }) {
  await ensureWasm();
  await freeEmulator();
  const emulator = new Emulator(numCores);
  state.emulator = emulator;
  state.lastUart = 0;
  state.lastUartFlushAt = 0;
  state.lastUartPollAt = 0;
  state.lastNetworkTxPollAt = 0;
  state.lastMetricsAt = 0;
  state.lastAutosaveAt = performance.now();
  state.lastAutosavePollAt = state.lastAutosaveAt;
  const result = emulator.boot_iso_with_disk(isoImage, numCores, diskSizeBytes);
  await prepareExecutionMode(numCores);
  const installDiskGeneration = emulator.install_disk_generation();
  state.lastAutosaveGeneration = installDiskGeneration;
  startNetworkProxy();
  return { metrics: metrics({ emulator, installDiskGeneration }), result };
}

export async function bootInstalledDisk({ diskSnapshot, extraBootargs = "", numCores }) {
  await ensureWasm();
  await freeEmulator();
  const emulator = new Emulator(numCores);
  state.emulator = emulator;
  state.lastUart = 0;
  state.lastUartFlushAt = 0;
  state.lastUartPollAt = 0;
  state.lastNetworkTxPollAt = 0;
  state.lastMetricsAt = 0;
  state.lastAutosaveAt = performance.now();
  state.lastAutosavePollAt = state.lastAutosaveAt;
  const result = emulator.boot_installed_disk_with_extra_bootargs(
    diskSnapshot,
    numCores,
    extraBootargs,
  );
  await prepareExecutionMode(numCores);
  const installDiskGeneration = emulator.install_disk_generation();
  state.lastAutosaveGeneration = installDiskGeneration;
  startNetworkProxy();
  return { metrics: metrics({ emulator, installDiskGeneration }), result };
}

export function restoreInstallDisk(snapshot) {
  const emulator = requireEmulator();
  const result = emulator.restore_install_disk(snapshot);
  const installDiskGeneration = emulator.install_disk_generation();
  state.lastAutosaveGeneration = installDiskGeneration;
  state.lastMetricsAt = performance.now();
  const metricsSnapshot = metrics({ emulator, installDiskGeneration });
  postMessage({ event: "metrics", metrics: metricsSnapshot });
  return { metrics: metricsSnapshot, result };
}

export function installDiskSnapshot() {
  const emulator = requireEmulator();
  const snapshot = emulator.install_disk_snapshot();
  return {
    transfer: [snapshot.buffer],
    value: { metrics: metrics({ emulator }), snapshot },
  };
}

export async function freeEmulator() {
  state.running = false;
  state.pumpScheduled = false;
  state.lastUart = 0;
  state.lastUartPollAt = 0;
  state.lastNetworkTxPollAt = 0;
  state.lastAutosavePollAt = 0;
  resetJitState();
  stopNetworkProxy();
  if (state.vcpuPool) {
    await state.vcpuPool.stop();
    state.vcpuPool = undefined;
  }
  state.executionMode = "cooperative";
  if (state.emulator) {
    state.emulator.free();
    state.emulator = undefined;
  }
}

export function metrics({
  emulator = requireEmulator(),
  includeUnchangedJitStats = true,
  installDiskGeneration,
} = {}) {
  const snapshot = {
    allocatedPages: emulator.allocated_pages(),
    installDiskAllocatedBytes: emulator.install_disk_allocated_bytes(),
    installDiskGeneration: installDiskGeneration ?? emulator.install_disk_generation(),
    installDiskSizeBytes: emulator.install_disk_size_bytes(),
    networkRxPackets: emulator.network_rx_packets(),
    networkStatus: state.networkStatus,
    networkTxPackets: emulator.network_tx_packets(),
    networkTxPending: emulator.network_tx_pending(),
    executionMode: state.executionMode,
    parallelMaxLocalInFlight: emulator.parallel_max_local_in_flight?.() ?? 1,
    parallelWorkerThreads: emulator.parallel_worker_threads?.() ?? 1,
    pc: emulator.pc(),
    totalSteps: emulator.total_steps(),
    uartOutputLen: emulator.uart_output_len(),
  };
  const stats = includeUnchangedJitStats ? jitStats() : changedJitStats();
  if (stats) {
    snapshot.jitStats = stats;
  }
  return snapshot;
}

async function prepareExecutionMode(numCores) {
  state.executionMode = "cooperative";
  if (!state.threadedWasm || numCores <= 1) {
    return;
  }
  try {
    state.vcpuPool = await VcpuPool.create(numCores, state.threadedWasm);
    state.executionMode = "parallel-wasm";
    state.jitEnabled = false;
  } catch (error) {
    state.wasmFallbackReason = error?.message ?? String(error);
    state.vcpuPool = undefined;
  }
}

export function setStepSlice(value) {
  state.stepSlice = Math.max(1000, Math.min(MAX_STEP_SLICE, Number(value) || DEFAULT_STEP_SLICE));
}

export function requireEmulator() {
  const emulator = state.emulator;
  if (!emulator) {
    throw new Error("Worker VM is not booted");
  }
  return emulator;
}
