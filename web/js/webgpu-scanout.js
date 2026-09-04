import { padBgraRows } from "./gpu-scanout-packet.js?v=20260904-virgl-depth-r1";

const SHADER = `
@group(0) @binding(0) var guest_texture: texture_2d<f32>;
@group(0) @binding(1) var guest_sampler: sampler;
struct Output { @builtin(position) position: vec4f, @location(0) uv: vec2f }
@vertex fn vertex_main(@builtin(vertex_index) i: u32) -> Output {
  var positions = array<vec2f, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
  let position = positions[i];
  var output: Output;
  output.position = vec4f(position, 0., 1.);
  output.uv = vec2f((position.x + 1.) * .5, (1. - position.y) * .5);
  return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f {
  return textureSampleLevel(guest_texture, guest_sampler, input.uv, 0.);
}`;

export class WebGpuScanoutRenderer {
  #bindGroup;
  #generation = 0;
  #needsFull = true;
  #pipeline;
  #sampler;
  #session;
  #texture;
  #textureHeight = 0;
  #textureUsage;
  #textureWidth = 0;

  constructor(session, textureUsage = globalThis.GPUTextureUsage ?? defaults()) {
    this.#session = session;
    this.#textureUsage = textureUsage;
  }

  get texture() {
    return this.#texture;
  }

  render(backend, state, requestedDirty) {
    this.#ensurePipeline(backend);
    this.#session.configure(state.width, state.height);
    this.#ensureTexture(backend, state.width, state.height);
    const dirty = this.#needsFull ? state.fullRect() : requestedDirty;
    const { bytesPerRow, data } = padBgraRows(state.extract(dirty), dirty.width, dirty.height);
    backend.device.queue.writeTexture(
      { origin: { x: dirty.x, y: dirty.y, z: 0 }, texture: this.#texture },
      data,
      { bytesPerRow, rowsPerImage: dirty.height },
      { depthOrArrayLayers: 1, height: dirty.height, width: dirty.width },
    );
    this.#needsFull = false;
    const encoder = backend.device.createCommandEncoder({ label: "guest scanout encoder" });
    const pass = encoder.beginRenderPass({
      colorAttachments: [{
        clearValue: { a: 1, b: 0, g: 0, r: 0 },
        loadOp: "clear",
        storeOp: "store",
        view: backend.canvasContext.getCurrentTexture().createView(),
      }],
      label: "guest scanout pass",
    });
    pass.setPipeline(this.#pipeline);
    pass.setBindGroup(0, this.#bindGroup);
    pass.draw(3);
    pass.end();
    backend.device.queue.submit([encoder.finish()]);
  }

  reset() {
    this.#texture?.destroy?.();
    this.#texture = undefined;
    this.#textureWidth = 0;
    this.#textureHeight = 0;
    this.#bindGroup = undefined;
    this.#needsFull = true;
  }

  invalidate() {
    this.reset();
    this.#generation = 0;
    this.#pipeline = undefined;
    this.#sampler = undefined;
  }

  #ensurePipeline(backend) {
    if (this.#generation === backend.deviceGeneration && this.#pipeline) return;
    this.invalidate();
    this.#generation = backend.deviceGeneration;
    const module = backend.device.createShaderModule({ code: SHADER, label: "guest scanout shader" });
    this.#pipeline = backend.device.createRenderPipeline({
      fragment: { entryPoint: "fragment_main", module, targets: [{ format: backend.format }] },
      label: "guest scanout pipeline",
      layout: "auto",
      primitive: { topology: "triangle-list" },
      vertex: { entryPoint: "vertex_main", module },
    });
    this.#sampler = backend.device.createSampler({ magFilter: "nearest", minFilter: "nearest" });
  }

  #ensureTexture(backend, width, height) {
    if (this.#texture && this.#textureWidth === width && this.#textureHeight === height) return;
    const limit = backend.device.limits?.maxTextureDimension2D;
    if (limit && (width > limit || height > limit)) {
      throw new Error(`Guest scanout ${width}×${height} exceeds WebGPU limit ${limit}`);
    }
    this.#texture?.destroy?.();
    this.#texture = backend.device.createTexture({
      format: "bgra8unorm",
      label: "guest scanout texture",
      size: { depthOrArrayLayers: 1, height, width },
      usage: this.#textureUsage.COPY_DST | this.#textureUsage.TEXTURE_BINDING |
        (this.#textureUsage.RENDER_ATTACHMENT ?? 0),
    });
    this.#textureWidth = width;
    this.#textureHeight = height;
    this.#bindGroup = backend.device.createBindGroup({
      entries: [
        { binding: 0, resource: this.#texture.createView() },
        { binding: 1, resource: this.#sampler },
      ],
      layout: this.#pipeline.getBindGroupLayout(0),
    });
    this.#needsFull = true;
  }
}

function defaults() {
  return { COPY_DST: 0x02, RENDER_ATTACHMENT: 0x10, TEXTURE_BINDING: 0x04 };
}
