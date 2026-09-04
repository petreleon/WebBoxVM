import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-depth-batch-compare-r1";

export async function renderVirglClear(session, backend, frame, isCurrent) {
  const { device } = backend;
  if (typeof device.queue.onSubmittedWorkDone !== "function") {
    throw new Error("WebGPU queue completion tracking is unavailable");
  }
  if (!isCurrent()) return false;
  session.configure(frame.canvasWidth, frame.canvasHeight);
  await captureWebGpuErrors(device, () => {
    const encoder = device.createCommandEncoder({ label: "VirGL clear encoder" });
    const pass = encoder.beginRenderPass({
      colorAttachments: [{
        clearValue: { r: frame.clearColor[0], g: frame.clearColor[1], b: frame.clearColor[2], a: frame.clearColor[3] },
        loadOp: "clear", storeOp: "store",
        view: backend.canvasContext.getCurrentTexture().createView(),
      }],
      label: "VirGL capset 1 clear pass",
    });
    pass.end();
    device.queue.submit([encoder.finish()]);
    return device.queue.onSubmittedWorkDone();
  });
  return isCurrent();
}
