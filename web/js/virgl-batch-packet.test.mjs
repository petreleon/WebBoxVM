import assert from "node:assert/strict";
import test from "node:test";
import { parseGpu3dPacket } from "./gpu-3d-packet.js?v=20260903-virgl-viewport-r1";
import { virglDrawPacket } from "./gpu-test-packets.mjs?v=20260903-virgl-viewport-r1";
import { virglVertexColorPacket } from "./gpu-test-virgl-vertex-color.mjs?v=20260903-virgl-viewport-r1";
import { virglTextureColorPacket } from "./gpu-test-virgl-texture-color.mjs?v=20260903-virgl-viewport-r1";

const POSITIONS = [
  -0.9, 0.7, 0, 1, -0.9, -0.7, 0, 1, -0.1, -0.7, 0, 1,
  0.1, 0.7, 0, 1, 0.1, -0.7, 0, 1, 0.9, -0.7, 0, 1,
];
const COLOR_TRIANGLE = [
  0, 0.75, 0, 1, 1, 0, 0, 1, -0.75, -0.75, 0, 1, 0, 1, 0, 1,
  0.75, -0.75, 0, 1, 0, 0, 1, 1,
];
const TEXTURE_COLOR_TRIANGLE = COLOR_TRIANGLE.flatMap((value, index) => index % 8 === 7 ? [value, 0, 1] : [value]);
const MAX_NORMALIZED_VERTICES = 3063;

test("VirGL packet schemas preserve two independent triangles", () => {
  const solid = parseGpu3dPacket(virglDrawPacket({ vertices: POSITIONS }));
  assert.equal(solid.vertexCount, 6);
  assert.equal(solid.vertices.length, 24);
  const colors = POSITIONS.flatMap((position, index) => index % 4 === 3 ? [position, 1, 0, 1, 1] : [position]);
  const vertexColor = parseGpu3dPacket(virglVertexColorPacket({ vertices: colors }));
  assert.equal(vertexColor.vertexCount, 6);
  const textured = POSITIONS.flatMap((position, index) => index % 4 === 3 ? [position, 0.5, 0.5] : [position]);
  const textureColor = parseGpu3dPacket(virglTextureColorPacket({ vertices: textured.flatMap((value, index) => index % 6 === 3 ? [value, 1, 1, 1, 1] : [value]) }));
  assert.equal(textureColor.vertexCount, 6);
});

test("VirGL parser accepts a maximum bounded normalized strip", () => {
  const maximum = (triangle) => Array.from(
    { length: MAX_NORMALIZED_VERTICES / 3 }, () => triangle,
  ).flat();
  assert.equal(parseGpu3dPacket(virglDrawPacket({ vertices: maximum(POSITIONS.slice(0, 12)) })).vertexCount, MAX_NORMALIZED_VERTICES);
  assert.equal(parseGpu3dPacket(virglVertexColorPacket({ vertices: maximum(COLOR_TRIANGLE) })).vertexCount, MAX_NORMALIZED_VERTICES);
  assert.equal(parseGpu3dPacket(virglTextureColorPacket({ vertices: maximum(TEXTURE_COLOR_TRIANGLE) })).vertexCount, MAX_NORMALIZED_VERTICES);
});
