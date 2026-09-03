const MAGIC = [0x56, 0x47, 0x44, 0x31]; // VGD1
const LEGACY_BYTES = 104;
const STATE_BYTES = 144;
const TEXTURED_BYTES = 176;
const TEXTURED_PAIR_BYTES = 184;
const REPEAT_TEXTURED_BYTES = 180;
const CLAMP_NEAREST = { addressMode: "clamp-to-edge", filter: "nearest" };
const REPEAT_NEAREST = { addressMode: "repeat", filter: "nearest" };
const CLAMP_LINEAR = { addressMode: "clamp-to-edge", filter: "linear" };
const MAX_DIMENSION = 8192;
const MAX_TEXTURE_DIMENSION = 64;
const VERTEX_COUNT = 3;

export function isVirglDrawPacket(packet) {
  return packet instanceof Uint8Array && MAGIC.every((byte, index) => packet[index] === byte);
}

export function extractVirglDrawSequence(packet) {
  if (!isVirglDrawPacket(packet) || packet.byteLength < 12) return undefined;
  const sequence = new DataView(packet.buffer, packet.byteOffset, 12).getUint32(8, true);
  return sequence === 0 ? undefined : sequence;
}

export function parseVirglDrawPacket(packet) {
  if (!(packet instanceof Uint8Array)) throw new TypeError("VirGL draw packet must be a Uint8Array");
  if (!isVirglDrawPacket(packet)) throw new Error("VirGL draw packet has invalid VGD1 magic");
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const version = view.getUint32(4, true);
  const expected = length(view, version);
  if (!expected || packet.byteLength !== expected) throw new Error("VirGL draw packet has invalid length or version");
  const sequence = view.getUint32(8, true);
  const canvasWidth = view.getUint32(12, true);
  const canvasHeight = view.getUint32(16, true);
  const vertexCount = view.getUint32(20, true);
  if (sequence === 0) throw new Error("VirGL draw packet sequence must be nonzero");
  if (vertexCount !== VERTEX_COUNT) throw new Error("VirGL draw packet must contain one triangle");
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`VirGL draw dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  const clearColor = colors(view, 24, "clear");
  const drawColor = colors(view, 40, "draw");
  const paired = version === 4;
  const extended = version === 5;
  const textured = version === 3 || extended || paired;
  const vertices = textured ? texturedVertices(view) : positions(view);
  const state = version === 1 ? {} : viewportState(
    view, canvasWidth, canvasHeight, textured ? 128 : 104, textured ? 152 : 128,
  );
  const texture = textured ? textureFrame(view, packet, paired, extended) : {};
  return {
    acceleration: paired ? "webgpu-virgl-capset1-texture-multiply"
      : textured ? "webgpu-virgl-capset1-texture" : "webgpu-virgl-capset1-draw",
    canvasHeight, canvasWidth, capsetId: 1, clearColor, drawColor,
    presentationLabel: paired ? "VirGL capset 1 dual-texture triangle"
      : textured ? "VirGL capset 1 textured triangle" : "VirGL capset 1 triangle",
    protocol: paired ? "virgl-texture-multiply" : textured ? "virgl-texture" : "virgl-draw",
    sequence, version, vertexCount,
    vertices, ...state, ...texture,
  };
}

function length(view, version) {
  if (version === 1) return LEGACY_BYTES;
  if (version === 2) return STATE_BYTES;
  if (version === 3) {
    return view.byteLength < TEXTURED_BYTES ? 0 : textureLength(view, 168, 172, TEXTURED_BYTES);
  }
  if (version === 5) {
    return view.byteLength < REPEAT_TEXTURED_BYTES || !samplerConfig(view.getUint32(168, true))
      ? 0 : textureLength(view, 172, 176, REPEAT_TEXTURED_BYTES);
  }
  if (version !== 4 || view.byteLength < TEXTURED_PAIR_BYTES) return 0;
  const left = textureBytes(view, 168, 172);
  const right = textureBytes(view, 176, 180);
  return left && right ? TEXTURED_PAIR_BYTES + left + right : 0;
}

function colors(view, offset, label) {
  const color = readFloats(view, offset, 4);
  if (![...color].every((value) => Number.isFinite(value) && value >= 0 && value <= 1)) {
    throw new Error(`VirGL ${label} color must contain normalized finite values`);
  }
  return color;
}

function positions(view) {
  const vertices = readFloats(view, 56, VERTEX_COUNT * 4);
  if (!validPositions(vertices, 4)) throw new Error("VirGL triangle positions must be bounded clip-space vec4 values");
  return vertices;
}

function texturedVertices(view) {
  const vertices = readFloats(view, 56, VERTEX_COUNT * 6);
  if (!validPositions(vertices, 6)
    || !vertices.every((value, index) => index % 6 < 4 || (Number.isFinite(value) && value >= -8 && value <= 8))) {
    throw new Error("VirGL textured triangle vertices must be bounded finite values");
  }
  return vertices;
}

function validPositions(vertices, stride) {
  const valid = vertices.every((value, index) => {
    const component = index % stride;
    return component < 2 || component === 2
      ? Number.isFinite(value) && value >= -1 && value <= 1
      : component === 3 ? value === 1 : true;
  });
  const [ax, ay] = vertices;
  const [bx, by] = vertices.subarray(stride);
  const [cx, cy] = vertices.subarray(stride * 2);
  return valid && Math.abs((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) >= 0.001;
}

function viewportState(view, width, height, viewportOffset, scissorOffset) {
  const viewport = readFloats(view, viewportOffset, 6);
  const [sx, sy, sz, tx, ty, tz] = viewport;
  const valid = viewport.every(Number.isFinite) && sx > 0 && sy > 0 && sz >= 0
    && tx - sx >= 0 && tx + sx <= width && ty - sy >= 0 && ty + sy <= height
    && tz - sz >= 0 && tz + sz <= 1;
  if (!valid) throw new Error("VirGL viewport must fit its bounded target");
  const [x, y, scissorWidth, scissorHeight] = [0, 4, 8, 12]
    .map((offset) => view.getUint32(scissorOffset + offset, true));
  const empty = x === 0 && y === 0 && scissorWidth === 0 && scissorHeight === 0;
  if (empty) return { viewport };
  if (!scissorWidth || !scissorHeight || x + scissorWidth > width || y + scissorHeight > height) {
    throw new Error("VirGL scissor must fit its bounded target");
  }
  return { viewport, scissor: { x, y, width: scissorWidth, height: scissorHeight } };
}

function textureLength(view, widthOffset, heightOffset, fixedBytes) {
  const bytes = textureBytes(view, widthOffset, heightOffset);
  return bytes ? fixedBytes + bytes : 0;
}

function textureBytes(view, widthOffset, heightOffset) {
  const width = view.getUint32(widthOffset, true);
  const height = view.getUint32(heightOffset, true);
  return width && height && width <= MAX_TEXTURE_DIMENSION && height <= MAX_TEXTURE_DIMENSION
    ? width * height * 4 : 0;
}

function textureFrame(view, packet, paired, extended) {
  const offset = extended ? 172 : 168;
  const left = textureAt(view, packet, offset, offset + 4, extended ? REPEAT_TEXTURED_BYTES : paired ? TEXTURED_PAIR_BYTES : TEXTURED_BYTES);
  if (!paired) return { texture: { ...left, ...(extended ? samplerConfig(view.getUint32(168, true)) : CLAMP_NEAREST) } };
  const right = textureAt(view, packet, 176, 180, TEXTURED_PAIR_BYTES + left.pixels.byteLength);
  return { textures: [left, right] };
}

function samplerConfig(word) {
  if (word === 0x1080) return REPEAT_NEAREST;
  if (word === 0x3292) return CLAMP_LINEAR;
}

function textureAt(view, packet, widthOffset, heightOffset, pixelOffset) {
  const width = view.getUint32(widthOffset, true);
  const height = view.getUint32(heightOffset, true);
  return { width, height, pixels: packet.subarray(pixelOffset, pixelOffset + width * height * 4) };
}

function readFloats(view, offset, count) {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) values[index] = view.getFloat32(offset + index * 4, true);
  return values;
}
