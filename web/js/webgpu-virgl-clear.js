import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-readback-pool-r1";

export async function renderVirglClear(session, outputs, backend, frame, isCurrent) {
  const { device } = backend;
  if (typeof device.queue.onSubmittedWorkDone !== "function") {
    throw new Error("WebGPU queue completion tracking is unavailable");
  }
  if (!isCurrent()) return false;
  const output = outputs.acquire(backend, frame);
  try {
    session.configure(frame.canvasWidth, frame.canvasHeight);
    await captureWebGpuErrors(device, () => issueClear(backend, frame, output));
    if (!isCurrent()) { outputs.abandon(output); return false; }
    return output ? outputs.publish(backend, output) && { resident: true } : true;
  } catch (error) { outputs.abandon(output); throw error; }
}

function issueClear(backend, frame, output) {
  const { device } = backend;
  const encoder = device.createCommandEncoder({ label: "VirGL clear encoder" });
  const canvas = backend.canvasContext.getCurrentTexture();
  const target = output?.texture ?? canvas;
  const pass = encoder.beginRenderPass({
    colorAttachments: [{
      clearValue: { r: frame.clearColor[0], g: frame.clearColor[1], b: frame.clearColor[2], a: frame.clearColor[3] },
      loadOp: "clear", storeOp: "store", view: target.createView(),
    }],
    label: "VirGL capset 1 clear pass",
  });
  pass.end();
  if (output) {
    if (typeof encoder.copyTextureToTexture !== "function") throw new Error("WebGPU texture copies are unavailable");
    encoder.copyTextureToTexture({ texture: target }, { texture: canvas }, {
      depthOrArrayLayers: 1, height: frame.canvasHeight, width: frame.canvasWidth,
    });
  }
  device.queue.submit([encoder.finish()]);
  return device.queue.onSubmittedWorkDone();
}
