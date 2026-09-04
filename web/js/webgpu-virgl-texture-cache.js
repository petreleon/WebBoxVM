import { padBgraRows } from "./gpu-scanout-packet.js?v=20260904-virgl-readback-pool-r1";

const MAX_BIND_GROUPS = 64;
const MAX_CACHE_BYTES = 4 * 1024 * 1024;
const MAX_CACHE_ENTRIES = 32;

export class VirglTextureSnapshotCache {
  #bindings = new Map(); #bindingCount = 0; #bytes = 0; #clock = 0; #count = 0; #entries = new Map(); #nextId = 1; #samplers = new Map(); #usage;

  constructor(options = {}) {
    const usage = options.textureUsage ?? globalThis.GPUTextureUsage ?? {};
    this.#usage = { COPY_DST: usage.COPY_DST ?? 2, TEXTURE_BINDING: usage.TEXTURE_BINDING ?? 4 };
  }

  bindGroup(device, pipeline, snapshots, retired) {
    const bindings = this.#resolve(device, snapshots, retired);
    if (bindings.some(({ entry }) => entry.id === undefined)) return createBindGroup(device, pipeline, bindings);
    const key = bindings.map(({ entry, samplerKey }) => `${entry.id}:${samplerKey}`).join("|");
    const existing = this.#bindings.get(pipeline)?.get(key);
    if (existing) return existing;
    if (this.#bindingCount === MAX_BIND_GROUPS) this.#clearBindings();
    const groups = this.#bindings.get(pipeline) ?? new Map();
    const group = createBindGroup(device, pipeline, bindings);
    groups.set(key, group); this.#bindings.set(pipeline, groups); this.#bindingCount += 1;
    return group;
  }

  bindGroupEntries(device, snapshots, retired) {
    return entries(this.#resolve(device, snapshots, retired));
  }

  invalidate() {
    for (const entries of this.#entries.values()) for (const entry of entries) entry.texture.destroy?.();
    this.#clearBindings();
    this.#bytes = 0; this.#clock = 0; this.#count = 0; this.#entries.clear(); this.#nextId = 1; this.#samplers.clear();
  }

  #resolve(device, snapshots, retired) {
    return snapshots.map((snapshot) => {
      const samplerKey = `${snapshot.addressMode}:${snapshot.filter}`;
      return { entry: this.#texture(device, snapshot, retired), sampler: this.#sampler(device, snapshot, samplerKey), samplerKey };
    });
  }

  #texture(device, snapshot, retired) {
    if (snapshot.gpuTexture) return { texture: snapshot.gpuTexture };
    const key = snapshotKey(snapshot); const entries = this.#entries.get(key) ?? [];
    const existing = entries.find((entry) => sameSnapshot(entry, snapshot));
    if (existing) { existing.age = ++this.#clock; return existing; }
    const texture = device.createTexture({
      format: "bgra8unorm", label: "VirGL material snapshot",
      size: { width: snapshot.width, height: snapshot.height, depthOrArrayLayers: 1 },
      usage: this.#usage.COPY_DST | this.#usage.TEXTURE_BINDING,
    });
    try {
      const upload = padBgraRows(snapshot.pixels, snapshot.width, snapshot.height);
      device.queue.writeTexture({ texture }, upload.data, { bytesPerRow: upload.bytesPerRow, rowsPerImage: snapshot.height }, { width: snapshot.width, height: snapshot.height, depthOrArrayLayers: 1 });
    } catch (error) { texture.destroy?.(); throw error; }
    const pixels = new Uint8Array(snapshot.pixels);
    if (pixels.byteLength > MAX_CACHE_BYTES) { retired.push(texture); return { texture }; }
    while (this.#count >= MAX_CACHE_ENTRIES || this.#bytes + pixels.byteLength > MAX_CACHE_BYTES) {
      if (!this.#evict(retired)) { retired.push(texture); return { texture }; }
    }
    const entry = { age: ++this.#clock, height: snapshot.height, id: this.#nextId++, pixels, texture, width: snapshot.width };
    entries.push(entry); this.#entries.set(key, entries); this.#bytes += pixels.byteLength; this.#count += 1;
    return entry;
  }

  #sampler(device, snapshot, key) {
    let sampler = this.#samplers.get(key);
    if (!sampler) {
      sampler = device.createSampler({ addressModeU: snapshot.addressMode, addressModeV: snapshot.addressMode, magFilter: snapshot.filter, minFilter: snapshot.filter, mipmapFilter: "nearest" });
      this.#samplers.set(key, sampler);
    }
    return sampler;
  }

  #evict(retired) {
    let oldest; let oldestKey;
    for (const [key, entries] of this.#entries) for (const entry of entries) {
      if (!oldest || entry.age < oldest.age) { oldest = entry; oldestKey = key; }
    }
    if (!oldest) return false;
    const entries = this.#entries.get(oldestKey); entries.splice(entries.indexOf(oldest), 1);
    if (!entries.length) this.#entries.delete(oldestKey);
    this.#bytes -= oldest.pixels.byteLength; this.#count -= 1; this.#clearBindings(); retired.push(oldest.texture); return true;
  }

  #clearBindings() { this.#bindings.clear(); this.#bindingCount = 0; }
}

function createBindGroup(device, pipeline, bindings) {
  return device.createBindGroup({ entries: entries(bindings), layout: pipeline.getBindGroupLayout(0) });
}

function entries(bindings) {
  return bindings.flatMap(({ entry, sampler }, index) => [
    { binding: index * 2, resource: entry.texture.createView() },
    { binding: index * 2 + 1, resource: sampler },
  ]);
}

function snapshotKey(snapshot) {
  return `${snapshot.width}x${snapshot.height}:${snapshot.pixels.byteLength}:${hash(snapshot.pixels)}`;
}

function sameSnapshot(entry, snapshot) {
  if (entry.width !== snapshot.width || entry.height !== snapshot.height || entry.pixels.byteLength !== snapshot.pixels.byteLength) return false;
  for (let index = 0; index < entry.pixels.byteLength; index += 1) if (entry.pixels[index] !== snapshot.pixels[index]) return false;
  return true;
}

function hash(bytes) {
  let value = 2166136261;
  for (const byte of bytes) value = Math.imul(value ^ byte, 16777619);
  return value >>> 0;
}
