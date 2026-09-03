const MAGIC = [0x56, 0x47, 0x44, 0x31]; // VGD1
const LEGACY_BYTES = 104;
const STATE_BYTES = 144;
const MAX_DIMENSION = 8192;
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
  const expected = version === 1 ? LEGACY_BYTES : version === 2 ? STATE_BYTES : 0;
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
  const vertices = positions(view);
  const state = version === 2 ? viewportState(view, canvasWidth, canvasHeight) : {};
  return {
    acceleration: "webgpu-virgl-capset1-draw",
    canvasHeight, canvasWidth, capsetId: 1, clearColor, drawColor,
    presentationLabel: "VirGL capset 1 triangle", protocol: "virgl-draw", sequence,
    version, vertexCount, vertices, ...state,
  };
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
  const valid = vertices.every((value, index) => Number.isFinite(value)
    && (index % 4 < 2 ? value >= -1 && value <= 1 : index % 4 === 2 ? value >= -1 && value <= 1 : value === 1));
  if (!valid) throw new Error("VirGL triangle positions must be bounded clip-space vec4 values");
  const [ax, ay, , , bx, by, , , cx, cy] = vertices;
  if (Math.abs((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) < 0.001) {
    throw new Error("VirGL triangle positions must not be degenerate");
  }
  return vertices;
}

function viewportState(view, width, height) {
  const viewport = readFloats(view, 104, 6);
  const [sx, sy, sz, tx, ty, tz] = viewport;
  const valid = viewport.every(Number.isFinite) && sx > 0 && sy > 0 && sz >= 0
    && tx - sx >= 0 && tx + sx <= width && ty - sy >= 0 && ty + sy <= height
    && tz - sz >= 0 && tz + sz <= 1;
  if (!valid) throw new Error("VirGL viewport must fit its bounded target");
  const [x, y, scissorWidth, scissorHeight] = [128, 132, 136, 140].map((offset) => view.getUint32(offset, true));
  const empty = x === 0 && y === 0 && scissorWidth === 0 && scissorHeight === 0;
  if (empty) return { viewport };
  if (!scissorWidth || !scissorHeight || x + scissorWidth > width || y + scissorHeight > height) {
    throw new Error("VirGL scissor must fit its bounded target");
  }
  return { viewport, scissor: { x, y, width: scissorWidth, height: scissorHeight } };
}

function readFloats(view, offset, count) {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) values[index] = view.getFloat32(offset + index * 4, true);
  return values;
}
