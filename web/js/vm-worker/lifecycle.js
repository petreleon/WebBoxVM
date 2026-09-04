import { Emulator, ensureWasm } from "./wasm.js?v=20260904-virgl-mixed-depth-batch-r1";
import { BootPhaseTimer } from "./boot-timing.js?v=20260904-virgl-mixed-depth-batch-r1";
import { prepareExecutionMode, transitionToParallel } from "./execution-mode.js?v=20260904-virgl-mixed-depth-batch-r1";
import { bootPreparedInstalledDisk } from "./installed-boot.js?v=20260904-virgl-mixed-depth-batch-r1";
import { changedJitStats, jitStats } from "./jit-stats.js?v=20260904-virgl-mixed-depth-batch-r1";
import { startNetworkProxy, stopNetworkProxy } from "./network.js?v=20260904-virgl-mixed-depth-batch-r1";
import { DEFAULT_STEP_SLICE, MAX_STEP_SLICE, state, resetJitState } from "./state.js?v=20260904-virgl-mixed-depth-batch-r1";
import { resetUartInput } from "./uart-input.js?v=20260904-virgl-mixed-depth-batch-r1";
import { resetVmPollState } from "./poll-state.js?v=20260904-virgl-mixed-depth-batch-r1";

export { prepareExecutionMode, transitionToParallel };

export async function bootIsoWithDisk({ diskSizeBytes, isoImage, numCores }) {
  await ensureWasm();
  await freeEmulator();
  const emulator = new Emulator(numCores);
  state.emulator = emulator;
  resetVmPollState();
  const result = emulator.boot_iso_with_disk(isoImage, numCores, diskSizeBytes);
  await prepareExecutionMode(numCores);
  const installDiskGeneration = emulator.install_disk_generation();
  state.lastAutosaveGeneration = installDiskGeneration;
  startNetworkProxy();
  return { metrics: metrics({ emulator, installDiskGeneration }), result };
}

export async function bootInstalledDisk({
  diskSnapshot,
  extraBootargs = "",
  numCores,
  stagedSmpRequested = true,
}) {
  const timer = new BootPhaseTimer();
  await ensureWasm();
  timer.end("wasmLoadMs");
  await freeEmulator();
  const preparation = await prepareExecutionMode(numCores, { deferParallel: true });
  timer.end("workerPoolMs");
  let emulator;
  let result;
  try {
    ({ emulator, result } = bootPreparedInstalledDisk(
      Emulator,
      diskSnapshot,
      extraBootargs,
      preparation,
      stagedSmpRequested,
      (created) => {
        timer.end("emulatorCreateMs");
        state.emulator = created;
        resetVmPollState();
      },
    ));
  } catch (error) {
    await freeEmulator();
    throw error;
  }
  timer.end("firmwarePreparationMs");
  const bootSucceeded = !result.startsWith("ERR:");
  const stagedSmp = bootSucceeded && emulator.staged_smp_enabled();
  if (!bootSucceeded && state.vcpuPool) {
    await state.vcpuPool.stop();
    state.vcpuPool = undefined;
    state.parallelTransitionDeferred = false;
  } else if (preparation.parallelReady && !stagedSmp) {
    await transitionToParallel();
  }
  const installDiskGeneration = emulator.install_disk_generation();
  state.lastAutosaveGeneration = installDiskGeneration;
  if (bootSucceeded) {
    startNetworkProxy();
  }
  return {
    bootTimings: timer.finish(),
    metrics: metrics({ emulator, installDiskGeneration }),
    result,
    stagedSmp,
  };
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
  state.lastGpuScanoutPollAt = Number.NEGATIVE_INFINITY;
  state.lastGpu3dPollAt = Number.NEGATIVE_INFINITY;
  state.lastAutosavePollAt = 0;
  resetUartInput();
  resetJitState();
  stopNetworkProxy();
  if (state.vcpuPool) {
    await state.vcpuPool.stop();
    state.vcpuPool = undefined;
  }
  state.executionMode = "cooperative";
  state.numCores = 0;
  state.parallelTransitionDeferred = false;
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
    cooperativeIdleFastForwardCycles: emulator.cooperative_idle_fast_forward_cycles?.() ?? 0n,
    cooperativeWfeParks: emulator.cooperative_wfe_parks?.() ?? 0n,
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
