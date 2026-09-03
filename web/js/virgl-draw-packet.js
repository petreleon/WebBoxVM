const MAGIC = [0x56, 0x47, 0x44, 0x31]; // VGD1
const PACKET_BYTES = 104;
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
  if (packet.byteLength !== PACKET_BYTES) throw new Error("VirGL draw packet has invalid length");
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const version = view.getUint32(4, true);
  const sequence = view.getUint32(8, true);
  const canvasWidth = view.getUint32(12, true);
  const canvasHeight = view.getUint32(16, true);
  const vertexCount = view.getUint32(20, true);
  if (version !== 1) throw new Error(`Unsupported VirGL draw packet version ${version}`);
  if (sequence === 0) throw new Error("VirGL draw packet sequence must be nonzero");
  if (vertexCount !== VERTEX_COUNT) throw new Error("VirGL draw packet must contain one triangle");
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`VirGL draw dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  const clearColor = colors(view, 24, "clear");
  const drawColor = colors(view, 40, "draw");
  const vertices = positions(view);
  return {
    acceleration: "webgpu-virgl-capset1-draw",
    canvasHeight, canvasWidth, capsetId: 1, clearColor, drawColor,
    presentationLabel: "VirGL capset 1 triangle", protocol: "virgl-draw", sequence,
    version, vertexCount, vertices,
  };
}

function colors(view, offset, label) {
  const color = new Float32Array(4);
  for (let index = 0; index < color.length; index += 1) {
    const value = view.getFloat32(offset + index * 4, true);
    if (!Number.isFinite(value) || value < 0 || value > 1) {
      throw new Error(`VirGL ${label} color must contain normalized finite values`);
    }
    color[index] = value;
  }
  return color;
}

function positions(view) {
  const vertices = new Float32Array(VERTEX_COUNT * 4);
  for (let index = 0; index < vertices.length; index += 1) {
    const value = view.getFloat32(56 + index * 4, true);
    const component = index % 4;
    const valid = Number.isFinite(value)
      && (component < 2 ? value >= -1 && value <= 1 : component === 2 ? value >= 0 && value <= 1 : value === 1);
    if (!valid) throw new Error("VirGL triangle positions must be bounded clip-space vec4 values");
    vertices[index] = value;
  }
  const [ax, ay, , , bx, by, , , cx, cy] = vertices;
  if (Math.abs((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) < 0.001) {
    throw new Error("VirGL triangle positions must not be degenerate");
  }
  return vertices;
}
