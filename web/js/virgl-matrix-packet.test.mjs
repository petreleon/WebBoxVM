import assert from "node:assert/strict";
import test from "node:test";
import { extractGpu3dSequence, parseGpu3dPacket } from "./gpu-3d-packet.js?v=20260904-virgl-readback-pool-r1";
import { virglMatrixPacket, virglMatrixVertexColorPacket } from "./gpu-test-virgl-matrix.mjs?v=20260904-virgl-readback-pool-r1";

test("VirGL matrix envelope retains raw vertices and validates projected bounds", () => {
  const packet = virglMatrixPacket({ sequence: 91 }); const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-draw"); assert.equal(frame.version, 15);
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-matrix"); assert.equal(frame.matrix.length, 16);
  assert.deepEqual([...frame.vertices.slice(0, 4)], [0, 0.75, 0, 1]);
  assert.equal(extractGpu3dSequence(packet), 91);
  const nonfinite = packet.slice(); new DataView(nonfinite.buffer).setFloat32(56, Number.NaN, true);
  assert.throws(() => parseGpu3dPacket(nonfinite), /matrix rows/);
  const invalid = packet.slice(); new DataView(invalid.buffer).setFloat32(116, 0, true);
  assert.throws(() => parseGpu3dPacket(invalid), /projection/);
});

test("VirGL matrix vertex-color envelope preserves generic attributes", () => {
  const packet = virglMatrixVertexColorPacket({ sequence: 93 }); const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-matrix-vertex-color"); assert.equal(frame.version, 16);
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-matrix-vertex-color"); assert.equal(frame.vertices.length, 24);
  assert.deepEqual([...frame.vertices.slice(4, 8)], [1, 0, 0, 1]);
  const reserved = packet.slice(); new DataView(reserved.buffer).setFloat32(40, 1, true);
  assert.throws(() => parseGpu3dPacket(reserved), /reserved color/);
  const invalidColor = packet.slice(); new DataView(invalidColor.buffer).setFloat32(136, 2, true);
  assert.throws(() => parseGpu3dPacket(invalidColor), /projection/);
});
