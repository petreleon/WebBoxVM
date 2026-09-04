import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-depth-vertex-color-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-depth-vertex-color-r1";
import { virglSolidBatchPacket } from "./gpu-test-virgl-solid-batch.mjs?v=20260904-virgl-depth-vertex-color-r1";

test("VirGL depth-batch envelope requires a clear-one ordered depth stream", () => {
  const packet = virglSolidBatchPacket({ draws: depthDraws(), sequence: 76, version: 2 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-depth-batch");
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-depth-batch");
  assert.equal(frame.depthClear, 1);
  assert.equal(frame.depthCompare, "less");
  assert.equal(frame.draws[0].vertices[2], -0.5);
  const invalid = packet.slice(); new DataView(invalid.buffer).setFloat32(44, 0, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth clear/);
});

test("VirGL depth-batch envelope carries one shared standard comparison", () => {
  const packet = virglSolidBatchPacket({ depthCompare: 2, draws: equalDraws(), sequence: 78, version: 3 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.version, 3); assert.equal(frame.depthCompare, "equal");
  const invalid = packet.slice(); new DataView(invalid.buffer).setUint32(24, 8, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth comparison/);
});

test("VirGL depth-batch envelope carries ordered per-draw comparisons", () => {
  const packet = virglSolidBatchPacket({ draws: mixedDraws(), sequence: 80, version: 4 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.version, 4); assert.equal(frame.depthCompare, undefined);
  assert.deepEqual(frame.draws.map((draw) => draw.depthCompare), ["less", "greater"]);
  const flagged = packet.slice(); new DataView(flagged.buffer).setUint32(24, 1, true);
  assert.throws(() => parseGpu3dPacket(flagged), /framing/);
  const invalid = packet.slice(); new DataView(invalid.buffer).setUint32(52, 8, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth comparison/);
});

test("VirGL depth-batch envelope carries ordered DSA write masks", () => {
  const packet = virglSolidBatchPacket({ draws: mixedWriteDraws(), sequence: 82, version: 5 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.version, 5); assert.deepEqual(frame.draws.map((draw) => draw.depthCompare), ["less", "greater"]);
  assert.deepEqual(frame.draws.map((draw) => draw.depthWriteEnabled), [true, false]);
  const invalid = packet.slice(); new DataView(invalid.buffer).setUint32(52, 2, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth state/);
});

test("VirGL depth-batch renderer clears one less-write depth attachment", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglSolidBatchPacket({ draws: depthDraws(), sequence: 77, version: 2 })), { sequence: 77, success: true });
  assert.deepEqual(device.pipelines[0].descriptor.depthStencil, {
    depthCompare: "less", depthWriteEnabled: true, format: "depth24plus",
  });
  assert.deepEqual(device.textures[0].descriptor, {
    format: "depth24plus", label: "VirGL capset 1 depth batch",
    size: { depthOrArrayLayers: 1, height: 768, width: 1024 }, usage: 0x10,
  });
  assert.equal(device.renderPasses.length, 1);
  assert.equal(device.renderPasses[0].depthStencilAttachment.depthClearValue, 1);
  assert.deepEqual(device.draw, [3, 3]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-depth-batch");
});

test("VirGL depth-batch renderer configures the shared standard comparison", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  const packet = virglSolidBatchPacket({ depthCompare: 2, draws: equalDraws(), sequence: 79, version: 3 });
  assert.deepEqual(await display.present3d(packet), { sequence: 79, success: true });
  assert.equal(device.pipelines[0].descriptor.depthStencil.depthCompare, "equal");
});

test("VirGL depth-batch renderer preserves ordered per-draw comparisons", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  const packet = virglSolidBatchPacket({ draws: mixedDraws(), sequence: 81, version: 4 });
  assert.deepEqual(await display.present3d(packet), { sequence: 81, success: true });
  assert.deepEqual(device.pipelines.map((pipeline) => pipeline.descriptor.depthStencil.depthCompare), ["less", "greater"]);
  assert.deepEqual(device.pipelineBinds.map((pipeline) => pipeline.descriptor.depthStencil.depthCompare), ["less", "greater"]);
  assert.equal(device.renderPasses.length, 1); assert.deepEqual(device.draw, [3, 3]);
});

test("VirGL depth-batch renderer preserves ordered depth write masks", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  const packet = virglSolidBatchPacket({ draws: mixedWriteDraws(), sequence: 83, version: 5 });
  assert.deepEqual(await display.present3d(packet), { sequence: 83, success: true });
  assert.deepEqual(device.pipelines.map((pipeline) => pipeline.descriptor.depthStencil), [
    { depthCompare: "less", depthWriteEnabled: true, format: "depth24plus" },
    { depthCompare: "greater", depthWriteEnabled: false, format: "depth24plus" },
  ]);
  assert.deepEqual(device.pipelineBinds.map((pipeline) => pipeline.descriptor.depthStencil.depthWriteEnabled), [true, false]);
});

function depthDraws() {
  const viewport = [512, 384, 0.5, 512, 384, 0.5];
  const triangle = (z) => [0, 0.75, z, 1, -0.75, -0.75, z, 1, 0.75, -0.75, z, 1];
  return [
    { drawColor: [1, 0, 0, 0.5], scissor: [0, 0, 1024, 768], vertices: triangle(-0.5), viewport },
    { drawColor: [0, 1, 0, 0.5], scissor: [0, 0, 1024, 768], vertices: triangle(0.5), viewport },
  ];
}

function equalDraws() {
  const viewport = [512, 384, 0.5, 512, 384, 0.5];
  const triangle = [0, 0.75, 1, 1, -0.75, -0.75, 1, 1, 0.75, -0.75, 1, 1];
  return [
    { drawColor: [1, 0, 0, 0.5], scissor: [0, 0, 1024, 768], vertices: triangle, viewport },
    { drawColor: [0, 0, 1, 0.5], scissor: [0, 0, 1024, 768], vertices: triangle, viewport },
  ];
}

function mixedDraws() {
  return depthDraws().map((draw, index) => ({ ...draw, depthCompare: [1, 4][index] }));
}

function mixedWriteDraws() {
  return mixedDraws().map((draw, index) => ({ ...draw, depthWriteEnabled: index === 0 }));
}
