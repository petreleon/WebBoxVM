import { defaultBufferUsage, ensureBuffer } from "./webgpu-3d-resources.js?v=20260904-virgl-material-batch-r1";
import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-material-batch-r1";

const SHADER = `
struct Output { @builtin(position) position: vec4f, @location(0) color: vec4f }
@vertex fn vertex_main(@location(0) position: vec4f, @location(1) color: vec4f) -> Output {
  var output: Output;
  output.position = position;
  output.position.z = (position.z + position.w) * 0.5;
  output.color = color;
  return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f { return input.color; }
`;
const SOURCE_OVER = {
  alpha: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "one" },
  color: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "src-alpha" },
};

export class VirglVertexColorRenderer {
  #bufferUsage;
  #depthCompare;
  #depthTexture;
  #depthWriteEnabled;
  #generation = 0;
  #height = 0;
  #pipeline;
  #revision = 0;
  #session;
  #textureUsage;
  #vertexBuffer;
  #vertexCapacity = 0;
  #width = 0;

  constructor(session, options = {}) {
    this.#session = session;
    this.#bufferUsage = options.bufferUsage ?? globalThis.GPUBufferUsage ?? defaultBufferUsage();
    this.#textureUsage = options.textureUsage ?? globalThis.GPUTextureUsage ?? { RENDER_ATTACHMENT: 0x10 };
  }

  async render(backend, frame, isCurrent) {
    const { device } = backend;
    if (typeof device.queue.onSubmittedWorkDone !== "function") {
      throw new Error("WebGPU queue completion tracking is unavailable");
    }
    if (!isCurrent()) return false;
    try {
      if (!await this.#ensurePipeline(backend, frame) || !isCurrent()) return false;
      const revision = this.#revision;
      await captureWebGpuErrors(device, () => this.#issueDraw(backend, frame));
      return revision === this.#revision && isCurrent();
    } catch (error) {
      this.invalidate();
      throw error;
    }
  }

  invalidate() {
    this.#revision += 1;
    this.#vertexBuffer?.destroy?.();
    this.#depthTexture?.destroy?.();
    this.#depthCompare = undefined;
    this.#depthTexture = undefined;
    this.#depthWriteEnabled = undefined;
    this.#height = 0;
    this.#pipeline = undefined;
    this.#generation = 0;
    this.#vertexBuffer = undefined;
    this.#vertexCapacity = 0;
    this.#width = 0;
  }

  async #ensurePipeline(backend, frame) {
    if (this.#generation === backend.deviceGeneration && this.#pipeline
      && this.#depthCompare === frame.depthCompare && this.#depthWriteEnabled === frame.depthWriteEnabled) return true;
    this.invalidate();
    this.#generation = backend.deviceGeneration;
    this.#depthCompare = frame.depthCompare;
    this.#depthWriteEnabled = frame.depthWriteEnabled;
    const revision = this.#revision;
    const { device } = backend;
    if (typeof device.createRenderPipelineAsync !== "function") {
      throw new Error("WebGPU asynchronous pipeline validation is unavailable");
    }
    const module = device.createShaderModule({ code: SHADER, label: "VirGL vertex-color shader" });
    const depthStencil = frame.depthCompare && { depthCompare: frame.depthCompare,
      depthWriteEnabled: frame.depthWriteEnabled, format: "depth24plus" };
    this.#pipeline = await captureWebGpuErrors(device, () => device.createRenderPipelineAsync({
      ...(depthStencil && { depthStencil }),
      fragment: { entryPoint: "fragment_main", module, targets: [{ blend: SOURCE_OVER, format: backend.format }] },
      label: "VirGL vertex-color pipeline", layout: "auto", primitive: { topology: "triangle-list" },
      vertex: { buffers: [{ arrayStride: 32, attributes: [
        { format: "float32x4", offset: 0, shaderLocation: 0 },
        { format: "float32x4", offset: 16, shaderLocation: 1 },
      ] }], entryPoint: "vertex_main", module },
    }));
    return revision === this.#revision;
  }

  #issueDraw(backend, frame) {
    const { device } = backend;
    [this.#vertexBuffer, this.#vertexCapacity] = ensureBuffer(
      device, this.#vertexBuffer, this.#vertexCapacity, frame.vertices.byteLength,
      "VirGL vertex-color positions and colors", this.#bufferUsage.COPY_DST | this.#bufferUsage.VERTEX,
    );
    this.#session.configure(frame.canvasWidth, frame.canvasHeight);
    device.queue.writeBuffer(this.#vertexBuffer, 0, frame.vertices);
    const encoder = device.createCommandEncoder({ label: "VirGL vertex-color encoder" });
    const depthStencilAttachment = frame.depthCompare && this.#depthAttachment(backend, frame);
    const pass = encoder.beginRenderPass({ ...(depthStencilAttachment && { depthStencilAttachment }), colorAttachments: [{
      clearValue: { r: frame.clearColor[0], g: frame.clearColor[1], b: frame.clearColor[2], a: frame.clearColor[3] },
      loadOp: "clear", storeOp: "store", view: backend.canvasContext.getCurrentTexture().createView(),
    }], label: "VirGL vertex-color pass" });
    pass.setPipeline(this.#pipeline);
    pass.setVertexBuffer(0, this.#vertexBuffer);
    const viewport = webGpuViewport(frame);
    if (viewport) pass.setViewport(...viewport);
    if (frame.scissor) pass.setScissorRect(frame.scissor.x, frame.scissor.y, frame.scissor.width, frame.scissor.height);
    pass.draw(frame.vertexCount);
    pass.end();
    device.queue.submit([encoder.finish()]);
    return device.queue.onSubmittedWorkDone();
  }

  #depthAttachment(backend, frame) {
    this.#ensureDepth(backend, frame.canvasWidth, frame.canvasHeight);
    return { depthClearValue: frame.depthClear, depthLoadOp: "clear", depthStoreOp: "store", view: this.#depthTexture.createView() };
  }

  #ensureDepth(backend, width, height) {
    if (this.#depthTexture && this.#width === width && this.#height === height) return;
    this.#depthTexture?.destroy?.();
    this.#depthTexture = backend.device.createTexture({ format: "depth24plus", label: "VirGL vertex-color depth",
      size: { depthOrArrayLayers: 1, height, width }, usage: this.#textureUsage.RENDER_ATTACHMENT ?? 0x10 });
    this.#width = width; this.#height = height;
  }
}

function webGpuViewport(frame) {
  if (!frame.viewport) return undefined;
  const [sx, sy, sz, tx, ty, tz] = frame.viewport;
  return [tx - sx, frame.canvasHeight - ty - sy, sx * 2, sy * 2, tz - sz, tz + sz];
}
