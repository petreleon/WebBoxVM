import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260903-virgl-viewport-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260903-virgl-viewport-r1";
import {
  virglClearPacket, virglDrawPacket, virglTexturedMultiplyPacket, virglTexturedPacket,
} from "./gpu-test-packets.mjs?v=20260903-virgl-viewport-r1";

test("standard VirGL capset-one clear renders and acknowledges after WebGPU completion", async () => {
  let finishWork;
  const workDone = new Promise((resolve) => { finishWork = resolve; });
  const device = fakeDevice({ workDone });
  const status = fakeStatus();
  const canvas = fakeCanvas({ webgpu: true });
  const display = new GuestDisplay(canvas, status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  let settled = false;
  const completion = display.present3d(virglClearPacket({ sequence: 43 })).then((result) => {
    settled = true;
    return result;
  });
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(settled, false);
  finishWork();
  const result = await completion;
  assert.deepEqual(result, { sequence: 43, success: true });
  assert.equal(device.submits, 1);
  assert.equal(device.pipelines.length, 0);
  assert.equal(device.buffers.length, 0);
  assert.equal(device.textures.length, 0);
  assert.deepEqual(device.renderPasses[0].colorAttachments[0].clearValue, {
    r: 0.25, g: 0.5, b: 0.75, a: 1,
  });
  assert.deepEqual([canvas.width, canvas.height], [1024, 768]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-clear");
  assert.equal(status.dataset.threeDCapsetId, "1");
});

test("standard VirGL capset-one draw renders a cached WebGPU triangle", async () => {
  let finishWork;
  const workDone = new Promise((resolve) => { finishWork = resolve; });
  const device = fakeDevice({ workDone });
  const status = fakeStatus();
  const canvas = fakeCanvas({ webgpu: true });
  const display = new GuestDisplay(canvas, status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  let settled = false;
  const completion = display.present3d(virglDrawPacket({ sequence: 44 })).then((result) => {
    settled = true;
    return result;
  });
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(settled, false);
  finishWork();
  assert.deepEqual(await completion, { sequence: 44, success: true });
  assert.equal(device.submits, 1);
  assert.equal(device.pipelines.length, 1);
  assert.equal(device.buffers.length, 2);
  assert.equal(device.textures.length, 0);
  assert.deepEqual(device.draw, [3]);
  assert.deepEqual(device.pipelines[0].descriptor.fragment.targets[0].blend, {
    alpha: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "one" },
    color: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "src-alpha" },
  });
  assert.deepEqual(device.renderPasses[0].colorAttachments[0].clearValue, {
    r: Math.fround(0.1), g: Math.fround(0.2), b: Math.fround(0.3), a: 1,
  });
  assert.deepEqual(device.viewports, [[0, 0, 1024, 768, 0, 1]]);
  assert.deepEqual(device.scissors, [[0, 0, 1024, 768]]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-draw");
});

test("standard VirGL sampler texture uploads bounded BGRA data to WebGPU", async () => {
  const device = fakeDevice();
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglTexturedPacket({ sequence: 45 })), {
    sequence: 45, success: true,
  });
  assert.equal(device.pipelines.length, 1);
  assert.equal(device.buffers.length, 1);
  assert.equal(device.textures.length, 1);
  assert.deepEqual(device.draw, [3]);
  assert.deepEqual(device.samplers, [{
    addressModeU: "clamp-to-edge", addressModeV: "clamp-to-edge", magFilter: "nearest",
    minFilter: "nearest", mipmapFilter: "nearest",
  }]);
  assert.equal(device.writes[0].layout.bytesPerRow, 256);
  assert.deepEqual([...device.writes[0].data.subarray(0, 16)], [
    10, 20, 30, 255, 40, 50, 60, 255, 0, 0, 0, 0, 0, 0, 0, 0,
  ]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-texture");
});

test("standard VirGL dual sampler textures bind independently through WebGPU", async () => {
  const device = fakeDevice();
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglTexturedMultiplyPacket({
    leftSampler: 0x3292, rightSampler: 0x1080, sequence: 46,
  })), {
    sequence: 46, success: true,
  });
  assert.equal(device.pipelines.length, 1);
  assert.equal(device.buffers.length, 1);
  assert.equal(device.textures.length, 2);
  assert.deepEqual(device.samplers, [
    { addressModeU: "clamp-to-edge", addressModeV: "clamp-to-edge", magFilter: "linear", minFilter: "linear", mipmapFilter: "nearest" },
    { addressModeU: "repeat", addressModeV: "repeat", magFilter: "nearest", minFilter: "nearest", mipmapFilter: "nearest" },
  ]);
  assert.deepEqual(device.draw, [3]);
  assert.equal(device.bindGroups[0].entries.length, 4);
  assert.deepEqual([...device.writes[0].data.subarray(0, 4)], [100, 100, 100, 255]);
  assert.deepEqual([...device.writes[1].data.subarray(0, 4)], [128, 128, 128, 255]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-texture-multiply");
});
