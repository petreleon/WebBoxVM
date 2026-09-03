export function gpuPacket({
  height = 1,
  pixels = [1, 2, 3, 255],
  scanoutHeight = 1,
  scanoutWidth = 1,
  width = 1,
  x = 0,
  y = 0,
} = {}) {
  const packet = new Uint8Array(32 + pixels.length);
  packet.set([0x57, 0x42, 0x47, 0x46]);
  const view = new DataView(packet.buffer);
  view.setUint32(4, 1, true);
  view.setUint32(8, scanoutWidth, true);
  view.setUint32(12, scanoutHeight, true);
  view.setUint32(16, x, true);
  view.setUint32(20, y, true);
  view.setUint32(24, width, true);
  view.setUint32(28, height, true);
  packet.set(pixels, 32);
  return packet;
}

export function gpu3dPacket({
  canvasHeight = 240,
  canvasWidth = 320,
  clearColor = [0.1, 0.2, 0.3, 1],
  indexCount,
  indices = [0, 1, 2],
  mvp = identityMatrix(),
  opcode = 1,
  sequence = 7,
  version = 1,
  vertexCount,
  vertices = triangleVertices(),
} = {}) {
  const actualVertexCount = vertices.length / 7;
  const actualIndexCount = indices.length;
  const packet = new Uint8Array(112 + vertices.length * 4 + indices.length * 2);
  packet.set([0x57, 0x42, 0x47, 0x33]);
  const view = new DataView(packet.buffer);
  view.setUint32(4, version, true);
  view.setUint32(8, opcode, true);
  view.setUint32(12, sequence, true);
  view.setUint32(16, canvasWidth, true);
  view.setUint32(20, canvasHeight, true);
  view.setUint32(24, vertexCount ?? actualVertexCount, true);
  view.setUint32(28, indexCount ?? actualIndexCount, true);
  writeFloats(view, 32, clearColor);
  writeFloats(view, 48, mvp);
  writeFloats(view, 112, vertices);
  const indexOffset = 112 + vertices.length * 4;
  indices.forEach((value, index) => view.setUint16(indexOffset + index * 2, value, true));
  return packet;
}

export function virglClearPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.25, 0.5, 0.75, 1],
  sequence = 7,
  version = 1,
} = {}) {
  const packet = new Uint8Array(36);
  packet.set([0x56, 0x47, 0x43, 0x31]);
  const view = new DataView(packet.buffer);
  view.setUint32(4, version, true);
  view.setUint32(8, sequence, true);
  view.setUint32(12, canvasWidth, true);
  view.setUint32(16, canvasHeight, true);
  writeFloats(view, 20, clearColor);
  return packet;
}

export function virglDrawPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.1, 0.2, 0.3, 1],
  drawColor = [0, 1, 0, 1],
  sequence = 7,
  version = 1,
  vertices = virglTriangle(),
} = {}) {
  const packet = new Uint8Array(104);
  packet.set([0x56, 0x47, 0x44, 0x31]);
  const view = new DataView(packet.buffer);
  view.setUint32(4, version, true);
  view.setUint32(8, sequence, true);
  view.setUint32(12, canvasWidth, true);
  view.setUint32(16, canvasHeight, true);
  view.setUint32(20, 3, true);
  writeFloats(view, 24, clearColor);
  writeFloats(view, 40, drawColor);
  writeFloats(view, 56, vertices);
  return packet;
}

function writeFloats(view, offset, values) {
  values.forEach((value, index) => view.setFloat32(offset + index * 4, value, true));
}

function identityMatrix() {
  return [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
}

function triangleVertices() {
  return [
    0, 0.5, 0, 1, 0, 0, 1,
    -0.5, -0.5, 0, 0, 1, 0, 1,
    0.5, -0.5, 0, 0, 0, 1, 1,
  ];
}

function virglTriangle() {
  return [0, 0.75, 0, 1, -0.75, -0.75, 0, 1, 0.75, -0.75, 0, 1];
}
