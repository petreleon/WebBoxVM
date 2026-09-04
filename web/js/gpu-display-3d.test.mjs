import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260904-virgl-depth-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-depth-r1";
import { gpu3dPacket, gpuPacket } from "./gpu-test-packets.mjs?v=20260904-virgl-depth-r1";

test("experimental 3D has no Canvas2D fallback or compatibility claim", async () => {
  const canvas = fakeCanvas({ canvas2d: true });
  const status = fakeStatus();
  const display = new GuestDisplay(canvas, status, {
    navigator: { gpu: { requestAdapter: async () => null } },
  });
  const result = await display.present3d(gpu3dPacket());
  assert.deepEqual(result, { sequence: 7, success: false });
  assert.equal(canvas.context2d.images.length, 0);
  assert.equal(status.dataset.threeDFramesReceived, "1");
  assert.equal(status.dataset.threeDDraws, "0");
  assert.equal(status.dataset.threeDErrors, "1");
  assert.notEqual(status.dataset.threeDAcceleration, "webgpu-experimental-capset");
  assert.match(status.dataset.threeDLastError, /requires WebGPU/);
});

test("malformed WBG3 geometry retains its device sequence for a negative acknowledgment", async () => {
  const packet = gpu3dPacket({ sequence: 91 });
  new DataView(packet.buffer).setUint16(packet.byteLength - 2, 99, true);
  const display = new GuestDisplay(fakeCanvas(), fakeStatus(), { navigator: {} });
  assert.deepEqual(await display.present3d(packet), { sequence: 91, success: false });
});

test("successful WBG3 draw waits for GPU completion and reuses resources", async () => {
  let finishWork;
  const workDone = new Promise((resolve) => { finishWork = resolve; });
  const device = fakeDevice({ workDone });
  const info = { architecture: "arch", description: "Adapter", isFallbackAdapter: false, vendor: "vendor" };
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device, true, info)]) },
  });
  let settled = false;
  const completion = display.present3d(gpu3dPacket({ sequence: 9 })).then((result) => {
    settled = true;
    return result;
  });
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(settled, false);
  finishWork();
  assert.deepEqual(await completion, { sequence: 9, success: true });
  assert.equal(device.pipelines.length, 1);
  assert.equal(device.pipelineAsyncCalls, 1);
  assert.equal(device.buffers.length, 3);
  assert.equal(device.textures.length, 1);
  assert.deepEqual(device.drawIndexed, [3]);
  assert.equal(device.bufferWrites[2].data.byteLength, 8);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-experimental-capset");
  assert.equal(status.dataset.threeDCapsetId, "7");
  assert.equal(status.dataset.threeDFramesReceived, "1");
  assert.equal(status.dataset.threeDDraws, "1");
  assert.equal(status.dataset.threeDErrors, "0");
  assert.equal(status.dataset.fallbackAdapter, "false");
  assert.equal(status.dataset.adapterVendor, "vendor");
  await display.present3d(gpu3dPacket({ sequence: 10 }));
  assert.equal(device.pipelines.length, 1);
  assert.equal(device.buffers.length, 3);
  assert.equal(device.textures.length, 1);
  assert.deepEqual(device.drawIndexed, [3, 3]);
});

test("reset cancels a 3D frame waiting on asynchronous pipeline validation", async () => {
  let markPipelineStarted;
  let releasePipeline;
  const pipelineStarted = new Promise((resolve) => { markPipelineStarted = resolve; });
  const pipelineReady = new Promise((resolve) => { releasePipeline = resolve; });
  const device = fakeDevice();
  device.createRenderPipelineAsync = async (descriptor) => {
    device.pipelineAsyncCalls += 1;
    markPipelineStarted();
    await pipelineReady;
    return device.createRenderPipeline(descriptor);
  };
  const canvas = fakeCanvas({ webgpu: true });
  const status = fakeStatus();
  const display = new GuestDisplay(canvas, status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });

  const stale = display.present3d(gpu3dPacket({ sequence: 12 }));
  await pipelineStarted;
  display.reset();
  await display.present(gpuPacket());
  releasePipeline();

  assert.deepEqual(await stale, { sequence: 12, success: false });
  assert.deepEqual(device.drawIndexed, []);
  assert.deepEqual([canvas.width, canvas.height], [1, 1]);
  assert.equal(status.dataset.threeDAcceleration, "inactive");
});

test("a failed draw after success clears the current 3D acceleration claim", async () => {
  const scopeErrors = [
    null, null, null, null, null, null,
    { message: "later draw is invalid" }, null,
  ];
  const device = fakeDevice({ scopeErrors });
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });

  assert.equal((await display.present3d(gpu3dPacket({ sequence: 20 }))).success, true);
  assert.equal((await display.present3d(gpu3dPacket({ sequence: 21 }))).success, false);
  assert.equal(status.dataset.threeDDraws, "1");
  assert.equal(status.dataset.threeDErrors, "1");
  assert.equal(status.dataset.threeDAcceleration, "error");
});

for (const [kind, scopeErrors] of [
  ["validation", [null, null, null, null, { message: "invalid submitted draw" }, null]],
  ["allocation", [null, null, null, null, null, { message: "GPU memory exhausted" }]],
]) {
  test(`a captured WebGPU ${kind} error returns a failed guest completion`, async () => {
    const device = fakeDevice({ scopeErrors });
    const status = fakeStatus();
    const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
      navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
    });
    assert.deepEqual(await display.present3d(gpu3dPacket({ sequence: 31 })), {
      sequence: 31,
      success: false,
    });
    assert.equal(device.submits, 1);
    assert.equal(status.dataset.threeDDraws, "0");
    assert.equal(status.dataset.threeDErrors, "1");
    assert.match(status.dataset.threeDLastError, new RegExp(kind));
  });
}

test("a post-loss 3D frame rebuilds resources for the new device generation", async () => {
  const first = fakeDevice();
  const second = fakeDevice();
  const gpu = fakeGpu([fakeAdapter(first), fakeAdapter(second)]);
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu } });
  assert.equal((await display.present3d(gpu3dPacket({ sequence: 1 }))).success, true);
  first.lose({ message: "3D reset" });
  await Promise.resolve();
  assert.equal((await display.present3d(gpu3dPacket({ sequence: 2 }))).success, true);
  assert.equal(gpu.requestCount, 2);
  assert.equal(second.pipelines.length, 1);
  assert.equal(second.buffers.length, 3);
  assert.equal(second.textures.length, 1);
  assert.equal(status.dataset.deviceGeneration, "2");
  assert.equal(status.dataset.threeDAcceleration, "webgpu-experimental-capset");
});

test("successful WBG3 owns the shared canvas until display reset", async () => {
  const canvas = fakeCanvas({ webgpu: true });
  const device = fakeDevice();
  const display = new GuestDisplay(canvas, fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  await display.present3d(gpu3dPacket({ canvasHeight: 240, canvasWidth: 320 }));
  assert.deepEqual([canvas.width, canvas.height], [320, 240]);
  await display.present(gpuPacket({
    height: 2,
    pixels: new Array(16).fill(1),
    scanoutHeight: 2,
    scanoutWidth: 2,
    width: 2,
  }));
  assert.deepEqual([canvas.width, canvas.height], [320, 240]);
  assert.deepEqual(device.drawIndexed, [3]);
  assert.equal(device.writes.length, 0);
  display.reset();
  await display.present(gpuPacket({ height: 2, pixels: new Array(16).fill(2),
    scanoutHeight: 2, scanoutWidth: 2, width: 2 }));
  assert.deepEqual([canvas.width, canvas.height], [2, 2]);
  assert.equal(device.writes.length, 1);
});
