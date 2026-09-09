import assert from "node:assert/strict";
import test from "node:test";
import { fakeCanvas, fakeDevice } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglMatrixTextureMultiplyPacket } from "./gpu-test-virgl-matrix.mjs?v=20260904-virgl-readback-pool-r1";
import { parseGpu3dPacket } from "./gpu-3d-packet.js?v=20260904-virgl-readback-pool-r1";
import { VirglMatrixTextureMultiplyRenderer } from "./webgpu-virgl-matrix-texture-multiply.js?v=20260904-virgl-readback-pool-r1";

const current = () => true;
const frame = (options) => parseGpu3dPacket(virglMatrixTextureMultiplyPacket(options));
const renderer = () => new VirglMatrixTextureMultiplyRenderer({ configure() {} });
const backend = (device, deviceGeneration = 1) => ({
  device, deviceGeneration, format: "bgra8unorm", canvasContext: fakeCanvas().contextGpu,
});
const tick = () => new Promise((resolve) => setImmediate(resolve));

test("matrix dual textures wait for GPU completion and reject invalidated work", async () => {
  let finish;
  const device = fakeDevice({ workDone: new Promise((resolve) => { finish = resolve; }) });
  const draw = renderer(); let settled = false;
  const pending = draw.render(backend(device), frame(), current).then((result) => { settled = true; return result; });
  await tick(); assert.equal(device.submits, 1); assert.equal(settled, false);
  draw.invalidate(); finish();
  assert.equal(await pending, false);
  assert.ok([...device.textures, ...device.buffers].every((resource) => resource.destroyed));
});

test("late matrix pipeline creation cannot replace a new device pipeline", async () => {
  const oldDevice = fakeDevice(); const newDevice = fakeDevice(); let finish;
  oldDevice.createRenderPipelineAsync = (descriptor) => new Promise((resolve) => {
    finish = () => resolve(oldDevice.createRenderPipeline(descriptor));
  });
  const draw = renderer(); const pending = draw.render(backend(oldDevice), frame(), current);
  await tick();
  assert.equal(await draw.render(backend(newDevice, 2), frame(), current), true);
  finish(); assert.equal(await pending, false);
  assert.equal(await draw.render(backend(newDevice, 2), frame(), current), true);
  assert.equal(oldDevice.submits, 0);
  assert.ok(newDevice.pipelineBinds.every((pipeline) => pipeline === newDevice.pipelines[0]));
  draw.invalidate();
});

test("failed second matrix texture allocation releases the first texture", async () => {
  const device = fakeDevice(); const createTexture = device.createTexture.bind(device);
  device.createTexture = (descriptor) => {
    if (device.textures.length === 1) throw new Error("second texture allocation failed");
    return createTexture(descriptor);
  };
  await assert.rejects(renderer().render(backend(device), frame(), current), /second texture allocation failed/);
  assert.equal(device.submits, 0); assert.equal(device.textures.length, 1);
  assert.ok([...device.textures, ...device.buffers].every((resource) => resource.destroyed));
});

test("matrix dual textures reuse uploads and update independent sampler and pixel state", async () => {
  const device = fakeDevice(); const draw = renderer(); const target = backend(device);
  await draw.render(target, frame(), current);
  await draw.render(target, frame({ sequence: 8 }), current);
  assert.equal(device.pipelines.length, 1); assert.equal(device.textures.length, 2);
  assert.equal(device.writes.length, 2); assert.equal(device.bindGroups.length, 1);
  await draw.render(target, frame({ rightSampler: 0x3292, rightPixels: new Uint8Array([1, 2, 3, 255]) }), current);
  assert.equal(device.writes.length, 3); assert.equal(device.bindGroups.length, 2);
  assert.equal(device.samplers.at(-1).minFilter, "linear");
  assert.equal(device.writes.at(-1).destination.texture, device.textures[1]);
  assert.equal(device.writes.at(-1).layout.bytesPerRow, 256);
  await draw.render(target, frame({ leftTextureWidth: 2, leftPixels: new Uint8Array(8) }), current);
  assert.ok(device.textures.slice(0, 2).every((texture) => texture.destroyed));
  assert.equal(device.writes.length, 5);
  draw.invalidate();
});

test("matrix dual textures reject queue failures and unavailable completion tracking", async () => {
  const device = fakeDevice(); const draw = renderer();
  device.queue.onSubmittedWorkDone = () => Promise.reject(new Error("GPU completion failed"));
  await assert.rejects(draw.render(backend(device), frame(), current), /GPU completion failed/);
  assert.ok([...device.textures, ...device.buffers].every((resource) => resource.destroyed));
  delete device.queue.onSubmittedWorkDone;
  await assert.rejects(draw.render(backend(device), frame(), current), /completion tracking/);
});
