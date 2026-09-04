const MAX_OUTPUT_BYTES = 4 * 1024 * 1024;
const MAX_OUTPUTS = 16;
const MAX_OUTPUT_TOTAL_BYTES = 16 * 1024 * 1024;

export class VirglResidentOutputTargets {
  #bytes = 0;
  #generation = 0;
  #outputs = new Map();
  #reserved = new Map();

  acquire(backend, frame) {
    if (!frame.residentCandidate || !this.#ready(backend)) return undefined;
    if (frame.residentPreviousProducer) return this.#replacement(frame);
    if (this.#reserved.size >= MAX_OUTPUTS) return undefined;
    const bytes = checkedBytes(frame.canvasWidth, frame.canvasHeight);
    if (!bytes || bytes > MAX_OUTPUT_BYTES || this.#bytes + bytes > MAX_OUTPUT_TOTAL_BYTES) return undefined;
    const texture = backend.device.createTexture({
      format: backend.format,
      label: `VirGL resident output ${frame.sequence}`,
      size: { depthOrArrayLayers: 1, height: frame.canvasHeight, width: frame.canvasWidth },
      usage: textureUsage().COPY_SRC | textureUsage().RENDER_ATTACHMENT,
    });
    const output = { bytes, height: frame.canvasHeight, sequence: frame.sequence, texture, width: frame.canvasWidth };
    this.#reserved.set(texture, bytes); this.#bytes += bytes;
    return output;
  }

  publish(backend, output) {
    if (!output || this.#generation !== backend.deviceGeneration) {
      this.abandon(output); return false;
    }
    if (output.previousSequence) return this.#publishReplacement(output);
    if (this.#outputs.has(output.sequence)) { this.discard(output); return false; }
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

  abandon(output) {
    if (output?.previousSequence) this.release(output.previousSequence);
    else this.discard(output);
  }

  discard(output) {
    const bytes = this.#reserved.get(output?.texture);
    if (bytes !== undefined) { this.#reserved.delete(output.texture); this.#bytes -= bytes; }
    output?.texture?.destroy?.();
  }

  invalidate() {
    for (const texture of this.#reserved.keys()) texture.destroy?.();
    this.#bytes = 0; this.#outputs.clear(); this.#reserved.clear(); this.#generation = 0;
  }

  #ready(backend) {
    if (this.#generation === backend.deviceGeneration) return true;
    this.invalidate(); this.#generation = backend.deviceGeneration;
    return true;
  }

  #replacement(frame) {
    const output = this.#outputs.get(frame.residentPreviousProducer);
    if (!output || output.width !== frame.canvasWidth || output.height !== frame.canvasHeight) return undefined;
    return { ...output, previousSequence: frame.residentPreviousProducer, sequence: frame.sequence };
  }

  #publishReplacement(output) {
    const previous = this.#outputs.get(output.previousSequence);
    if (!previous || previous.texture !== output.texture || this.#outputs.has(output.sequence)) {
      this.abandon(output); return false;
    }
    this.#outputs.delete(output.previousSequence);
    previous.sequence = output.sequence;
    this.#outputs.set(output.sequence, previous);
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
