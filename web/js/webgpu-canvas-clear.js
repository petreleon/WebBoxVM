export function clearWebGpuCanvas(device, context) {
  if (!device || !context) return false;
  try {
    const encoder = device.createCommandEncoder({ label: "guest display reset encoder" });
    const pass = encoder.beginRenderPass({ colorAttachments: [{
      clearValue: { a: 1, b: 0, g: 0, r: 0 },
      loadOp: "clear",
      storeOp: "store",
      view: context.getCurrentTexture().createView(),
    }], label: "guest display reset pass" });
    pass.end();
    device.queue.submit([encoder.finish()]);
    return true;
  } catch {
    // Reset remains safe when the device is already lost or unavailable.
    return false;
  }
}
