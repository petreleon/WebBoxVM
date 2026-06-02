import init, { Emulator } from "../pkg/emulator.js";
import { assertWasm64Supported } from "./wasm64.js";

const AUTOSAVE_INTERVAL_MS = 5000;
const DEFAULT_STEP_SLICE = 1_000_000;
const MAX_FRAME_MS = 24;
const MAX_FRAME_BATCHES = 8;
const METRICS_INTERVAL_MS = 100;

let emulator;
let wasmReady = false;
let running = false;
let pumpScheduled = false;
let stepSlice = DEFAULT_STEP_SLICE;
let lastUart = 0;
let lastMetricsAt = 0;
let lastAutosaveAt = 0;
let lastAutosaveGeneration = 0n;

self.onmessage = (event) => {
  handleMessage(event.data).catch((error) => {
    postMessage({ error: errorMessage(error), event: "error" });
  });
};

async function handleMessage(message) {
  const { id, payload = {}, type } = message;

  try {
    const response = await handleRequest(type, payload);
    if (id === undefined) {
      return;
    }
    if (response?.transfer) {
      postMessage({ id, ok: true, value: response.value }, response.transfer);
    } else {
      postMessage({ id, ok: true, value: response });
    }
  } catch (error) {
    if (id === undefined) {
      postMessage({ error: errorMessage(error), event: "error" });
    } else {
      postMessage({ error: errorMessage(error), id, ok: false });
    }
  }
}

async function handleRequest(type, payload) {
  switch (type) {
    case "bootIsoWithDisk":
      return bootIsoWithDisk(payload);
    case "free":
      freeEmulator();
      return {};
    case "installDiskSnapshot":
      return installDiskSnapshot();
    case "pause":
      running = false;
      return {};
    case "restoreInstallDisk":
      return restoreInstallDisk(payload.snapshot);
    case "resume":
    case "start":
      setStepSlice(payload.stepSlice);
      running = true;
      schedulePump();
      return {};
    case "sendUartBytes":
      emulator?.send_uart_bytes(payload.input);
      return {};
    case "sendUartInput":
      emulator?.send_uart_input(payload.input);
      return {};
    case "setStepSlice":
      setStepSlice(payload.stepSlice);
      return {};
    case "stop":
      running = false;
      return {};
    default:
      throw new Error(`Unknown worker VM request: ${type}`);
  }
}

async function ensureWasm() {
  if (wasmReady) {
    return;
  }
  assertWasm64Supported();
  await init({
    module_or_path: new URL("../pkg/emulator_bg.wasm", import.meta.url),
  });
  wasmReady = true;
}

async function bootIsoWithDisk({ diskSizeBytes, isoImage, numCores }) {
  await ensureWasm();
  freeEmulator();
  emulator = new Emulator(numCores);
  lastUart = 0;
  lastMetricsAt = 0;
  lastAutosaveAt = performance.now();
  const result = emulator.boot_iso_with_disk(isoImage, numCores, diskSizeBytes);
  lastAutosaveGeneration = emulator.install_disk_generation();
  return { metrics: metrics(), result };
}

function restoreInstallDisk(snapshot) {
  requireEmulator();
  const result = emulator.restore_install_disk(snapshot);
  lastAutosaveGeneration = emulator.install_disk_generation();
  postMetrics({ force: true });
  return { metrics: metrics(), result };
}

function installDiskSnapshot() {
  requireEmulator();
  const snapshot = emulator.install_disk_snapshot();
  return {
    transfer: [snapshot.buffer],
    value: { metrics: metrics(), snapshot },
  };
}

function freeEmulator() {
  running = false;
  pumpScheduled = false;
  lastUart = 0;
  if (emulator) {
    emulator.free();
    emulator = undefined;
  }
}

function schedulePump() {
  if (!running || pumpScheduled || !emulator) {
    return;
  }
  pumpScheduled = true;
  setTimeout(runPump, 0);
}

function runPump() {
  pumpScheduled = false;
  if (!running || !emulator) {
    return;
  }

  const frameStart = performance.now();
  let batches = 0;

  try {
    do {
      emulator.run_kernel(stepSlice);
      drainUart();
      batches += 1;
    } while (running && performance.now() - frameStart < MAX_FRAME_MS && batches < MAX_FRAME_BATCHES);

    maybePostMetrics();
    maybeRequestAutosave();
    schedulePump();
  } catch (error) {
    running = false;
    postMessage({ error: errorMessage(error), event: "error" });
  }
}

function drainUart() {
  const output = emulator.uart_output_since(lastUart);
  if (!output) {
    return;
  }
  lastUart = emulator.uart_output_len();
  postMessage({ event: "uart", output });
}

function maybePostMetrics() {
  const now = performance.now();
  if (now - lastMetricsAt < METRICS_INTERVAL_MS) {
    return;
  }
  lastMetricsAt = now;
  postMetrics();
}

function postMetrics({ force = false } = {}) {
  if (!emulator) {
    return;
  }
  if (force) {
    lastMetricsAt = performance.now();
  }
  postMessage({ event: "metrics", metrics: metrics() });
}

function maybeRequestAutosave() {
  const generation = emulator.install_disk_generation();
  if (generation === lastAutosaveGeneration) {
    return;
  }

  const now = performance.now();
  if (now - lastAutosaveAt < AUTOSAVE_INTERVAL_MS) {
    return;
  }

  lastAutosaveAt = now;
  lastAutosaveGeneration = generation;
  postMessage({ event: "autosave" });
}

function metrics() {
  requireEmulator();
  return {
    allocatedPages: emulator.allocated_pages(),
    installDiskAllocatedBytes: emulator.install_disk_allocated_bytes(),
    installDiskGeneration: emulator.install_disk_generation(),
    installDiskSizeBytes: emulator.install_disk_size_bytes(),
    pc: emulator.pc(),
    totalSteps: emulator.total_steps(),
    uartOutputLen: emulator.uart_output_len(),
  };
}

function setStepSlice(value) {
  stepSlice = Math.max(1000, Math.min(1_000_000, Number(value) || DEFAULT_STEP_SLICE));
}

function requireEmulator() {
  if (!emulator) {
    throw new Error("Worker VM is not booted");
  }
}

function errorMessage(error) {
  return error?.message ?? String(error);
}
