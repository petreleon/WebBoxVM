import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-depth-write-mask-r1";
import { defaultBufferUsage, ensureBuffer, paddedIndexBytes, pipelineDescriptor, renderPassDescriptor } from "./webgpu-3d-resources.js?v=20260904-virgl-depth-write-mask-r1";
import { renderVirglClear } from "./webgpu-virgl-clear.js?v=20260904-virgl-depth-write-mask-r1"; import { VirglDrawRenderer } from "./webgpu-virgl-draw.js?v=20260904-virgl-depth-write-mask-r1";
import { VirglDepthRenderer } from "./webgpu-virgl-depth.js?v=20260904-virgl-depth-write-mask-r1";
import { VirglDepthBatchRenderer } from "./webgpu-virgl-depth-batch.js?v=20260904-virgl-depth-write-mask-r1";
import { VirglSolidBatchRenderer } from "./webgpu-virgl-solid-batch.js?v=20260904-virgl-depth-write-mask-r1";
import { VirglTextureRenderer } from "./webgpu-virgl-texture.js?v=20260904-virgl-depth-write-mask-r1";
import { VirglTextureMultiplyRenderer } from "./webgpu-virgl-texture-multiply.js?v=20260904-virgl-depth-write-mask-r1";
import { VirglVertexColorRenderer } from "./webgpu-virgl-vertex-color.js?v=20260904-virgl-depth-write-mask-r1";
import { VirglTextureColorRenderer } from "./webgpu-virgl-texture-color.js?v=20260904-virgl-depth-write-mask-r1";

const SHADER = `
struct Scene { mvp: mat4x4<f32> }
@group(0) @binding(0) var<uniform> scene: Scene;
struct Output { @builtin(position) position: vec4f, @location(0) color: vec4f }
@vertex fn vertex_main(
  @location(0) position: vec3f,
  @location(1) color: vec4f,
) -> Output {
  var output: Output;
  output.position = scene.mvp * vec4f(position, 1.);
  output.color = color;
  return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f { return input.color; }
`;

export class ExperimentalWebGpu3dRenderer {
  #bindGroup;
  #bufferUsage;
  #canvasHeight = 0;
  #canvasWidth = 0;
  #depthTexture;
  #generation = 0;
  #indexBuffer;
  #indexCapacity = 0;
  #pipeline;
  #revision = 0;
  #session;
  #textureUsage;
  #uniformBuffer;
  #vertexBuffer;
  #vertexCapacity = 0;
  #virglDraw; #virglSolidBatch; #virglDepthBatch; #virglDepth; #virglTexture; #virglTextureMultiply; #virglVertexColor; #virglTextureColor;

  constructor(session, options = {}) {
    this.#session = session;
    this.#bufferUsage = options.bufferUsage ?? globalThis.GPUBufferUsage ?? defaultBufferUsage();
    this.#textureUsage = options.textureUsage ?? globalThis.GPUTextureUsage ?? { RENDER_ATTACHMENT: 0x10 };
    [this.#virglDraw, this.#virglSolidBatch, this.#virglDepthBatch, this.#virglDepth, this.#virglTexture, this.#virglTextureMultiply, this.#virglVertexColor, this.#virglTextureColor] = [
      new VirglDrawRenderer(session, options), new VirglSolidBatchRenderer(session, options), new VirglDepthBatchRenderer(session, options), new VirglDepthRenderer(session, options), new VirglTextureRenderer(session, options),
      new VirglTextureMultiplyRenderer(session, options), new VirglVertexColorRenderer(session, options), new VirglTextureColorRenderer(session, options),
    ];
  }

  async render(backend, frame, isCurrent = () => true) {
    if (frame.protocol === "virgl-clear") {
      return renderVirglClear(this.#session, backend, frame, isCurrent);
    }
    if (frame.protocol === "virgl-draw") return this.#virglDraw.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-solid-batch") return this.#virglSolidBatch.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-depth-batch") return this.#virglDepthBatch.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-depth") return this.#virglDepth.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-texture") return this.#virglTexture.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-texture-multiply") return this.#virglTextureMultiply.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-vertex-color") return this.#virglVertexColor.render(backend, frame, isCurrent); if (frame.protocol === "virgl-texture-color") return this.#virglTextureColor.render(backend, frame, isCurrent);
    const { device } = backend;
    if (typeof device.queue.onSubmittedWorkDone !== "function") {
      throw new Error("WebGPU queue completion tracking is unavailable");
    }
    try {
      if (!await this.#ensurePipeline(backend) || !isCurrent()) return false;
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
    this.#virglDraw.invalidate(); this.#virglSolidBatch.invalidate(); this.#virglDepthBatch.invalidate(); this.#virglDepth.invalidate(); this.#virglTexture.invalidate(); this.#virglTextureMultiply.invalidate(); this.#virglVertexColor.invalidate(); this.#virglTextureColor.invalidate();
    this.#uniformBuffer?.destroy?.();
    this.#vertexBuffer?.destroy?.();
    this.#indexBuffer?.destroy?.();
    this.#depthTexture?.destroy?.();
    this.#bindGroup = undefined;
    this.#uniformBuffer = undefined;
    this.#vertexBuffer = undefined;
    this.#indexBuffer = undefined;
    this.#depthTexture = undefined;
    this.#pipeline = undefined;
    this.#generation = 0;
    this.#vertexCapacity = 0;
    this.#indexCapacity = 0;
    this.#canvasWidth = 0;
    this.#canvasHeight = 0;
  }

  async #ensurePipeline(backend) {
    if (this.#generation === backend.deviceGeneration && this.#pipeline) return true;
    this.invalidate();
    this.#generation = backend.deviceGeneration;
    const revision = this.#revision;
    const { device } = backend;
    if (typeof device.createRenderPipelineAsync !== "function") {
      throw new Error("WebGPU asynchronous pipeline validation is unavailable");
    }
    const pipeline = await captureWebGpuErrors(device, () => {
      const module = device.createShaderModule({ code: SHADER, label: "experimental guest 3D shader" });
      return device.createRenderPipelineAsync(pipelineDescriptor(module, backend.format));
    });
    if (revision !== this.#revision) return false;
    this.#pipeline = pipeline;
    await captureWebGpuErrors(device, () => {
      this.#uniformBuffer = device.createBuffer({
        label: "experimental guest 3D MVP", size: 64,
        usage: this.#bufferUsage.COPY_DST | this.#bufferUsage.UNIFORM,
      });
      this.#bindGroup = device.createBindGroup({
        entries: [{ binding: 0, resource: { buffer: this.#uniformBuffer } }],
        layout: this.#pipeline.getBindGroupLayout(0),
      });
    });
    return revision === this.#revision;
  }

  #issueDraw(backend, frame) {
    const { device } = backend;
    this.#ensureBuffers(device, frame);
    this.#session.configure(frame.canvasWidth, frame.canvasHeight);
    this.#ensureDepth(backend, frame.canvasWidth, frame.canvasHeight);
    device.queue.writeBuffer(this.#uniformBuffer, 0, frame.mvp);
    if (frame.vertices.byteLength) device.queue.writeBuffer(this.#vertexBuffer, 0, frame.vertices);
    if (frame.indices.byteLength) {
      device.queue.writeBuffer(this.#indexBuffer, 0, paddedIndexBytes(frame.indices));
    }
    const encoder = device.createCommandEncoder({ label: "experimental guest 3D encoder" });
    const pass = encoder.beginRenderPass(renderPassDescriptor(backend, this.#depthTexture, frame));
    pass.setPipeline(this.#pipeline);
    pass.setBindGroup(0, this.#bindGroup);
    pass.setVertexBuffer(0, this.#vertexBuffer);
    pass.setIndexBuffer(this.#indexBuffer, "uint16");
    pass.drawIndexed(frame.indexCount);
    pass.end();
    device.queue.submit([encoder.finish()]);
    return device.queue.onSubmittedWorkDone();
  }

  #ensureBuffers(device, frame) {
    [this.#vertexBuffer, this.#vertexCapacity] = ensureBuffer(
      device, this.#vertexBuffer, this.#vertexCapacity, frame.vertices.byteLength,
      "experimental guest 3D vertices", this.#bufferUsage.COPY_DST | this.#bufferUsage.VERTEX,
    );
    [this.#indexBuffer, this.#indexCapacity] = ensureBuffer(
      device, this.#indexBuffer, this.#indexCapacity, frame.indices.byteLength,
      "experimental guest 3D indices", this.#bufferUsage.COPY_DST | this.#bufferUsage.INDEX,
    );
  }

  #ensureDepth(backend, width, height) {
    if (this.#depthTexture && this.#canvasWidth === width && this.#canvasHeight === height) return;
    this.#depthTexture?.destroy?.();
    this.#depthTexture = backend.device.createTexture({
      format: "depth24plus",
      label: "experimental guest 3D depth",
      size: { depthOrArrayLayers: 1, height, width },
      usage: this.#textureUsage.RENDER_ATTACHMENT ?? 0x10,
    });
    this.#canvasWidth = width;
    this.#canvasHeight = height;
  }
}
