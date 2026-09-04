const MAGIC = [0x56, 0x47, 0x4d, 0x31];
const HEADER_BYTES = 48;
const REPLACEMENT_HEADER_BYTES = 52;
const DRAW_BYTES = 52;
const MAX_DIMENSION = 8192;
const MAX_DRAWS = 16;
const MAX_TEXTURE_DIMENSION = 64;
const MAX_VERTICES = 3063;
const RGB_WRITE_MASK = 7;
const DEPTH_COMPARE = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];

export function isVirglMaterialBatchPacket(packet) {
  return packet instanceof Uint8Array && MAGIC.every((byte, index) => packet[index] === byte);
}

export function extractVirglMaterialBatchSequence(packet) {
  if (!isVirglMaterialBatchPacket(packet) || packet.byteLength < 12) return undefined;
  return new DataView(packet.buffer, packet.byteOffset, packet.byteLength).getUint32(8, true) || undefined;
}

export function parseVirglMaterialBatchPacket(packet) {
  if (!isVirglMaterialBatchPacket(packet) || packet.byteLength < HEADER_BYTES) throw new Error("VirGL material-batch packet has invalid VGM1 framing");
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const [version, sequence, canvasWidth, canvasHeight, drawCount, flags] = [4, 8, 12, 16, 20, 24].map((offset) => view.getUint32(offset, true));
  const residentCandidate = [2, 3].includes(version); const replace = [4, 5, 6, 7, 8, 9].includes(version);
  const masked = [8, 9].includes(version); const depth = version === 1 ? flags === 1 : [5, 7, 9].includes(version);
  const writeMask = masked ? flags : [6, 7].includes(version) ? RGB_WRITE_MASK : 0xF;
  if (![1, 2, 3, 4, 5, 6, 7, 8, 9].includes(version) || !sequence || drawCount < 1 || drawCount > MAX_DRAWS
    || (version === 1 && drawCount < 2)
    || (masked ? flags < 1 || flags > 0xF : version === 1 ? flags > 1 : residentCandidate ? flags !== 2 : flags !== (depth ? 1 : 0))
    || (version === 3 && packet.byteLength < REPLACEMENT_HEADER_BYTES)) {
    throw new Error("VirGL material-batch packet has invalid VGM1 framing");
  }
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) throw new Error(`VirGL material-batch dimensions must be between 1 and ${MAX_DIMENSION}`);
  const clearColor = colors(view, 28, "clear");
  if (view.getFloat32(44, true) !== (depth ? 1 : 0)) throw new Error("VirGL material-batch depth clear is invalid");
  const residentPreviousProducer = version === 3 ? view.getUint32(48, true) : undefined;
  if (version === 3 && (!residentPreviousProducer || residentPreviousProducer === sequence)) {
    throw new Error("VirGL material-batch replacement producer is invalid");
  }
  let offset = version === 3 ? REPLACEMENT_HEADER_BYTES : HEADER_BYTES;
  let totalVertices = 0;
  const draws = [];
  for (let index = 0; index < drawCount; index += 1) {
    const result = draw(view, packet, offset, canvasWidth, canvasHeight, depth);
    totalVertices += result.draw.vertexCount;
    if (totalVertices > MAX_DRAWS * MAX_VERTICES) throw new Error("VirGL material-batch vertex budget is invalid");
    draws.push(result.draw); offset = result.next;
  }
  if (offset !== packet.byteLength) throw new Error("VirGL material-batch packet has trailing bytes");
  return {
    acceleration: depth ? "webgpu-virgl-capset1-depth-material-batch" : "webgpu-virgl-capset1-material-batch",
    blend: replace ? "replace" : "source-over",
    canvasHeight, canvasWidth, capsetId: 1, clearColor, depth, depthClear: depth ? 1 : 0, draws, writeMask,
    presentationLabel: depth ? "VirGL capset 1 mixed-material depth batch" : "VirGL capset 1 mixed-material draw batch",
    protocol: "virgl-material-batch", residentCandidate, residentPreviousProducer, sequence, version,
  };
}

function draw(view, packet, offset, width, height, depth) {
  if (offset + DRAW_BYTES > packet.byteLength) throw new Error("VirGL material-batch draw is truncated");
  const kind = view.getUint32(offset, true);
  const vertexCount = view.getUint32(offset + 8, true);
  const material = materialName(kind);
  if (!material || vertexCount < 3 || vertexCount > MAX_VERTICES || vertexCount % 3) throw new Error("VirGL material-batch draw type or vertex count is invalid");
  if (depth && material === "texture-pair") throw new Error("VirGL material-batch depth material is unsupported");
  const depthState = readDepth(view.getUint32(offset + 4, true), depth);
  const viewport = floats(view, offset + 12, 6); validateViewport(viewport, width, height);
  const scissor = readScissor(view, offset + 36, width, height);
  let cursor = offset + DRAW_BYTES;
  const decoded = materialData(view, packet, cursor, material);
  cursor = decoded.next;
  const vertexBytes = vertexCount * stride(material);
  if (cursor + vertexBytes > packet.byteLength) throw new Error("VirGL material-batch vertices are truncated");
  const vertices = floats(view, cursor, vertexBytes / 4);
  if (!validVertices(vertices, material)) throw new Error("VirGL material-batch vertices are invalid");
  return { draw: { ...decoded, ...depthState, material, scissor, vertexCount, vertices, viewport }, next: cursor + vertexBytes };
}

function materialData(view, packet, offset, material) {
  if (material === "solid") return { drawColor: colors(view, offset, "draw"), next: offset + 16 };
  if (material === "vertex-color") return { next: offset };
  const first = texture(view, packet, offset);
  if (material !== "texture-pair") return { texture: first.texture, next: first.next };
  const second = texture(view, packet, first.next);
  return { textures: [first.texture, second.texture], next: second.next };
}

function texture(view, packet, offset) {
  if (offset + 12 > packet.byteLength) throw new Error("VirGL material-batch texture is truncated");
  const sampler = samplerConfig(view.getUint32(offset, true));
  const width = view.getUint32(offset + 4, true); const height = view.getUint32(offset + 8, true);
  const bytes = width && height && width <= MAX_TEXTURE_DIMENSION && height <= MAX_TEXTURE_DIMENSION ? width * height * 4 : 0;
  const next = offset + 12 + bytes;
  if (!sampler || !bytes || next > packet.byteLength) throw new Error("VirGL material-batch texture framing is invalid");
  return { next, texture: { ...sampler, height, pixels: packet.subarray(offset + 12, next), width } };
}

function readDepth(word, depth) {
  if (!depth && word === 0) return {};
  if (!depth || (word & 1) === 0 || word > 31 || !DEPTH_COMPARE[word >> 2]) throw new Error("VirGL material-batch depth state is invalid");
  return { depthCompare: DEPTH_COMPARE[word >> 2], depthWriteEnabled: (word & 2) !== 0 };
}

function materialName(kind) {
  return [undefined, "solid", "vertex-color", "texture", "texture-pair", "texture-color"][kind];
}

function stride(material) {
  return ({ solid: 16, "vertex-color": 32, texture: 24, "texture-pair": 24, "texture-color": 40 })[material];
}

function colors(view, offset, label) {
  const values = floats(view, offset, 4);
  if (![...values].every((value) => Number.isFinite(value) && value >= 0 && value <= 1)) throw new Error(`VirGL material-batch ${label} color is invalid`);
  return values;
}

function validVertices(vertices, material) {
  const width = stride(material) / 4;
  const valid = vertices.every((value, index) => {
    const component = index % width;
    if (component < 3) return Number.isFinite(value) && value >= -1 && value <= 1;
    if (component === 3) return value === 1;
    if (material === "vertex-color" || material === "texture-color") return component < 8 ? Number.isFinite(value) && value >= 0 && value <= 1 : Number.isFinite(value) && value >= -8 && value <= 8;
    return Number.isFinite(value) && value >= -8 && value <= 8;
  });
  for (let base = 0; valid && base < vertices.length; base += width * 3) {
    const [ax, ay] = vertices.subarray(base, base + 2); const [bx, by] = vertices.subarray(base + width, base + width + 2); const [cx, cy] = vertices.subarray(base + width * 2, base + width * 2 + 2);
    if (Math.abs((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) < 0.001) return false;
  }
  return valid;
}

function validateViewport(viewport, width, height) {
  const [sx, sy, sz, tx, ty, tz] = viewport;
  if (!viewport.every(Number.isFinite) || sx <= 0 || sy <= 0 || sz < 0 || tx - sx < 0 || tx + sx > width || ty - sy < 0 || ty + sy > height || tz - sz < 0 || tz + sz > 1) throw new Error("VirGL material-batch viewport must fit its target");
}

function readScissor(view, offset, width, height) {
  const [x, y, scissorWidth, scissorHeight] = [0, 4, 8, 12].map((delta) => view.getUint32(offset + delta, true));
  if (x === 0 && y === 0 && scissorWidth === 0 && scissorHeight === 0) return undefined;
  if (!scissorWidth || !scissorHeight || x + scissorWidth > width || y + scissorHeight > height) throw new Error("VirGL material-batch scissor must fit its target");
  return { height: scissorHeight, width: scissorWidth, x, y };
}

function samplerConfig(word) {
  if (word === 0x1092) return { addressMode: "clamp-to-edge", filter: "nearest" };
  if (word === 0x1080) return { addressMode: "repeat", filter: "nearest" };
  if (word === 0x3292) return { addressMode: "clamp-to-edge", filter: "linear" };
}

function floats(view, offset, count) {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) values[index] = view.getFloat32(offset + index * 4, true);
  return values;
}
