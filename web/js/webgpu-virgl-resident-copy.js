import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-readback-pool-r1";

export async function renderVirglResidentCopy(outputs, backend, frame, isCurrent) {
  const { device } = backend;
  if (!isCurrent() || typeof device.queue.onSubmittedWorkDone !== "function") return false;
  const source = outputs.get(backend, frame);
  const target = source && outputs.acquire(backend, frame);
  if (!target) return false;
  try {
    await captureWebGpuErrors(device, () => issueCopy(device, source.texture, target.texture, frame));
    if (!isCurrent()) { outputs.abandon(target); return false; }
    return outputs.publish(backend, target) && { resident: true };
  } catch (error) { outputs.abandon(target); throw error; }
}

function issueCopy(device, source, target, frame) {
  const encoder = device.createCommandEncoder({ label: "VirGL resident GPU copy" });
  if (typeof encoder.copyTextureToTexture !== "function") throw new Error("WebGPU texture copies are unavailable");
  encoder.copyTextureToTexture({ texture: source }, { texture: target }, {
    depthOrArrayLayers: 1, height: frame.canvasHeight, width: frame.canvasWidth,
  });
  device.queue.submit([encoder.finish()]);
  return device.queue.onSubmittedWorkDone();
}
