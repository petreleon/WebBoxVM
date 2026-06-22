import { tryRunOrCompileJitBlock } from "./jit-hot.js";
import { errorMessage } from "./errors.js";
import { maybePostMetrics, maybeRequestAutosave } from "./metrics-events.js";
import { drainNetworkTx } from "./network.js";
import {
  JIT_PROBE_STEP_SLICE,
  MAX_FRAME_BATCHES,
  MAX_FRAME_MS,
  NETWORK_STEP_SLICE,
  state,
} from "./state.js";

export function schedulePump() {
  if (!state.running || state.pumpScheduled || !state.emulator) {
    return;
  }
  state.pumpScheduled = true;
  setTimeout(runPump, 0);
}

async function runPump() {
  state.pumpScheduled = false;
  if (!state.running || !state.emulator) {
    return;
  }

  const frameStart = performance.now();
  let batches = 0;

  try {
    do {
      const usedJit = await tryRunOrCompileJitBlock();
      if (!state.running) {
        return;
      }
      if (!usedJit) {
        state.emulator.run_kernel(interpreterStepSlice());
      }
      const sentNetworkFrames = drainNetworkTx();
      drainUart();
      batches += 1;
      if (sentNetworkFrames > 0) {
        break;
      }
    } while (
      state.running &&
      performance.now() - frameStart < MAX_FRAME_MS &&
      batches < MAX_FRAME_BATCHES
    );

    maybePostMetrics();
    maybeRequestAutosave();
    schedulePump();
  } catch (error) {
    state.running = false;
    postMessage({ error: errorMessage(error), event: "error" });
  }
}

export function interpreterStepSlice() {
  if (!state.jitEnabled) {
    return networkResponsiveStepSlice();
  }
  return Math.min(networkResponsiveStepSlice(), JIT_PROBE_STEP_SLICE);
}

function networkResponsiveStepSlice() {
  if (state.networkStatus === "connected") {
    return Math.min(state.stepSlice, NETWORK_STEP_SLICE);
  }
  return state.stepSlice;
}

function drainUart() {
  const output = state.emulator.uart_output_since(state.lastUart);
  if (!output) {
    return;
  }
  state.lastUart = state.emulator.uart_output_len();
  postMessage({ event: "uart", output });
}
