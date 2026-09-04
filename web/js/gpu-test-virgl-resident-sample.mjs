export function virglResidentSamplePacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0, 0, 0, 1],
  material = "texture",
  producerSequence = 90,
  sampler = 0x1092,
  sequence = 91,
  sourceHeight = 65,
  sourceWidth = 65,
  version = 12,
  writeMask = 0xF,
} = {}) {
  const vertices = material === "texture-color" ? colorVertices() : textureVertices();
  const bytes = 48 + 52 + 16 + vertices.length * 4; const packet = new Uint8Array(bytes); const view = new DataView(packet.buffer);
  packet.set([0x56, 0x47, 0x4d, 0x31]);
  [version, sequence, canvasWidth, canvasHeight, 1, version === 12 ? 2 : writeMask].forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  floats(view, 28, clearColor); view.setFloat32(44, 0, true);
  view.setUint32(48, material === "texture-color" ? 5 : 3, true); view.setUint32(56, vertices.length / (material === "texture-color" ? 10 : 6), true);
  floats(view, 60, [canvasWidth / 2, canvasHeight / 2, .5, canvasWidth / 2, canvasHeight / 2, .5]);
  [0, 0, canvasWidth, canvasHeight].forEach((value, index) => view.setUint32(84 + index * 4, value, true));
  [sampler, sourceWidth, sourceHeight, producerSequence].forEach((value, index) => view.setUint32(100 + index * 4, value, true));
  floats(view, 116, vertices); return packet;
}

function textureVertices() {
  return [0, .75, 0, 1, 0, 1, -.75, -.75, 0, 1, 0, 0, .75, -.75, 0, 1, 1, 0];
}

function colorVertices() {
  return [0, .75, 0, 1, 1, 1, 1, 1, 0, 1, -.75, -.75, 0, 1, 1, 1, 1, 1, 0, 0, .75, -.75, 0, 1, 1, 1, 1, 1, 1, 0];
}

function floats(view, offset, values) { values.forEach((value, index) => view.setFloat32(offset + index * 4, value, true)); }
