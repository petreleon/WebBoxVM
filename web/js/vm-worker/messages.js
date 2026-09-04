import {
  bootInstalledDisk,
  bootIsoWithDisk,
  freeEmulator,
  installDiskSnapshot,
  metrics,
  requireEmulator,
  restoreInstallDisk,
  setStepSlice,
  transitionToParallel,
} from "./lifecycle.js?v=20260904-virgl-gpu-readback-r1";
import { withEmulatorAccess } from "./access.js?v=20260904-virgl-gpu-readback-r1";
import { compileJitBlock } from "./jit-compile.js?v=20260904-virgl-gpu-readback-r1";
import { errorMessage } from "./errors.js?v=20260904-virgl-gpu-readback-r1";
import { runJitBlock } from "./jit-run.js?v=20260904-virgl-gpu-readback-r1";
import { schedulePump } from "./pump.js?v=20260904-virgl-gpu-readback-r1";
import { resetJitState, state } from "./state.js?v=20260904-virgl-gpu-readback-r1";
import {
  beginUrgentUartMessage,
  finishUrgentUartMessage,
  injectUartMessage,
} from "./uart-input.js?v=20260904-virgl-gpu-readback-r1";

export async function handleMessage(message) {
  const { id, payload = {}, type } = message;
  const urgentUart = beginUrgentUartMessage(message);

  try {
    const response = await withEmulatorAccess(async () => {
      try {
        return await handleRequest(type, payload);
      } finally {
        finishUrgentUartMessage(urgentUart);
      }
    });
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
    case "bootInstalledDisk":
      return bootInstalledDisk(payload);
    case "compileJitBlock":
      return compileJitBlock(payload);
    case "currentInstruction":
      return state.emulator?.current_instruction(payload.coreId);
    case "debugReadPaU64":
      return state.emulator?.debug_read_pa_u64(BigInt(payload.pa));
    case "debugReadVaU64":
      return state.emulator?.debug_read_va_u64(BigInt(payload.va), payload.coreId);
    case "debugTranslateVa":
      return state.emulator?.debug_translate_va(BigInt(payload.va), payload.coreId);
    case "free":
      await freeEmulator();
      return {};
    case "gpu3dAck": {
      const emulator = requireEmulator();
      if (payload.success && payload.readback) {
        const { format, pixels } = payload.readback;
        if (!(pixels instanceof Uint8Array) || !Number.isInteger(format)) {
          throw new Error("GPU 3D readback acknowledgment is malformed");
        }
        if (typeof emulator.gpu_3d_complete_readback === "function") {
          return { accepted: emulator.gpu_3d_complete_readback(payload.sequence, format, pixels) };
        }
      }
      if (typeof emulator.gpu_3d_complete !== "function") {
        throw new Error("Worker VM wasm export gpu_3d_complete is unavailable");
      }
      return {
        accepted: emulator.gpu_3d_complete(payload.sequence, Boolean(payload.success)),
      };
    }
    case "installDiskSnapshot":
      return installDiskSnapshot();
    case "pause":
      state.running = false;
      return {};
    case "restoreInstallDisk":
      return restoreInstallDisk(payload.snapshot);
    case "resume":
    case "start":
      setStepSlice(payload.stepSlice);
      state.running = true;
      schedulePump();
      return {};
    case "runJitBlock":
      return runJitBlock(payload);
    case "sendUartBytes":
    case "sendUartInput":
      injectUartMessage(type, payload.input);
      return {};
    case "setStepSlice":
      setStepSlice(payload.stepSlice);
      return {};
    case "setJitEnabled":
      state.jitEnabled = Boolean(payload.enabled);
      resetJitState();
      return {};
    case "stop":
      state.running = false;
      return {};
    case "transitionToParallel": {
      const result = await transitionToParallel();
      schedulePump();
      return state.emulator ? { ...result, metrics: metrics() } : result;
    }
    default:
      throw new Error(`Unknown worker VM request: ${type}`);
  }
}
