import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260903-virgl-viewport-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260903-virgl-viewport-r1";
import { virglClearPacket, virglDrawPacket } from "./gpu-test-packets.mjs?v=20260903-virgl-viewport-r1";

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
