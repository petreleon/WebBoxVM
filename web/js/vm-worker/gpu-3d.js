import { GPU_3D_POLL_INTERVAL_MS, state } from "./state.js?v=20260904-virgl-material-batch-r1";

export function maybePostGpu3d(
  now,
  emulator = state.emulator,
  post = (message, transfer) => globalThis.postMessage(message, transfer),
) {
  if (
    !emulator ||
    typeof emulator.gpu_3d_update !== "function" ||
    now - state.lastGpu3dPollAt < GPU_3D_POLL_INTERVAL_MS
  ) {
    return false;
  }

  state.lastGpu3dPollAt = now;
  const update = emulator.gpu_3d_update();
  if (!(update instanceof Uint8Array)) {
    throw new TypeError("gpu_3d_update() must return a Uint8Array");
  }
  if (update.byteLength === 0) {
    return false;
  }

  const packet = transferablePacket(update);
  post({ event: "gpu3dFrame", packet }, [packet.buffer]);
  return true;
}

function transferablePacket(update) {
  if (
    update.buffer instanceof ArrayBuffer &&
    update.byteOffset === 0 &&
    update.byteLength === update.buffer.byteLength
  ) {
    return update;
  }
  return update.slice();
}
