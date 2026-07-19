import { tryRunOrCompileNextJitBlock } from "./jit-hot.js?v=20260720-firmware-fast-boot-r2";
import { withEmulatorAccess } from "./access.js?v=20260720-firmware-fast-boot-r2";
import { errorMessage } from "./errors.js?v=20260720-firmware-fast-boot-r2";
import { maybePostMetrics, maybeRequestAutosave } from "./metrics-events.js?v=20260720-firmware-fast-boot-r2";
import { drainNetworkTx } from "./network.js?v=20260720-firmware-fast-boot-r2";
import {
  JIT_PROBE_STEP_SLICE,
  MAX_FRAME_BATCHES,
  MAX_FRAME_MS,
  NETWORK_IDLE_FAST_MS,
  NETWORK_STEP_SLICE,
  NETWORK_TX_POLL_INTERVAL_MS,
  UART_FLUSH_BYTES,
  UART_FLUSH_INTERVAL_MS,
  UART_POLL_INTERVAL_MS,
  state,
} from "./state.js?v=20260720-firmware-fast-boot-r2";

const schedulePumpTask = createPumpTaskScheduler();

export function schedulePump() {
  if (!state.running || state.pumpScheduled || !state.emulator) {
    return;
  }
  state.pumpScheduled = true;
  schedulePumpTask(() => {
    void withEmulatorAccess(runPump);
  });
}

export function createPumpTaskScheduler({
  MessageChannelCtor = globalThis.MessageChannel,
  timeout = setTimeout,
} = {}) {
  if (typeof MessageChannelCtor !== "function") {
    return (callback) => timeout(callback, 0);
  }
  const queue = [];
  const channel = new MessageChannelCtor();
  const runNext = () => queue.shift()?.();
  if (typeof channel.port1.addEventListener === "function") {
    channel.port1.addEventListener("message", runNext);
    channel.port1.start?.();
  } else {
    channel.port1.onmessage = runNext;
  }
  channel.port1.unref?.();
  channel.port2.unref?.();
  return (callback) => {
    queue.push(callback);
    channel.port2.postMessage(undefined);
  };
}

async function runPump() {
  state.pumpScheduled = false;
  if (!state.running || !state.emulator) {
    return;
  }

  const frameStart = performance.now();
  let now = frameStart;
  let batches = 0;
  let emulator;

  try {
    do {
      emulator = state.emulator;
      if (!emulator) {
        return;
      }
      let usedJit =
        state.executionMode === "parallel-wasm"
          ? false
          : tryRunOrCompileNextJitBlock(emulator);
      if (usedJit !== true && usedJit !== false) {
        usedJit = await usedJit;
        if (!state.running) {
          return;
        }
        emulator = state.emulator;
        if (!emulator) {
          return;
        }
      }
      if (!state.running) {
        return;
      }
      if (!usedJit) {
        const stepSlice = interpreterStepSlice(now, emulator);
        if (state.executionMode === "parallel-wasm") {
          await state.vcpuPool.runRound(emulator, stepSlice);
        } else {
          emulator.run_kernel(stepSlice);
        }
      }
      now = performance.now();
      const sentNetworkFrames =
        state.networkStatus === "connected" ? drainNetworkTx(now, emulator) : 0;
      drainUart(now, emulator);
      batches += 1;
      if (sentNetworkFrames > 0) {
        break;
      }
    } while (state.running && shouldContinuePumpFrame(frameStart, now, batches));

    maybePostMetrics(now, emulator);
    maybeRequestAutosave(now, emulator);
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
  if (
    state.lastNetworkTxPollAt !== 0 &&
    now - state.lastNetworkTxPollAt < NETWORK_TX_POLL_INTERVAL_MS
  ) {
    return false;
  }
  return (emulator?.network_tx_pending?.() ?? 0) > 0;
}

export function drainUart(now, emulator = state.emulator) {
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
