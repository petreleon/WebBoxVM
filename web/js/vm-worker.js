import init, { Emulator } from "../pkg/emulator.js";
import { assertWasm64Supported } from "./wasm64.js";

const AUTOSAVE_INTERVAL_MS = 5000;
const DEFAULT_STEP_SLICE = 1_000_000;
const MAX_FRAME_MS = 24;
const MAX_FRAME_BATCHES = 8;
const METRICS_INTERVAL_MS = 100;
const JIT_ENABLED = true;
const JIT_HOT_THRESHOLD = 2;
const JIT_MAX_BLOCKS = 4096;

let emulator;
let wasmExports;
let wasmReady = false;
let running = false;
let pumpScheduled = false;
let stepSlice = DEFAULT_STEP_SLICE;
let jitBlocks = new Map();
let jitBlockHits = new Map();
let jitRejectedBlocks = new Set();
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
    case "compileJitBlock":
      return compileJitBlock(payload);
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
    case "runJitBlock":
      return runJitBlock(payload);
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
  wasmExports = await init({
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
  jitBlocks = new Map();
  jitBlockHits = new Map();
  jitRejectedBlocks = new Set();
  if (emulator) {
    emulator.free();
    emulator = undefined;
  }
}

async function compileJitBlock({ coreId = 0 } = {}) {
  requireEmulator();
  const owner = emulator;
  if (!wasmExports?.memory) {
    throw new Error("Wasm memory export is unavailable for JIT blocks");
  }

  const pc = owner.pc();
  const bytes = owner.jit_compile_current_block(coreId);
  if (!bytes.length) {
    return {
      compiled: false,
      error: owner.jit_last_error(),
      pc,
    };
  }

  const key = jitBlockKey(coreId, pc);
  const steps = owner.jit_last_block_steps();
  const startPc = owner.jit_last_block_start_pc();
  const startPa = owner.jit_last_block_start_pa();
  const exitPc = owner.jit_last_block_exit_pc();
  const rawHash = owner.jit_last_block_raw_hash();
  const { instance, module } = await WebAssembly.instantiate(bytes, {
    env: { memory: wasmExports.memory },
  });
  if (emulator !== owner) {
    return {
      compiled: false,
      error: "VM changed while compiling JIT block",
      pc,
    };
  }
  if (jitBlocks.size >= JIT_MAX_BLOCKS) {
    const evictedKey = jitBlocks.keys().next().value;
    jitBlocks.delete(evictedKey);
    jitBlockHits.delete(evictedKey);
    jitRejectedBlocks.delete(evictedKey);
  }
  jitBlocks.set(key, { exitPc, instance, module, rawHash, startPa, startPc, steps });
  jitRejectedBlocks.delete(key);

  return {
    compiled: true,
    bytes: bytes.length,
    exitPc,
    pc,
    rawHash,
    startPa,
    statePtr: owner.jit_state_ptr(),
    stateSize: owner.jit_state_size(),
    steps,
  };
}

async function runJitBlock({ coreId = 0 } = {}) {
  requireEmulator();
  const pc = emulator.pc();
  const key = jitBlockKey(coreId, pc);
  let entry = jitBlocks.get(key);

  if (!entry) {
    const compiled = await compileJitBlock({ coreId });
    if (!compiled.compiled) {
      return compiled;
    }
    entry = jitBlocks.get(key);
  }

  if (
    !emulator.jit_validate_block(
      coreId,
      entry.startPc,
      entry.startPa,
      entry.rawHash,
      entry.steps,
    )
  ) {
    jitBlocks.delete(key);
    return { compiled: true, committed: false, error: emulator.jit_last_error(), pc };
  }

  if (!emulator.jit_sync_state_from_core(coreId)) {
    return { compiled: true, committed: false, error: emulator.jit_last_error(), pc };
  }

  const exitPc = entry.instance.exports.run(emulator.jit_state_ptr());
  if (exitPc !== entry.exitPc) {
    jitBlocks.delete(key);
    return {
      compiled: true,
      committed: false,
      error: `JIT block returned 0x${exitPc.toString(16)} instead of 0x${entry.exitPc.toString(16)}`,
      exitPc,
      pc,
    };
  }
  const committed = emulator.jit_commit_state_to_core(coreId, entry.steps, entry.exitPc);
  if (!committed) {
    return {
      compiled: true,
      committed: false,
      error: emulator.jit_last_error(),
      exitPc,
      pc,
    };
  }

  postMetrics({ force: true });
  return {
    committed: true,
    exitPc,
    pc,
    steps: entry.steps,
  };
}

async function tryRunOrCompileJitBlock(coreId = 0) {
  if (!JIT_ENABLED || !emulator) {
    return false;
  }

  const pc = emulator.pc();
  const key = jitBlockKey(coreId, pc);
  const cached = jitBlocks.get(key);
  if (cached) {
    const result = runCachedJitBlock(coreId, key, cached);
    if (result.committed) {
      return true;
    }
    if (result.invalidated) {
      jitBlocks.delete(key);
    } else {
      return false;
    }
  }

  if (jitRejectedBlocks.has(key)) {
    return false;
  }

  const hits = (jitBlockHits.get(key) ?? 0) + 1;
  jitBlockHits.set(key, hits);
  if (hits < JIT_HOT_THRESHOLD) {
    return false;
  }

  const compiled = await compileJitBlock({ coreId });
  if (!running || !compiled.compiled) {
    if (!compiled.compiled) {
      jitRejectedBlocks.add(key);
    }
    return false;
  }

  const entry = jitBlocks.get(key);
  if (!entry) {
    return false;
  }
  return runCachedJitBlock(coreId, key, entry).committed;
}

function runCachedJitBlock(coreId, key, entry) {
  const pc = emulator.pc();
  if (
    !emulator.jit_validate_block(
      coreId,
      entry.startPc,
      entry.startPa,
      entry.rawHash,
      entry.steps,
    )
  ) {
    return { committed: false, error: emulator.jit_last_error(), invalidated: true, pc };
  }

  if (!emulator.jit_sync_state_from_core(coreId)) {
    return { committed: false, error: emulator.jit_last_error(), pc };
  }

  const exitPc = entry.instance.exports.run(emulator.jit_state_ptr());
  if (exitPc !== entry.exitPc) {
    jitBlocks.delete(key);
    return {
      committed: false,
      error: `JIT block returned 0x${exitPc.toString(16)} instead of 0x${entry.exitPc.toString(16)}`,
      invalidated: true,
      pc,
    };
  }

  const committed = emulator.jit_commit_state_to_core(coreId, entry.steps, entry.exitPc);
  return {
    committed,
    error: committed ? "" : emulator.jit_last_error(),
    exitPc,
    pc,
    steps: entry.steps,
  };
}

function schedulePump() {
  if (!running || pumpScheduled || !emulator) {
    return;
  }
  pumpScheduled = true;
  setTimeout(runPump, 0);
}

async function runPump() {
  pumpScheduled = false;
  if (!running || !emulator) {
    return;
  }

  const frameStart = performance.now();
  let batches = 0;

  try {
    do {
      const usedJit = await tryRunOrCompileJitBlock();
      if (!running) {
        return;
      }
      if (!usedJit) {
        emulator.run_kernel(stepSlice);
      }
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

function jitBlockKey(coreId, pc) {
  return `${coreId}:${pc.toString(16)}`;
}

function errorMessage(error) {
  return error?.message ?? String(error);
}
