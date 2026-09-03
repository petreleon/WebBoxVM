import assert from "node:assert/strict";
import test from "node:test";
import {
  extractGpu3dSequence,
  parseGpu3dPacket,
} from "./gpu-3d-packet.js?v=20260903-virgl-viewport-r1";
import {
  gpu3dPacket, virglClearPacket, virglDrawPacket, virglTexturedMultiplyPacket, virglTexturedPacket,
} from "./gpu-test-packets.mjs?v=20260903-virgl-viewport-r1";

test("WBG3 parser decodes bounded indexed geometry from an offset view", () => {
  const packet = gpu3dPacket({ sequence: 42 });
  const framed = new Uint8Array(packet.byteLength + 5);
  framed.set(packet, 3);
  const frame = parseGpu3dPacket(framed.subarray(3, 3 + packet.byteLength));
  assert.equal(frame.version, 1);
  assert.equal(frame.opcode, 1);
  assert.equal(frame.sequence, 42);
  assert.equal(frame.canvasWidth, 320);
  assert.equal(frame.canvasHeight, 240);
  assert.equal(frame.vertexCount, 3);
  assert.deepEqual([...frame.indices], [0, 1, 2]);
  assert.deepEqual([...frame.clearColor], [0.1, 0.2, 0.3, 1].map(Math.fround));
  assert.equal(frame.mvp.length, 16);
  assert.equal(frame.vertices.length, 21);
});

test("WBG3 parser enforces magic, version, opcode, dimensions, and exact length", () => {
  const valid = gpu3dPacket();
  const badMagic = valid.slice();
  badMagic[0] = 0;
  assert.throws(() => parseGpu3dPacket(badMagic), /invalid WBG3 magic/);
  assertMutation(valid, 4, 2, /version 2/);
  assertMutation(valid, 8, 2, /opcode 2/);
  assertMutation(valid, 16, 8193, /between 1 and 8192/);
  assert.throws(() => parseGpu3dPacket(valid.subarray(0, -1)), /length mismatch/);
});

test("WBG3 parser bounds geometry and validates every triangle index", () => {
  const valid = gpu3dPacket();
  assertMutation(valid, 24, 4097, /vertex count exceeds 4096/);
  assertMutation(valid, 28, 12289, /index count exceeds 12288/);
  assertMutation(valid, 28, 2, /divisible by 3/);
  const badIndex = gpu3dPacket({ indices: [0, 1, 3] });
  assert.throws(() => parseGpu3dPacket(badIndex), /outside 3 vertices/);
});

test("WBG3 parser rejects non-finite clear, MVP, and vertex values", () => {
  for (const [offset, label] of [[32, /clear color/], [48, /MVP/], [112, /vertex/]]) {
    const packet = gpu3dPacket();
    new DataView(packet.buffer).setFloat32(offset, Number.NaN, true);
    assert.throws(() => parseGpu3dPacket(packet), label);
  }
});

test("WBG3 sequence extraction trusts only a nonzero sequence behind its magic", () => {
  const packet = gpu3dPacket({ sequence: 73 });
  assert.equal(extractGpu3dSequence(packet.subarray(0, 16)), 73);
  packet[0] = 0;
  assert.equal(extractGpu3dSequence(packet), undefined);
  assert.equal(extractGpu3dSequence(new Uint8Array(15)), undefined);
  assert.equal(extractGpu3dSequence("WBG3"), undefined);
});

test("VirGL clear envelope identifies capset one and rejects invalid color values", () => {
  const packet = virglClearPacket({ sequence: 61 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-clear");
  assert.equal(frame.capsetId, 1);
  assert.equal(frame.sequence, 61);
  assert.deepEqual([...frame.clearColor], [0.25, 0.5, 0.75, 1].map(Math.fround));
  assert.equal(extractGpu3dSequence(packet), 61);
  new DataView(packet.buffer).setFloat32(20, Number.NaN, true);
  assert.throws(() => parseGpu3dPacket(packet), /normalized finite/);
});

test("VirGL draw envelope validates one bounded clip-space triangle", () => {
  const packet = virglDrawPacket({ sequence: 62 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-draw");
  assert.equal(frame.version, 2);
  assert.equal(frame.sequence, 62);
  assert.equal(frame.vertexCount, 3);
  assert.deepEqual([...frame.drawColor], [0, 1, 0, 0.25]);
  assert.deepEqual([...frame.viewport], [512, 384, 0.5, 512, 384, 0.5]);
  assert.deepEqual(frame.scissor, { x: 0, y: 0, width: 1024, height: 768 });
  assert.equal(extractGpu3dSequence(packet), 62);
  const legacy = parseGpu3dPacket(virglDrawPacket({ sequence: 63, version: 1 }));
  assert.equal(legacy.version, 1);
  assert.equal(legacy.viewport, undefined);
  assert.equal(legacy.scissor, undefined);
  const count = packet.slice();
  new DataView(count.buffer).setUint32(20, 2, true);
  assert.throws(() => parseGpu3dPacket(count), /one triangle/);
  const position = packet.slice();
  new DataView(position.buffer).setFloat32(56, 2, true);
  assert.throws(() => parseGpu3dPacket(position), /clip-space/);
  const viewport = packet.slice();
  new DataView(viewport.buffer).setFloat32(104, Number.NaN, true);
  assert.throws(() => parseGpu3dPacket(viewport), /viewport/);
  const scissor = packet.slice();
  new DataView(scissor.buffer).setUint32(136, 1025, true);
  assert.throws(() => parseGpu3dPacket(scissor), /scissor/);
});

test("VirGL textured envelope snapshots bounded BGRA sampler data", () => {
  const packet = virglTexturedPacket({ sequence: 64 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-texture");
  assert.equal(frame.version, 3);
  assert.equal(frame.sequence, 64);
  assert.equal(frame.vertices.length, 18);
  assert.deepEqual([...frame.texture.pixels], [10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255]);
  assert.equal(extractGpu3dSequence(packet), 64);
  const badSize = packet.slice();
  new DataView(badSize.buffer).setUint32(168, 65, true);
  assert.throws(() => parseGpu3dPacket(badSize), /length or version/);
  const badUv = packet.slice();
  new DataView(badUv.buffer).setFloat32(72, Number.NaN, true);
  assert.throws(() => parseGpu3dPacket(badUv), /textured triangle/);
});

test("VirGL dual-texture envelope snapshots two bounded sampler slots", () => {
  const packet = virglTexturedMultiplyPacket({ sequence: 65 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-texture-multiply");
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-texture-multiply");
  assert.equal(frame.version, 4);
  assert.equal(frame.textures.length, 2);
  assert.deepEqual([...frame.textures[0].pixels], [100, 100, 100, 255, 100, 100, 100, 255, 100, 100, 100, 255, 100, 100, 100, 255]);
  assert.deepEqual([...frame.textures[1].pixels], [128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255]);
  const badSize = packet.slice();
  new DataView(badSize.buffer).setUint32(176, 65, true);
  assert.throws(() => parseGpu3dPacket(badSize), /length or version/);
  assert.throws(() => parseGpu3dPacket(packet.subarray(0, -1)), /length or version/);
});

function assertMutation(packet, offset, value, expected) {
  const mutated = packet.slice();
  new DataView(mutated.buffer).setUint32(offset, value, true);
  assert.throws(() => parseGpu3dPacket(mutated), expected);
}
