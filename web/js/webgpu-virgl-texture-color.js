import { defaultBufferUsage, ensureBuffer } from "./webgpu-3d-resources.js?v=20260904-virgl-depth-batch-compare-r1";
import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-depth-batch-compare-r1";
import { padBgraRows } from "./gpu-scanout-packet.js?v=20260904-virgl-depth-batch-compare-r1";

const SHADER = `
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var sampler0: sampler;
struct Output { @builtin(position) position: vec4f, @location(0) color: vec4f, @location(1) uv: vec2f }
@vertex fn vertex_main(@location(0) position: vec4f, @location(1) color: vec4f, @location(2) uv: vec2f) -> Output {
  var output: Output;
  output.position = position;
  output.position.z = (position.z + position.w) * 0.5;
  output.color = color;
  output.uv = uv;
  return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f {
  return textureSampleLevel(source, sampler0, vec2f(input.uv.x, 1.0 - input.uv.y), 0.0) * input.color;
}
`;
const SOURCE_OVER = {
  alpha: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "one" },
  color: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "src-alpha" },
};

export class VirglTextureColorRenderer {
  #bindGroup; #bufferUsage; #generation = 0; #pipeline; #revision = 0; #sampler; #samplerKey;
  #session; #texture; #textureHeight = 0; #textureUsage; #textureWidth = 0; #vertexBuffer; #vertexCapacity = 0;

  constructor(session, options = {}) {
    this.#session = session;
    this.#bufferUsage = options.bufferUsage ?? globalThis.GPUBufferUsage ?? defaultBufferUsage();
    this.#textureUsage = options.textureUsage ?? globalThis.GPUTextureUsage
      ?? { COPY_DST: 0x2, TEXTURE_BINDING: 0x4 };
  }

  async render(backend, frame, isCurrent) {
    const { device } = backend;
    if (typeof device.queue.onSubmittedWorkDone !== "function") throw new Error("WebGPU queue completion tracking is unavailable");
    if (!isCurrent()) return false;
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
    this.#vertexBuffer?.destroy?.(); this.#texture?.destroy?.();
    this.#bindGroup = undefined; this.#pipeline = undefined; this.#sampler = undefined; this.#samplerKey = undefined;
    this.#texture = undefined; this.#generation = 0; this.#textureHeight = 0; this.#textureWidth = 0;
    this.#vertexBuffer = undefined; this.#vertexCapacity = 0;
  }

  async #ensurePipeline(backend) {
    if (this.#generation === backend.deviceGeneration && this.#pipeline) return true;
    this.invalidate(); this.#generation = backend.deviceGeneration;
    const revision = this.#revision;
    const { device } = backend;
    if (typeof device.createRenderPipelineAsync !== "function") throw new Error("WebGPU asynchronous pipeline validation is unavailable");
    const module = device.createShaderModule({ code: SHADER, label: "VirGL texture-color shader" });
    this.#pipeline = await captureWebGpuErrors(device, () => device.createRenderPipelineAsync({
      fragment: { entryPoint: "fragment_main", module, targets: [{ blend: SOURCE_OVER, format: backend.format }] },
      label: "VirGL texture-color pipeline", layout: "auto", primitive: { topology: "triangle-list" },
      vertex: { buffers: [{ arrayStride: 40, attributes: [
        { format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 },
        { format: "float32x2", offset: 32, shaderLocation: 2 },
      ] }], entryPoint: "vertex_main", module },
    }));
    return revision === this.#revision;
  }

  #issueDraw(backend, frame) {
    const { device, format, canvasContext } = backend;
    [this.#vertexBuffer, this.#vertexCapacity] = ensureBuffer(
      device, this.#vertexBuffer, this.#vertexCapacity, frame.vertices.byteLength,
      "VirGL texture-color vertices", this.#bufferUsage.COPY_DST | this.#bufferUsage.VERTEX,
    );
    this.#ensureSampler(device, frame.texture); this.#ensureTexture(device, frame.texture);
    this.#session.configure(frame.canvasWidth, frame.canvasHeight);
    device.queue.writeBuffer(this.#vertexBuffer, 0, frame.vertices);
    const upload = padBgraRows(frame.texture.pixels, frame.texture.width, frame.texture.height);
    device.queue.writeTexture({ texture: this.#texture }, upload.data, {
      bytesPerRow: upload.bytesPerRow, rowsPerImage: frame.texture.height,
    }, { width: frame.texture.width, height: frame.texture.height, depthOrArrayLayers: 1 });
    const encoder = device.createCommandEncoder({ label: "VirGL texture-color encoder" });
    const pass = encoder.beginRenderPass({ colorAttachments: [{
      clearValue: { r: frame.clearColor[0], g: frame.clearColor[1], b: frame.clearColor[2], a: frame.clearColor[3] },
      loadOp: "clear", storeOp: "store", view: canvasContext.getCurrentTexture().createView(),
    }], label: "VirGL texture-color pass" });
    pass.setPipeline(this.#pipeline); pass.setBindGroup(0, this.#bindGroup); pass.setVertexBuffer(0, this.#vertexBuffer);
    const viewport = webGpuViewport(frame);
    if (viewport) pass.setViewport(...viewport);
    if (frame.scissor) pass.setScissorRect(frame.scissor.x, frame.scissor.y, frame.scissor.width, frame.scissor.height);
    pass.draw(frame.vertexCount); pass.end(); device.queue.submit([encoder.finish()]);
    return device.queue.onSubmittedWorkDone();
  }

  #ensureTexture(device, texture) {
    if (!this.#texture || this.#textureWidth !== texture.width || this.#textureHeight !== texture.height) {
      this.#texture?.destroy?.();
      this.#texture = device.createTexture({ format: "bgra8unorm", label: "VirGL texture-color source",
        size: { width: texture.width, height: texture.height, depthOrArrayLayers: 1 },
        usage: this.#textureUsage.COPY_DST | this.#textureUsage.TEXTURE_BINDING });
      this.#textureWidth = texture.width; this.#textureHeight = texture.height; this.#bindGroup = undefined;
    }
    if (!this.#bindGroup) this.#bindGroup = device.createBindGroup({ entries: [
      { binding: 0, resource: this.#texture.createView() }, { binding: 1, resource: this.#sampler },
    ], layout: this.#pipeline.getBindGroupLayout(0) });
  }

  #ensureSampler(device, texture) {
    const key = `${texture.addressMode}/${texture.filter}`;
    if (this.#sampler && this.#samplerKey === key) return;
    this.#sampler = device.createSampler({ addressModeU: texture.addressMode, addressModeV: texture.addressMode,
      magFilter: texture.filter, minFilter: texture.filter, mipmapFilter: "nearest" });
    this.#samplerKey = key; this.#bindGroup = undefined;
  }
}

function webGpuViewport(frame) {
  if (!frame.viewport) return undefined;
  const [sx, sy, sz, tx, ty, tz] = frame.viewport;
  return [tx - sx, frame.canvasHeight - ty - sy, sx * 2, sy * 2, tz - sz, tz + sz];
}
