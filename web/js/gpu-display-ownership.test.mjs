import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260903-virgl-viewport-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260903-virgl-viewport-r1";
import { gpu3dPacket, gpuPacket } from "./gpu-test-packets.mjs?v=20260903-virgl-viewport-r1";

test("a pending WBG3 claim cancels a scanout waiting for WebGPU initialization", async () => {
  let markRequested;
  let releaseDevice;
  const requested = new Promise((resolve) => { markRequested = resolve; });
  const ready = new Promise((resolve) => { releaseDevice = resolve; });
  const device = fakeDevice();
  const adapter = fakeAdapter(device);
  adapter.requestDevice = async () => {
    markRequested();
    return ready;
  };
  const canvas = fakeCanvas({ webgpu: true });
  const status = fakeStatus();
  const display = new GuestDisplay(canvas, status, { navigator: { gpu: fakeGpu([adapter]) } });

  const scanout = display.present(gpuPacket());
  await requested;
  const draw = display.present3d(gpu3dPacket({ canvasHeight: 240, canvasWidth: 320 }));
  releaseDevice(device);

  assert.equal(await scanout, false);
  assert.deepEqual(await draw, { sequence: 7, success: true });
  await display.whenIdle();
  assert.deepEqual([canvas.width, canvas.height], [320, 240]);
  assert.deepEqual(device.drawIndexed, [3]);
  assert.equal(device.writes.length, 0);
  assert.equal(status.dataset.framesReceived, "1");
  assert.equal(status.dataset.uploads, "0");
});

test("completed WBG3 remains visible while later WBGF frames update only the shadow", async () => {
  const device = fakeDevice();
  const canvas = fakeCanvas({ webgpu: true });
  const status = fakeStatus();
  const display = new GuestDisplay(canvas, status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  await display.present(gpuPacket({ pixels: [1, 2, 3, 255] }));
  await display.present3d(gpu3dPacket({ canvasHeight: 240, canvasWidth: 320 }));

  const ignored = await Promise.all([
    display.present(gpuPacket({ pixels: [4, 5, 6, 255] })),
    display.present(gpuPacket({ pixels: [7, 8, 9, 255] })),
    display.present(gpuPacket({ pixels: [10, 11, 12, 255] })),
  ]);
  await display.whenIdle();

  assert.deepEqual(ignored, [false, false, false]);
  assert.deepEqual([canvas.width, canvas.height], [320, 240]);
  assert.equal(device.writes.length, 1);
  assert.equal(device.submits, 2);
  assert.equal(status.dataset.framesReceived, "4");
  assert.equal(status.dataset.uploads, "1");
});

test("a failed WBG3 draw restores the latest full 2D shadow", async () => {
  const scopeErrors = [null, null, null, null, { message: "rejected draw" }, null];
  const device = fakeDevice({ scopeErrors });
  const canvas = fakeCanvas({ webgpu: true });
  const status = fakeStatus();
  const display = new GuestDisplay(canvas, status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });

  const draw = display.present3d(gpu3dPacket({ canvasHeight: 240, canvasWidth: 320 }));
  await display.present(gpuPacket({ pixels: [9, 8, 7, 255] }));
  assert.deepEqual(await draw, { sequence: 7, success: false });
  await display.whenIdle();

  assert.deepEqual([canvas.width, canvas.height], [1, 1]);
  assert.equal(device.writes.length, 1);
  assert.deepEqual([...device.writes[0].data.subarray(0, 4)], [9, 8, 7, 255]);
  assert.equal(status.dataset.uploads, "1");
  assert.equal(status.dataset.threeDAcceleration, "error");
});

test("device loss drops WBG3 ownership and restores the newest 2D shadow", async () => {
  const first = fakeDevice();
  const second = fakeDevice();
  const canvas = fakeCanvas({ webgpu: true });
  const status = fakeStatus();
  const display = new GuestDisplay(canvas, status, {
    navigator: { gpu: fakeGpu([fakeAdapter(first), fakeAdapter(second)]) },
  });

  await display.present(gpuPacket({ pixels: [1, 2, 3, 255] }));
  await display.present3d(gpu3dPacket({ canvasHeight: 240, canvasWidth: 320 }));
  await display.present(gpuPacket({ pixels: [11, 12, 13, 255] }));
  assert.equal(first.writes.length, 1);
  first.lose({ message: "ownership test" });
  await Promise.resolve();
  await display.whenIdle();

  assert.deepEqual([canvas.width, canvas.height], [1, 1]);
  assert.equal(second.writes.length, 1);
  assert.deepEqual([...second.writes[0].data.subarray(0, 4)], [11, 12, 13, 255]);
  assert.equal(status.dataset.deviceGeneration, "2");
  assert.equal(status.dataset.uploads, "2");
});
