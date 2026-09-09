// With the web server open in Playwright CLI:
// playwright-cli run-code --filename scripts/check_virgl_matrix_texture_multiply.mjs
async (page) => {
  const result = await page.evaluate(async () => {
    const { GuestDisplay } = await import("/js/gpu-display.js");
    const { ExperimentalWebGpu3dRenderer } = await import("/js/webgpu-3d.js");
    const { parseGpu3dPacket } = await import("/js/gpu-3d-packet.js");
    const { virglMatrixTextureMultiplyPacket } = await import("/js/gpu-test-virgl-matrix.mjs");
    const require = (condition, message) => { if (!condition) throw new Error(message); };
    const adapter = await navigator.gpu?.requestAdapter();
    require(adapter, "WebGPU adapter is unavailable");
    const device = await adapter.requestDevice(); const errors = [];
    device.addEventListener("uncapturederror", (event) => errors.push(event.error.message));
    const canvas = document.createElement("canvas"); const status = document.createElement("p");
    document.body.append(canvas, status);
    const display = new GuestDisplay(canvas, status);
    const target = device.createTexture({
      size: [64, 64], format: "bgra8unorm",
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
    });
    const readback = device.createBuffer({ size: 64 * 256, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
    const renderer = new ExperimentalWebGpu3dRenderer({ configure() {} });
    const backend = { device, deviceGeneration: 1, format: "bgra8unorm", canvasContext: { getCurrentTexture: () => target } };
    const samples = [];
    try {
      for (const [index, rightSampler] of [0x1092, 0x1080, 0x3292].entries()) {
        const packet = virglMatrixTextureMultiplyPacket({
          canvasWidth: 64, canvasHeight: 64, rightSampler, sequence: index + 1,
          leftPixels: new Uint8Array([100, 160, 200, 255]), rightPixels: new Uint8Array([128, 64, 255, 255]),
        });
        const acknowledgment = await display.present3d(packet);
        require(acknowledgment.success, `GuestDisplay rejected sampler ${rightSampler}: ${status.textContent}`);
        require(status.dataset.threeDAcceleration === "webgpu-virgl-capset1-matrix-texture-multiply", "Wrong renderer route");
        require(await renderer.render(backend, parseGpu3dPacket(packet)), "WebGPU render did not complete");
        const encoder = device.createCommandEncoder();
        encoder.copyTextureToBuffer({ texture: target }, { buffer: readback, bytesPerRow: 256 }, [64, 64]);
        device.queue.submit([encoder.finish()]); await readback.mapAsync(GPUMapMode.READ);
        const bytes = new Uint8Array(readback.getMappedRange());
        const inside = [...bytes.slice((32 * 64 + 40) * 4, (32 * 64 + 40) * 4 + 4)];
        const outside = [...bytes.slice(0, 4)]; readback.unmap();
        require(inside.every((value, channel) => Math.abs(value - [50, 40, 200, 255][channel]) <= 1), `Wrong BGRA product: ${inside}`);
        require(outside.every((value, channel) => Math.abs(value - [77, 51, 26, 255][channel]) <= 1), `Wrong BGRA clear: ${outside}`);
        samples.push({ rightSampler, inside, outside, acknowledgment });
      }
      await device.queue.onSubmittedWorkDone();
      require(errors.length === 0, errors.join("; "));
      return { adapter: { vendor: adapter.info.vendor, architecture: adapter.info.architecture, device: adapter.info.device }, samples, errors };
    } finally {
      renderer.invalidate(); readback.destroy(); target.destroy(); device.destroy();
      display.destroy(); canvas.remove(); status.remove();
    }
  });
  return result;
}
