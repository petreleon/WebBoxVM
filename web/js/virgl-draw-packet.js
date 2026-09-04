import { parseVirglVertexColorPacket } from "./virgl-vertex-color-packet.js?v=20260904-virgl-readback-pool-r1";
import { parseVirglTextureColorPacket } from "./virgl-texture-color-packet.js?v=20260904-virgl-readback-pool-r1";
import { parseVirglDepthPacket } from "./virgl-depth-packet.js?v=20260904-virgl-readback-pool-r1";
import { parseVirglDepthTexturePacket } from "./virgl-depth-texture-packet.js?v=20260904-virgl-readback-pool-r1"; import { parseVirglDepthTextureColorPacket } from "./virgl-depth-texture-color-packet.js?v=20260904-virgl-readback-pool-r1";

const MAGIC = [0x56, 0x47, 0x44, 0x31]; // VGD1
const MAX_DIMENSION = 8192;
const MAX_TEXTURE_DIMENSION = 64;
const MAX_VERTEX_COUNT = 3063;
const VERTICES_PER_TRIANGLE = 3;
const CLAMP_NEAREST = { addressMode: "clamp-to-edge", filter: "nearest" };
const REPEAT_NEAREST = { addressMode: "repeat", filter: "nearest" };
const CLAMP_LINEAR = { addressMode: "clamp-to-edge", filter: "linear" };

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
  if (!isVirglDrawPacket(packet) || packet.byteLength < 24) throw new Error("VirGL draw packet has invalid VGD1 magic");
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const version = view.getUint32(4, true);
  if (version === 7 || version === 12) return parseVirglVertexColorPacket(packet);
  if (version === 8) return parseVirglTextureColorPacket(packet);
  if (version === 9 || version === 10 || version === 11) return parseVirglDepthPacket(packet);
  if (version === 13) return parseVirglDepthTexturePacket(packet);
  if (version === 14) return parseVirglDepthTextureColorPacket(packet);
  const sequence = view.getUint32(8, true);
  const canvasWidth = view.getUint32(12, true);
  const canvasHeight = view.getUint32(16, true);
  const vertexCount = view.getUint32(20, true);
  if (!sequence) throw new Error("VirGL draw packet sequence must be nonzero");
  if (!validCount(vertexCount)) throw new Error(`VirGL draw vertex count must be 3..${MAX_VERTEX_COUNT} and divisible by 3`);
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`VirGL draw dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  const layout = packetLayout(view, version, vertexCount);
  if (!layout) throw new Error("VirGL draw packet has invalid length or version");
  const clearColor = colors(view, 24, "clear");
  const drawColor = colors(view, 40, "draw");
  const textured = layout.stride === 6;
  const vertices = readFloats(view, 56, vertexCount * layout.stride);
  if (!validPositions(vertices, layout.stride) || textured && !validUvs(vertices)) {
    throw new Error(textured ? "VirGL textured vertices are invalid" : "VirGL triangle positions are invalid");
  }
  const state = version === 1 ? {} : viewportState(view, canvasWidth, canvasHeight, layout.state);
  const texture = textured ? textureFrame(view, packet, version, layout) : {};
  const paired = version === 4 || version === 6;
  return {
    acceleration: paired ? "webgpu-virgl-capset1-texture-multiply"
      : textured ? "webgpu-virgl-capset1-texture" : "webgpu-virgl-capset1-draw",
    canvasHeight, canvasWidth, capsetId: 1, clearColor, drawColor, sequence, version, vertexCount, vertices,
    presentationLabel: paired ? "VirGL capset 1 dual-texture triangles"
      : textured ? "VirGL capset 1 textured triangles" : "VirGL capset 1 triangles",
    protocol: paired ? "virgl-texture-multiply" : textured ? "virgl-texture" : "virgl-draw",
    ...state, ...texture,
  };
}

function packetLayout(view, version, vertexCount) {
  const stride = [1, 2].includes(version) ? 4 : [3, 4, 5, 6].includes(version) ? 6 : 0;
  const state = 56 + vertexCount * stride * 4;
  if (!stride) return undefined;
  if (version === 1) return view.byteLength === state ? { stride, state } : undefined;
  if (version === 2) return view.byteLength === state + 40 ? { stride, state } : undefined;
  if (version === 3) return textureLayout(view, stride, state, [state + 40, state + 44], state + 48);
  if (version === 5 && view.byteLength >= state + 44 && [0x1080, 0x3292].includes(view.getUint32(state + 40, true))) {
    return textureLayout(view, stride, state, [state + 44, state + 48], state + 52);
  }
  if (version === 4) return pairLayout(view, stride, state, state + 56);
  if (version === 6 && view.byteLength >= state + 48
    && [state + 40, state + 44].every((offset) => samplerConfig(view.getUint32(offset, true)))) {
    return pairLayout(view, stride, state, state + 64);
  }
}

function textureLayout(view, stride, state, [width, height], pixels) {
  const bytes = textureBytes(view, width, height);
  return bytes && view.byteLength === pixels + bytes ? { stride, state, pixels } : undefined;
}

function pairLayout(view, stride, state, pixels) {
  const sampled = pixels === state + 64;
  const left = textureBytes(view, state + (sampled ? 48 : 40), state + (sampled ? 52 : 44));
  const right = textureBytes(view, state + (sampled ? 56 : 48), state + (sampled ? 60 : 52));
  return left && right && view.byteLength === pixels + left + right ? { stride, state, pixels } : undefined;
}

function textureFrame(view, packet, version, { state, pixels }) {
  if (version === 6) {
    const left = textureAt(view, packet, state + 48, state + 52, pixels);
    const right = textureAt(view, packet, state + 56, state + 60, pixels + left.pixels.byteLength);
    return { textures: [{ ...left, ...samplerConfig(view.getUint32(state + 40, true)) }, { ...right, ...samplerConfig(view.getUint32(state + 44, true)) }] };
  }
  const extended = version === 5;
  const left = textureAt(view, packet, state + (extended ? 44 : 40), state + (extended ? 48 : 44), pixels);
  if (version !== 4) return { texture: { ...left, ...(extended ? samplerConfig(view.getUint32(state + 40, true)) : CLAMP_NEAREST) } };
  const right = textureAt(view, packet, state + 48, state + 52, pixels + left.pixels.byteLength);
  return { textures: [left, right] };
}

function validCount(count) {
  return count >= VERTICES_PER_TRIANGLE && count <= MAX_VERTEX_COUNT && count % VERTICES_PER_TRIANGLE === 0;
}

function colors(view, offset, label) {
  const color = readFloats(view, offset, 4);
  if (![...color].every((value) => Number.isFinite(value) && value >= 0 && value <= 1)) {
    throw new Error(`VirGL ${label} color must contain normalized finite values`);
  }
  return color;
}

function validPositions(vertices, stride) {
  const valid = vertices.every((value, index) => {
    const component = index % stride;
    return component < 3 ? Number.isFinite(value) && value >= -1 && value <= 1 : component === 3 ? value === 1 : true;
  });
  for (let base = 0; valid && base < vertices.length; base += stride * VERTICES_PER_TRIANGLE) {
    const [ax, ay] = vertices.subarray(base, base + 2);
    const [bx, by] = vertices.subarray(base + stride, base + stride + 2);
    const [cx, cy] = vertices.subarray(base + 2 * stride, base + 2 * stride + 2);
    if (Math.abs((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) < 0.001) return false;
  }
  return valid;
}

function validUvs(vertices) {
  return vertices.every((value, index) => index % 6 < 4 || Number.isFinite(value) && value >= -8 && value <= 8);
}

function viewportState(view, width, height, state) {
  const viewport = readFloats(view, state, 6);
  const [sx, sy, sz, tx, ty, tz] = viewport;
  const valid = viewport.every(Number.isFinite) && sx > 0 && sy > 0 && sz >= 0
    && tx - sx >= 0 && tx + sx <= width && ty - sy >= 0 && ty + sy <= height && tz - sz >= 0 && tz + sz <= 1;
  if (!valid) throw new Error("VirGL viewport must fit its bounded target");
  const [x, y, scissorWidth, scissorHeight] = [0, 4, 8, 12].map((offset) => view.getUint32(state + 24 + offset, true));
  if (x === 0 && y === 0 && scissorWidth === 0 && scissorHeight === 0) return { viewport };
  if (!scissorWidth || !scissorHeight || x + scissorWidth > width || y + scissorHeight > height) {
    throw new Error("VirGL scissor must fit its bounded target");
  }
  return { viewport, scissor: { x, y, width: scissorWidth, height: scissorHeight } };
}

function textureBytes(view, widthOffset, heightOffset) {
  if (view.byteLength < heightOffset + 4) return 0;
  const width = view.getUint32(widthOffset, true);
  const height = view.getUint32(heightOffset, true);
  return width && height && width <= MAX_TEXTURE_DIMENSION && height <= MAX_TEXTURE_DIMENSION ? width * height * 4 : 0;
}

function samplerConfig(word) {
  if (word === 0x1092) return CLAMP_NEAREST;
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
