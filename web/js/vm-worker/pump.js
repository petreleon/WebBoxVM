import { tryRunOrCompileJitBlock } from "./jit-hot.js";
import { errorMessage } from "./errors.js";
import { maybePostMetrics, maybeRequestAutosave } from "./metrics-events.js";
import { drainNetworkTx } from "./network.js";
import {
  JIT_PROBE_STEP_SLICE,
  MAX_FRAME_BATCHES,
  MAX_FRAME_MS,
  NETWORK_IDLE_FAST_MS,
  NETWORK_STEP_SLICE,
  UART_FLUSH_BYTES,
  UART_FLUSH_INTERVAL_MS,
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
      drainUart(performance.now());
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
  if (networkNeedsResponsiveSlices()) {
    return Math.min(state.stepSlice, NETWORK_STEP_SLICE);
  }
  return state.stepSlice;
}

function networkNeedsResponsiveSlices() {
  if (state.networkStatus !== "connected") {
    return false;
  }
  if (state.emulator?.network_tx_pending?.() > 0) {
    return true;
  }
  return performance.now() - state.lastNetworkActivityAt < NETWORK_IDLE_FAST_MS;
}

function drainUart(now) {
  const uartLen = state.emulator.uart_output_len();
  const pendingBytes = uartLen - state.lastUart;
  if (!shouldFlushUart(pendingBytes, now, state.lastUartFlushAt)) {
    return;
  }
  const output = state.emulator.uart_output_since(state.lastUart);
  state.lastUart = uartLen;
  state.lastUartFlushAt = now;
  postMessage({ event: "uart", output });
}

export function shouldFlushUart(pendingBytes, now, lastFlushAt) {
  if (pendingBytes <= 0) {
    return false;
  }
  return pendingBytes >= UART_FLUSH_BYTES || now - lastFlushAt >= UART_FLUSH_INTERVAL_MS;
}
