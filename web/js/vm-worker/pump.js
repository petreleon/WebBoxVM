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
  UART_POLL_INTERVAL_MS,
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
  let now = frameStart;
  let batches = 0;

  try {
    do {
      let usedJit = tryRunOrCompileJitBlock();
      if (usedJit?.then) {
        usedJit = await usedJit;
      }
      if (!state.running) {
        return;
      }
      if (!usedJit) {
        now = performance.now();
        const emulator = state.emulator;
        if (!emulator) {
          return;
        }
        emulator.run_kernel(interpreterStepSlice(now, emulator));
      }
      now = performance.now();
      const sentNetworkFrames = drainNetworkTx(now);
      drainUart(now);
      batches += 1;
      if (sentNetworkFrames > 0) {
        break;
      }
    } while (state.running && shouldContinuePumpFrame(frameStart, now, batches));

    maybePostMetrics(now);
    maybeRequestAutosave(now);
    schedulePump();
  } catch (error) {
    state.running = false;
    postMessage({ error: errorMessage(error), event: "error" });
  }
}

export function interpreterStepSlice(now = performance.now(), emulator = state.emulator) {
  if (!state.jitEnabled) {
    return networkResponsiveStepSlice(now, emulator);
  }
  return Math.min(networkResponsiveStepSlice(now, emulator), JIT_PROBE_STEP_SLICE);
}

export function shouldContinuePumpFrame(frameStart, now, batches) {
  return now - frameStart < MAX_FRAME_MS && batches < MAX_FRAME_BATCHES;
}

function networkResponsiveStepSlice(now, emulator) {
  if (networkNeedsResponsiveSlices(now, emulator)) {
    return Math.min(state.stepSlice, NETWORK_STEP_SLICE);
  }
  return state.stepSlice;
}

function networkNeedsResponsiveSlices(now, emulator) {
  if (state.networkStatus !== "connected") {
    return false;
  }
  if (now - state.lastNetworkActivityAt < NETWORK_IDLE_FAST_MS) {
    return true;
  }
  return (emulator?.network_tx_pending?.() ?? 0) > 0;
}

export function drainUart(now) {
  const emulator = state.emulator;
  if (!emulator || !shouldPollUart(now, state.lastUartPollAt)) {
    return;
  }
  state.lastUartPollAt = now;
  const uartLen = emulator.uart_output_len();
  const pendingBytes = uartLen - state.lastUart;
  if (!shouldFlushUart(pendingBytes, now, state.lastUartFlushAt)) {
    return;
  }
  const output = emulator.uart_output_since(state.lastUart);
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

export function shouldPollUart(now, lastPollAt) {
  return lastPollAt === 0 || now - lastPollAt >= UART_POLL_INTERVAL_MS;
}
