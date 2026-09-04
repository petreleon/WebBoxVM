const MAX_OUTPUT_BYTES = 4 * 1024 * 1024;
const MAX_OUTPUTS = 4;

export class VirglResidentOutputTargets {
  #generation = 0;
  #outputs = new Map();

  acquire(backend, frame) {
    if (!frame.residentCandidate || !this.#ready(backend) || this.#outputs.size >= MAX_OUTPUTS) return undefined;
    const bytes = checkedBytes(frame.canvasWidth, frame.canvasHeight);
    if (!bytes || bytes > MAX_OUTPUT_BYTES) return undefined;
    const texture = backend.device.createTexture({
      format: backend.format,
      label: `VirGL resident output ${frame.sequence}`,
      size: { depthOrArrayLayers: 1, height: frame.canvasHeight, width: frame.canvasWidth },
      usage: textureUsage().COPY_SRC | textureUsage().RENDER_ATTACHMENT,
    });
    return { height: frame.canvasHeight, sequence: frame.sequence, texture, width: frame.canvasWidth };
  }

  publish(backend, output) {
    if (!output || this.#generation !== backend.deviceGeneration || this.#outputs.has(output.sequence)) {
      this.discard(output); return false;
    }
    this.#outputs.set(output.sequence, output);
    return true;
  }

  get(backend, frame) {
    if (!this.#ready(backend)) return undefined;
    const output = this.#outputs.get(frame.producerSequence);
    return output?.width === frame.canvasWidth && output.height === frame.canvasHeight ? output : undefined;
  }

  release(sequence) {
    const output = this.#outputs.get(sequence);
    this.#outputs.delete(sequence);
    this.discard(output);
  }

  discard(output) { output?.texture?.destroy?.(); }

  invalidate() {
    for (const output of this.#outputs.values()) this.discard(output);
    this.#outputs.clear(); this.#generation = 0;
  }

  #ready(backend) {
    if (this.#generation === backend.deviceGeneration) return true;
    this.invalidate(); this.#generation = backend.deviceGeneration;
    return true;
  }
}

function checkedBytes(width, height) {
  const bytes = width * height * 4;
  return Number.isSafeInteger(bytes) && bytes > 0 ? bytes : undefined;
}

function textureUsage() {
  return globalThis.GPUTextureUsage ?? { COPY_SRC: 1, RENDER_ATTACHMENT: 16 };
}
