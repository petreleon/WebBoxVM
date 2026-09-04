import { GPU_SCANOUT_POLL_INTERVAL_MS, state } from "./state.js?v=20260904-virgl-gpu-readback-r1";

export function maybePostGpuScanout(
  now,
  emulator = state.emulator,
  post = (message, transfer) => globalThis.postMessage(message, transfer),
) {
  if (
    !emulator ||
    typeof emulator.gpu_scanout_update !== "function" ||
    now - state.lastGpuScanoutPollAt < GPU_SCANOUT_POLL_INTERVAL_MS
  ) {
    return false;
  }

  state.lastGpuScanoutPollAt = now;
  const resetPosted = maybePostGpuReset(emulator, post);
  const update = emulator.gpu_scanout_update();
  if (!(update instanceof Uint8Array)) {
    throw new TypeError("gpu_scanout_update() must return a Uint8Array");
  }
  if (update.byteLength === 0) {
    return resetPosted;
  }

  const packet = transferablePacket(update);
  post({ event: "gpuFrame", packet }, [packet.buffer]);
  return true;
}

function maybePostGpuReset(emulator, post) {
  if (typeof emulator.gpu_reset_generation !== "function") return false;
  const generation = emulator.gpu_reset_generation();
  if (!Number.isInteger(generation) || generation < 0 || generation > 0xffff_ffff) {
    throw new TypeError("gpu_reset_generation() must return a u32");
  }
  const previous = state.gpuResetGeneration;
  state.gpuResetGeneration = generation;
  if (previous === undefined || previous === generation) return false;
  post({ event: "gpuReset", generation });
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
