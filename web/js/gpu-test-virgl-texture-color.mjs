export function virglTextureColorPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.1, 0.2, 0.3, 1],
  sampler = 0x1092,
  scissor = [0, 0, canvasWidth, canvasHeight],
  sequence = 8,
  texture = [128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255],
  vertices = colorTextureTriangle(),
  viewport = [canvasWidth / 2, canvasHeight / 2, 0.5, canvasWidth / 2, canvasHeight / 2, 0.5],
} = {}) {
  const packet = new Uint8Array(228 + texture.length);
  packet.set([0x56, 0x47, 0x44, 0x31]);
  const view = new DataView(packet.buffer);
  [8, sequence, canvasWidth, canvasHeight, 3].forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  writeFloats(view, 24, clearColor); writeFloats(view, 56, vertices); writeFloats(view, 176, viewport);
  scissor.forEach((value, index) => view.setUint32(200 + index * 4, value, true));
  view.setUint32(216, sampler, true); view.setUint32(220, 2, true); view.setUint32(224, 2, true);
  packet.set(texture, 228);
  return packet;
}

function colorTextureTriangle() {
  return [
    0, 0.75, 0, 1, 1, 0, 0, 1, 0, 1,
    -0.75, -0.75, 0, 1, 0, 1, 0, 1, 0, 1,
    0.75, -0.75, 0, 1, 0, 0, 1, 1, 0, 1,
  ];
}

function writeFloats(view, offset, values) {
  values.forEach((value, index) => view.setFloat32(offset + index * 4, value, true));
}
