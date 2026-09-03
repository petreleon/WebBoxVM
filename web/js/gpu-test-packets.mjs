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
  drawColor = [0, 1, 0, 0.25],
  scissor,
  sequence = 7,
  version = 2,
  vertices = virglTriangle(),
  viewport,
} = {}) {
  const packet = new Uint8Array(version === 1 ? 104 : 144);
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
  if (version === 2) {
    writeFloats(view, 104, viewport ?? [canvasWidth / 2, canvasHeight / 2, 0.5, canvasWidth / 2, canvasHeight / 2, 0.5]);
    const values = scissor ?? [0, 0, canvasWidth, canvasHeight];
    values.forEach((value, index) => view.setUint32(128 + index * 4, value, true));
  }
  return packet;
}

export function virglTexturedPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.1, 0.2, 0.3, 1],
  scissor = [0, 0, canvasWidth, canvasHeight],
  sequence = 7,
  texture = [10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255],
  vertices = texturedVirglTriangle(),
  viewport = [canvasWidth / 2, canvasHeight / 2, 0.5, canvasWidth / 2, canvasHeight / 2, 0.5],
} = {}) {
  const packet = new Uint8Array(176 + texture.length);
  packet.set([0x56, 0x47, 0x44, 0x31]);
  const view = new DataView(packet.buffer);
  view.setUint32(4, 3, true);
  view.setUint32(8, sequence, true);
  view.setUint32(12, canvasWidth, true);
  view.setUint32(16, canvasHeight, true);
  view.setUint32(20, 3, true);
  writeFloats(view, 24, clearColor);
  writeFloats(view, 40, [0, 0, 0, 0]);
  writeFloats(view, 56, vertices);
  writeFloats(view, 128, viewport);
  scissor.forEach((value, index) => view.setUint32(152 + index * 4, value, true));
  view.setUint32(168, 2, true);
  view.setUint32(172, 2, true);
  packet.set(texture, 176);
  return packet;
}

export function virglTexturedMultiplyPacket({
  leftSampler = 0x1092,
  rightSampler = 0x1092,
  textureLeft = [100, 100, 100, 255, 100, 100, 100, 255, 100, 100, 100, 255, 100, 100, 100, 255],
  textureRight = [128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255],
  ...options
} = {}) {
  const sampled = leftSampler !== 0x1092 || rightSampler !== 0x1092;
  const left = virglTexturedPacket({ ...options, texture: textureLeft });
  const offset = sampled ? 192 : 184;
  const packet = new Uint8Array(offset + textureLeft.length + textureRight.length);
  packet.set(left.subarray(0, 168));
  const view = new DataView(packet.buffer);
  view.setUint32(4, sampled ? 6 : 4, true);
  if (sampled) [leftSampler, rightSampler, 2, 2, 2, 2]
    .forEach((value, index) => view.setUint32(168 + index * 4, value, true));
  else [168, 172, 176, 180].forEach((at) => view.setUint32(at, 2, true));
  packet.set(textureLeft, offset);
  packet.set(textureRight, offset + textureLeft.length);
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

function texturedVirglTriangle() {
  return [0, 0.75, 0, 1, 0, 1, -0.75, -0.75, 0, 1, 0, 1, 0.75, -0.75, 0, 1, 0, 1];
}
