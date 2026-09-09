import assert from "node:assert/strict";
import test from "node:test";
import { extractGpu3dSequence, parseGpu3dPacket } from "./gpu-3d-packet.js?v=20260904-virgl-readback-pool-r1";
import { virglMatrixPacket, virglMatrixTextureMultiplyPacket, virglMatrixTexturePacket, virglMatrixVertexColorPacket } from "./gpu-test-virgl-matrix.mjs?v=20260904-virgl-readback-pool-r1";

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

test("VirGL matrix texture envelope preserves raw UVs and its sampler snapshot", () => {
  const packet = virglMatrixTexturePacket({ sequence: 95 }); const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-matrix-texture"); assert.equal(frame.version, 17);
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-matrix-texture"); assert.deepEqual([...frame.vertices.slice(4, 6)], [0, 1]);
  assert.deepEqual(frame.texture, { addressMode: "clamp-to-edge", filter: "nearest", width: 1, height: 1, pixels: packet.subarray(244, 248) });
  const sampler = packet.slice(); new DataView(sampler.buffer).setUint32(232, 0, true);
  assert.throws(() => parseGpu3dPacket(sampler), /texture framing/);
  const invalidUv = packet.slice(); new DataView(invalidUv.buffer).setFloat32(136, 9, true);
  assert.throws(() => parseGpu3dPacket(invalidUv), /projection/);
});

test("VirGL matrix dual-texture envelope preserves independent sampler snapshots", () => {
  const packet = virglMatrixTextureMultiplyPacket({ sequence: 97, rightSampler: 0x1080 }); const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-matrix-texture-multiply"); assert.equal(frame.version, 18);
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-matrix-texture-multiply"); assert.equal(frame.textures.length, 2);
  assert.deepEqual(frame.textures.map(({ addressMode, filter }) => [addressMode, filter]), [["clamp-to-edge", "nearest"], ["repeat", "nearest"]]);
  const sampler = packet.slice(); new DataView(sampler.buffer).setUint32(236, 0, true);
  assert.throws(() => parseGpu3dPacket(sampler), /texture framing/);
});

test("VirGL matrix dual textures reject truncated, oversized and invalid texture frames", () => {
  const packet = virglMatrixTextureMultiplyPacket();
  for (const length of [232, 240, 255, 256, packet.length - 1]) {
    assert.throws(() => parseGpu3dPacket(packet.subarray(0, length)), /texture framing/);
  }
  const extra = new Uint8Array(packet.length + 1); extra.set(packet);
  assert.throws(() => parseGpu3dPacket(extra), /texture framing/);
  for (const offset of [240, 244, 248, 252]) {
    for (const value of [0, 65, 0xffffffff]) {
      const invalid = packet.slice(); new DataView(invalid.buffer).setUint32(offset, value, true);
      assert.throws(() => parseGpu3dPacket(invalid), /texture framing/);
    }
  }
  const maxPixels = new Uint8Array(64 * 64 * 4);
  const bounded = parseGpu3dPacket(virglMatrixTextureMultiplyPacket({
    leftTextureWidth: 64, leftTextureHeight: 64, leftPixels: maxPixels,
    rightTextureWidth: 64, rightTextureHeight: 64, rightPixels: maxPixels,
  }));
  assert.deepEqual(bounded.textures.map((texture) => texture.pixels.length), [16384, 16384]);
});
