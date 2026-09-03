import { tryRunOrCompileNextJitBlock } from "./jit-hot.js?v=20260903-virgl-capset1-r1";
import { withEmulatorAccess } from "./access.js?v=20260903-virgl-capset1-r1";
import { errorMessage } from "./errors.js?v=20260903-virgl-capset1-r1";
import { maybePostGpu3d } from "./gpu-3d.js?v=20260903-virgl-capset1-r1";
import { maybePostMetrics, maybeRequestAutosave } from "./metrics-events.js?v=20260903-virgl-capset1-r1";
import { maybePostGpuScanout } from "./gpu-scanout.js?v=20260903-virgl-capset1-r1";
import { drainNetworkTx } from "./network.js?v=20260903-virgl-capset1-r1";
import {
  JIT_PROBE_STEP_SLICE,
  MAX_FRAME_BATCHES,
  MAX_FRAME_MS,
  NETWORK_IDLE_FAST_MS,
  NETWORK_STEP_SLICE,
  NETWORK_TX_POLL_INTERVAL_MS,
  state,
} from "./state.js?v=20260903-virgl-capset1-r1";
import {
  isInputResponsive,
  markUartGuestServiced,
  responsiveStepSlice,
} from "./uart-input.js?v=20260903-virgl-capset1-r1";
import { drainUart } from "./uart-output.js?v=20260903-virgl-capset1-r1";

export { drainUart, shouldFlushUart, shouldPollUart } from "./uart-output.js?v=20260903-virgl-capset1-r1";

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
      if (state.urgentUartWaiters > 0) {
        schedulePump();
        return;
      }
      markUartGuestServiced(now);
      const sentNetworkFrames =
        state.networkStatus === "connected" ? drainNetworkTx(now, emulator) : 0;
      const sentUart = drainUart(now, emulator);
      const sentGpuFrame = maybePostGpuScanout(now, emulator);
      const sentGpu3dFrame = maybePostGpu3d(now, emulator);
      batches += 1;
      if (sentNetworkFrames > 0 || sentUart || sentGpuFrame || sentGpu3dFrame) {
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

export function shouldContinuePumpFrame(
  frameStart,
  now,
  batches,
  responsive = isInputResponsive(now),
  executionMode = state.executionMode,
) {
  const canReceiveMessagesDuringBatch = executionMode === "parallel-wasm";
  return (
    (!responsive || canReceiveMessagesDuringBatch) &&
    now - frameStart < MAX_FRAME_MS &&
    batches < MAX_FRAME_BATCHES
  );
}

function networkResponsiveStepSlice(now, emulator) {
  const baseSlice = responsiveStepSlice(now);
  if (networkNeedsResponsiveSlices(now, emulator)) {
    return Math.min(baseSlice, NETWORK_STEP_SLICE);
  }
  return baseSlice;
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
