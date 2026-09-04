import { VcpuPool } from "./vcpu-pool.js?v=20260904-virgl-depth-batch-r1";
import { resetJitState, state } from "./state.js?v=20260904-virgl-depth-batch-r1";

export async function prepareExecutionMode(
  numCores,
  { createPool = VcpuPool.create, deferParallel = false } = {},
) {
  state.executionMode = deferParallel ? "cooperative-jit" : "cooperative";
  state.numCores = numCores;
  state.parallelTransitionDeferred = false;
  if (numCores <= 1) {
    return fallbackPreparation(numCores);
  }
  if (!state.threadedWasm) {
    const reason = state.wasmFallbackReason ?? "Threaded Wasm is unavailable";
    return fallbackPreparation(numCores, reason);
  }
  try {
    state.vcpuPool = await createPool(numCores, state.threadedWasm);
    if (deferParallel) {
      state.parallelTransitionDeferred = true;
      return { bootCores: numCores, parallelReady: true };
    }
    state.executionMode = "parallel-wasm";
    state.jitEnabled = false;
    return { bootCores: numCores, parallelReady: true };
  } catch (error) {
    state.wasmFallbackReason = error?.message ?? String(error);
    state.vcpuPool = undefined;
    return fallbackPreparation(numCores, state.wasmFallbackReason);
  }
}

export async function transitionToParallel() {
  if (state.executionMode === "parallel-wasm") {
    return unchanged();
  }
  if (!state.parallelTransitionDeferred || state.numCores <= 1) {
    return unchanged();
  }

  state.parallelTransitionDeferred = false;
  const pool = state.vcpuPool;
  if (!pool?.isReady(state.numCores)) {
    await pool?.stop();
    state.vcpuPool = undefined;
    return fallback(state.wasmFallbackReason ?? "Parallel workers are unavailable");
  }

  const wasRunning = state.running;
  state.running = false;
  try {
    resetJitState();
    state.executionMode = "parallel-wasm";
    state.jitEnabled = false;
    return { executionMode: state.executionMode, transitioned: true };
  } finally {
    state.running = wasRunning;
  }
}

function unchanged() {
  return { executionMode: state.executionMode, transitioned: false };
}

function fallback(reason) {
  return { executionMode: state.executionMode, reason, transitioned: false };
}

function fallbackPreparation(numCores, reason = undefined) {
  state.numCores = numCores;
  return { bootCores: numCores, parallelReady: false, reason };
}
