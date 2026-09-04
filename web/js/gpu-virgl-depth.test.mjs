import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-depth-texture-color-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-depth-texture-color-r1";
import { virglDepthPacket, virglDepthTexturePacket } from "./gpu-test-virgl-depth.mjs?v=20260904-virgl-depth-texture-color-r1";
import { virglVertexColorPacket } from "./gpu-test-virgl-vertex-color.mjs?v=20260904-virgl-depth-texture-color-r1";

test("VirGL depth envelope requires its canonical depth clear and viewport state", () => {
  const packet = virglDepthPacket({ sequence: 71 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-depth");
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-depth");
  assert.equal(frame.depthClear, 1);
  assert.equal(frame.depthCompare, "less");
  assert.equal(frame.vertices[2], -0.5);
  const invalid = packet.slice();
  new DataView(invalid.buffer).setFloat32(invalid.byteLength - 4, 0.5, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth clear/);
});

test("VirGL depth envelope carries standard equal comparison state", () => {
  const packet = virglDepthPacket({ depthCompare: 2, sequence: 73, vertices: equalTriangle() });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.version, 10);
  assert.equal(frame.depthCompare, "equal");
  const invalid = packet.slice(); new DataView(invalid.buffer).setUint32(invalid.byteLength - 4, 8, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth compare/);
});

test("VirGL depth envelope carries a canonical read-only DSA state", () => {
  const packet = virglDepthPacket({ depthCompare: 4, depthWriteEnabled: false, sequence: 75 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.version, 11); assert.equal(frame.depthCompare, "greater");
  assert.equal(frame.depthWriteEnabled, false);
  const invalid = packet.slice(); new DataView(invalid.buffer).setUint32(invalid.byteLength - 4, 16, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth state/);
});

test("VirGL depth envelope maps every accepted standard comparison", () => {
  const names = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];
  names.forEach((name, depthCompare) => {
    assert.equal(parseGpu3dPacket(virglDepthPacket({ depthCompare, sequence: 80 + depthCompare })).depthCompare, name);
  });
});

test("VirGL depth renderer creates a less-write WebGPU depth attachment", async () => {
  const device = fakeDevice();
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglDepthPacket({ sequence: 72 })), { sequence: 72, success: true });
  assert.deepEqual(device.pipelines[0].descriptor.depthStencil, {
    depthCompare: "less", depthWriteEnabled: true, format: "depth24plus",
  });
  assert.deepEqual(device.textures[0].descriptor, {
    format: "depth24plus", label: "VirGL capset 1 depth",
    size: { depthOrArrayLayers: 1, height: 768, width: 1024 }, usage: 0x10,
  });
  assert.deepEqual(device.renderPasses[0].depthStencilAttachment, {
    depthClearValue: 1, depthLoadOp: "clear", depthStoreOp: "store", view: { kind: "texture-view" },
  });
  assert.deepEqual(device.draw, [3]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-depth");
});

test("VirGL depth renderer configures the requested standard comparison", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglDepthPacket({ depthCompare: 2, sequence: 74, vertices: equalTriangle() })), { sequence: 74, success: true });
  assert.equal(device.pipelines[0].descriptor.depthStencil.depthCompare, "equal");
});

test("VirGL depth renderer preserves the requested depth write mask", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  const packet = virglDepthPacket({ depthCompare: 4, depthWriteEnabled: false, sequence: 76 });
  assert.deepEqual(await display.present3d(packet), { sequence: 76, success: true });
  assert.deepEqual(device.pipelines[0].descriptor.depthStencil, {
    depthCompare: "greater", depthWriteEnabled: false, format: "depth24plus",
  });
});

test("VirGL depth vertex-color envelope carries a canonical DSA state", () => {
  const packet = virglVertexColorPacket({ depthState: { compare: 4, write: false }, sequence: 87 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-depth-vertex-color");
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-depth-vertex-color");
  assert.equal(frame.version, 12); assert.equal(frame.depthClear, 1);
  assert.equal(frame.depthCompare, "greater"); assert.equal(frame.depthWriteEnabled, false);
  const invalid = packet.slice(); new DataView(invalid.buffer).setUint32(invalid.byteLength - 4, 16, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth state/);
});

test("VirGL depth vertex-color renderer uses its requested WebGPU depth state", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  const packet = virglVertexColorPacket({ depthState: { compare: 4, write: false }, sequence: 88 });
  assert.deepEqual(await display.present3d(packet), { sequence: 88, success: true });
  assert.deepEqual(device.pipelines[0].descriptor.depthStencil, {
    depthCompare: "greater", depthWriteEnabled: false, format: "depth24plus",
  });
  assert.deepEqual(device.textures[0].descriptor, {
    format: "depth24plus", label: "VirGL vertex-color depth",
    size: { depthOrArrayLayers: 1, height: 768, width: 1024 }, usage: 0x10,
  });
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-depth-vertex-color");
});

test("VirGL depth-texture envelope keeps a canonical sampler and DSA", () => {
  const packet = virglDepthTexturePacket({ depthCompare: 4, depthWriteEnabled: false, sampler: 0x3292, sequence: 91 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-depth-texture"); assert.equal(frame.version, 13);
  assert.equal(frame.depthCompare, "greater"); assert.equal(frame.depthWriteEnabled, false);
  assert.deepEqual([frame.texture.addressMode, frame.texture.filter], ["clamp-to-edge", "linear"]);
  const invalid = packet.slice(); new DataView(invalid.buffer).setUint32(invalid.byteLength - 4, 16, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth state/);
});

test("VirGL depth-texture renderer attaches requested WebGPU depth and sampler state", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu: fakeGpu([fakeAdapter(device)]) } });
  const packet = virglDepthTexturePacket({ depthCompare: 4, depthWriteEnabled: false, sampler: 0x3292, sequence: 92 });
  assert.deepEqual(await display.present3d(packet), { sequence: 92, success: true });
  assert.deepEqual(device.pipelines[0].descriptor.depthStencil, { depthCompare: "greater", depthWriteEnabled: false, format: "depth24plus" });
  assert.deepEqual(device.samplers[0], { addressModeU: "clamp-to-edge", addressModeV: "clamp-to-edge", magFilter: "linear", minFilter: "linear", mipmapFilter: "nearest" });
  assert.deepEqual(device.textures.find((texture) => texture.descriptor.label === "VirGL depth-texture depth").descriptor,
    { format: "depth24plus", label: "VirGL depth-texture depth", size: { depthOrArrayLayers: 1, height: 768, width: 1024 }, usage: 0x10 });
  assert.deepEqual(device.renderPasses[0].depthStencilAttachment,
    { depthClearValue: 1, depthLoadOp: "clear", depthStoreOp: "store", view: { kind: "texture-view" } });
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-depth-texture");
});

function equalTriangle() {
  return [0, 0.75, 1, 1, -0.75, -0.75, 1, 1, 0.75, -0.75, 1, 1];
}
