export function virglMaterialBatchPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0, 0, 0, 1],
  depth = true,
  draws = defaultDraws(canvasWidth, canvasHeight),
  sequence = 91,
} = {}) {
  const body = draws.reduce((total, draw) => total + 52 + materialBytes(draw) + draw.vertices.length * 4, 0);
  const packet = new Uint8Array(48 + body); const view = new DataView(packet.buffer);
  packet.set([0x56, 0x47, 0x4d, 0x31]);
  [1, sequence, canvasWidth, canvasHeight, draws.length, depth ? 1 : 0].forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  floats(view, 28, clearColor); view.setFloat32(44, depth ? 1 : 0, true);
  let offset = 48;
  for (const draw of draws) offset = writeDraw(view, packet, offset, draw, depth);
  return packet;
}

function writeDraw(view, packet, offset, draw, depth) {
  const kind = kindOf(draw.material); const count = draw.vertices.length / stride(draw.material);
  view.setUint32(offset, kind, true); view.setUint32(offset + 4, depth ? dsa(draw) : 0, true); view.setUint32(offset + 8, count, true);
  floats(view, offset + 12, draw.viewport); draw.scissor.forEach((value, index) => view.setUint32(offset + 36 + index * 4, value, true));
  let cursor = offset + 52;
  if (draw.material === "solid") { floats(view, cursor, draw.drawColor); cursor += 16; }
  if (draw.material === "texture" || draw.material === "texture-color") cursor = writeTexture(view, packet, cursor, draw.texture);
  if (draw.material === "texture-pair") for (const texture of draw.textures) cursor = writeTexture(view, packet, cursor, texture);
  floats(view, cursor, draw.vertices); return cursor + draw.vertices.length * 4;
}

function writeTexture(view, packet, offset, texture) {
  view.setUint32(offset, samplerWord(texture), true); view.setUint32(offset + 4, texture.width, true); view.setUint32(offset + 8, texture.height, true);
  packet.set(texture.pixels, offset + 12); return offset + 12 + texture.pixels.length;
}

function materialBytes(draw) {
  if (draw.material === "solid") return 16;
  if (draw.material === "vertex-color") return 0;
  if (draw.material === "texture-pair") return draw.textures.reduce((total, texture) => total + 12 + texture.pixels.length, 0);
  return 12 + draw.texture.pixels.length;
}

function dsa(draw) { return 1 | (draw.depthWriteEnabled === false ? 0 : 2) | ((draw.depthCompare ?? 1) << 2); }
function kindOf(material) { return ["solid", "vertex-color", "texture", "texture-pair", "texture-color"].indexOf(material) + 1; }
function stride(material) { return ({ solid: 4, "vertex-color": 8, texture: 6, "texture-pair": 6, "texture-color": 10 })[material]; }
function samplerWord(texture) {
  if (texture.addressMode === "repeat") return 0x1080;
  return texture.filter === "linear" ? 0x3292 : 0x1092;
}
function floats(view, offset, values) { values.forEach((value, index) => view.setFloat32(offset + index * 4, value, true)); }

function defaultDraws(width, height) {
  const viewport = [width / 2, height / 2, .5, width / 2, height / 2, .5];
  const near = [0, .75, -.5, 1, -.75, -.75, -.5, 1, .75, -.75, -.5, 1];
  const far = [0, .75, .5, 1, 1, 0, 0, 1, 0, 1, -.75, -.75, .5, 1, 0, 1, 0, 1, 0, 1, .75, -.75, .5, 1, 0, 0, 1, 1, 0, 1];
  const texture = { addressMode: "clamp-to-edge", filter: "nearest", height: 2, pixels: new Uint8Array(16).fill(128), width: 2 };
  return [
    { drawColor: [1, 0, 0, .5], material: "solid", scissor: [0, 0, width, height], vertices: near, viewport },
    { depthCompare: 4, depthWriteEnabled: false, material: "texture-color", scissor: [0, 0, width, height], texture, vertices: far, viewport },
  ];
}
