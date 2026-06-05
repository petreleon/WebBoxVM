import { Emulator, ensureWasm } from "./wasm.js?v=20260606-jitprobe";
import { jitStats } from "./jit-stats.js?v=20260606-jitprobe";
import { DEFAULT_STEP_SLICE, MAX_STEP_SLICE, state, resetJitState } from "./state.js?v=20260606-jitprobe";

export async function bootIsoWithDisk({ diskSizeBytes, isoImage, numCores }) {
  await ensureWasm();
  freeEmulator();
  state.emulator = new Emulator(numCores);
  state.lastUart = 0;
  state.lastMetricsAt = 0;
  state.lastAutosaveAt = performance.now();
  const result = state.emulator.boot_iso_with_disk(isoImage, numCores, diskSizeBytes);
  state.lastAutosaveGeneration = state.emulator.install_disk_generation();
  return { metrics: metrics(), result };
}

export function restoreInstallDisk(snapshot) {
  requireEmulator();
  const result = state.emulator.restore_install_disk(snapshot);
  state.lastAutosaveGeneration = state.emulator.install_disk_generation();
  state.lastMetricsAt = performance.now();
  postMessage({ event: "metrics", metrics: metrics() });
  return { metrics: metrics(), result };
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
  resetJitState();
  if (state.emulator) {
    state.emulator.free();
    state.emulator = undefined;
  }
}

export function metrics() {
  requireEmulator();
  return {
    allocatedPages: state.emulator.allocated_pages(),
    installDiskAllocatedBytes: state.emulator.install_disk_allocated_bytes(),
    installDiskGeneration: state.emulator.install_disk_generation(),
    installDiskSizeBytes: state.emulator.install_disk_size_bytes(),
    jitStats: jitStats(),
    pc: state.emulator.pc(),
    totalSteps: state.emulator.total_steps(),
    uartOutputLen: state.emulator.uart_output_len(),
  };
}

export function setStepSlice(value) {
  state.stepSlice = Math.max(1000, Math.min(MAX_STEP_SLICE, Number(value) || DEFAULT_STEP_SLICE));
}

export function requireEmulator() {
  if (!state.emulator) {
    throw new Error("Worker VM is not booted");
  }
}
