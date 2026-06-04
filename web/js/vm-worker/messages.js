import {
  bootIsoWithDisk,
  freeEmulator,
  installDiskSnapshot,
  restoreInstallDisk,
  setStepSlice,
} from "./lifecycle.js";
import { compileJitBlock } from "./jit-compile.js";
import { errorMessage } from "./errors.js";
import { runJitBlock } from "./jit-run.js";
import { schedulePump } from "./pump.js";
import { state } from "./state.js";

export async function handleMessage(message) {
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
      state.emulator?.send_uart_bytes(payload.input);
      return {};
    case "sendUartInput":
      state.emulator?.send_uart_input(payload.input);
      return {};
    case "setStepSlice":
      setStepSlice(payload.stepSlice);
      return {};
    case "stop":
      state.running = false;
      return {};
    default:
      throw new Error(`Unknown worker VM request: ${type}`);
  }
}
