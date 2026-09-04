export function virglDepthPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.1, 0.2, 0.3, 1],
  depthCompare = 1,
  depthWriteEnabled = true,
  depthClear = 1,
  drawColor = [0, 1, 0, 0.25],
  scissor = [0, 0, canvasWidth, canvasHeight],
  sequence = 70,
  vertices = triangle(),
  viewport = [canvasWidth / 2, canvasHeight / 2, 0.5, canvasWidth / 2, canvasHeight / 2, 0.5],
} = {}) {
  const vertexCount = vertices.length / 4;
  const state = 56 + vertices.length * 4;
  const version = !depthWriteEnabled ? 11 : depthCompare === 1 ? 9 : 10;
  const packet = new Uint8Array(state + (version === 9 ? 44 : 48));
  packet.set([0x56, 0x47, 0x44, 0x31]);
  const view = new DataView(packet.buffer);
  [version, sequence, canvasWidth, canvasHeight, vertexCount].forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  writeFloats(view, 24, clearColor); writeFloats(view, 40, drawColor); writeFloats(view, 56, vertices);
  writeFloats(view, state, viewport);
  scissor.forEach((value, index) => view.setUint32(state + 24 + index * 4, value, true));
  view.setFloat32(state + 40, depthClear, true);
  if (version === 10) view.setUint32(state + 44, depthCompare, true);
  if (version === 11) view.setUint32(state + 44, 1 | (depthCompare << 2), true);
  return packet;
}

export function virglDepthTexturePacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.1, 0.2, 0.3, 1],
  depthClear = 1,
  depthCompare = 1,
  depthWriteEnabled = true,
  sampler = 0x1092,
  scissor = [0, 0, canvasWidth, canvasHeight],
  sequence = 90,
  texture = [10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255],
  vertices = texturedTriangle(),
  viewport = [canvasWidth / 2, canvasHeight / 2, 0.5, canvasWidth / 2, canvasHeight / 2, 0.5],
} = {}) {
  const vertexCount = vertices.length / 6;
  const state = 56 + vertices.length * 4;
  const packet = new Uint8Array(state + 60 + texture.length);
  packet.set([0x56, 0x47, 0x44, 0x31]);
  const view = new DataView(packet.buffer);
  [13, sequence, canvasWidth, canvasHeight, vertexCount].forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  writeFloats(view, 24, clearColor); writeFloats(view, 56, vertices); writeFloats(view, state, viewport);
  scissor.forEach((value, index) => view.setUint32(state + 24 + index * 4, value, true));
  [sampler, 2, 2].forEach((value, index) => view.setUint32(state + 40 + index * 4, value, true));
  packet.set(texture, state + 52);
  view.setFloat32(state + 52 + texture.length, depthClear, true);
  view.setUint32(state + 56 + texture.length, 1 | (depthWriteEnabled ? 2 : 0) | (depthCompare << 2), true);
  return packet;
}

export function virglDepthTextureColorPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.1, 0.2, 0.3, 1],
  depthClear = 1,
  depthCompare = 1,
  depthWriteEnabled = true,
  sampler = 0x1092,
  scissor = [0, 0, canvasWidth, canvasHeight],
  sequence = 93,
  texture = [128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255],
  vertices = colorTextureTriangle(),
  viewport = [canvasWidth / 2, canvasHeight / 2, 0.5, canvasWidth / 2, canvasHeight / 2, 0.5],
} = {}) {
  const vertexCount = vertices.length / 10;
  const state = 56 + vertices.length * 4;
  const packet = new Uint8Array(state + 60 + texture.length);
  packet.set([0x56, 0x47, 0x44, 0x31]);
  const view = new DataView(packet.buffer);
  [14, sequence, canvasWidth, canvasHeight, vertexCount].forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  writeFloats(view, 24, clearColor); writeFloats(view, 56, vertices); writeFloats(view, state, viewport);
  scissor.forEach((value, index) => view.setUint32(state + 24 + index * 4, value, true));
  [sampler, 2, 2].forEach((value, index) => view.setUint32(state + 40 + index * 4, value, true));
  packet.set(texture, state + 52); view.setFloat32(state + 52 + texture.length, depthClear, true);
  view.setUint32(state + 56 + texture.length, 1 | (depthWriteEnabled ? 2 : 0) | (depthCompare << 2), true);
  return packet;
}

function writeFloats(view, offset, values) {
  values.forEach((value, index) => view.setFloat32(offset + index * 4, value, true));
}

function triangle() {
  return [0, 0.75, -0.5, 1, -0.75, -0.75, -0.5, 1, 0.75, -0.75, -0.5, 1];
}

function texturedTriangle() {
  return [0, 0.75, -0.5, 1, 0, 1, -0.75, -0.75, -0.5, 1, 0, 1, 0.75, -0.75, -0.5, 1, 0, 1];
}

function colorTextureTriangle() {
  return [
    0, 0.75, -0.5, 1, 1, 0, 0, 1, 0, 1,
    -0.75, -0.75, -0.5, 1, 0, 1, 0, 1, 0, 1,
    0.75, -0.75, -0.5, 1, 0, 0, 1, 1, 0, 1,
  ];
}
