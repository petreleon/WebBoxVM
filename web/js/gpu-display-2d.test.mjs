import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260904-virgl-depth-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-depth-r1";
import { gpuPacket } from "./gpu-test-packets.mjs?v=20260904-virgl-depth-r1";

test("missing WebGPU falls back to Canvas2D and converts BGRA to RGBA", async () => {
  const canvas = fakeCanvas({ canvas2d: true });
  const status = fakeStatus();
  const display = new GuestDisplay(canvas, status, {
    navigator: { gpu: { requestAdapter: async () => null } },
  });
  await display.present(gpuPacket({ pixels: [1, 2, 3, 255] }));
  assert.equal(status.dataset.backend, "canvas2d");
  assert.equal(status.dataset.adapter, "none");
  assert.equal(status.dataset.framesReceived, "1");
  assert.equal(status.dataset.uploads, "1");
  assert.match(status.dataset.lastError, /No WebGPU adapter/);
  assert.deepEqual([...canvas.context2d.images[0].image.data], [3, 2, 1, 255]);
  display.reset();
  assert.equal(status.dataset.framesReceived, "0");
  assert.equal(status.dataset.uploads, "0");
  assert.equal(canvas.context2d.clears.length, 1);
});

test("WebGPU keeps one texture and uploads padded dirty rectangles", async () => {
  const device = fakeDevice();
  const gpu = fakeGpu([fakeAdapter(device, true)]);
  const canvas = fakeCanvas({ webgpu: true });
  const status = fakeStatus();
  const display = new GuestDisplay(canvas, status, { navigator: { gpu } });
  await display.present(gpuPacket({
    height: 2,
    pixels: new Array(32).fill(10),
    scanoutHeight: 2,
    scanoutWidth: 4,
    width: 4,
  }));
  await display.present(gpuPacket({
    height: 2,
    pixels: [1, 2, 3, 255, 5, 6, 7, 255],
    scanoutHeight: 2,
    scanoutWidth: 4,
    width: 1,
    x: 1,
  }));
  const dirty = device.writes[1];
  assert.equal(device.textures.length, 1);
  assert.deepEqual(dirty.destination.origin, { x: 1, y: 0, z: 0 });
  assert.equal(dirty.layout.bytesPerRow, 256);
  assert.deepEqual([...dirty.data.subarray(0, 4)], [1, 2, 3, 255]);
  assert.deepEqual([...dirty.data.subarray(256, 260)], [5, 6, 7, 255]);
  assert.equal(status.dataset.threeDAcceleration, "inactive");
  assert.equal(status.dataset.fallbackAdapter, "true");
  assert.equal(status.dataset.uploads, "2");
  const backend = await display.acquireWebGpuBackend();
  assert.equal(backend.device, device);
  assert.equal(backend.scanoutTexture, device.textures[0]);
});

test("modern adapter info drives idle diagnostics and overrides the legacy flag", async () => {
  const device = fakeDevice();
  const info = {
    architecture: "test-arch",
    description: "Test Adapter",
    isFallbackAdapter: true,
    vendor: "test-vendor",
  };
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device, false, info)]) },
  });
  await display.acquireWebGpuBackend();
  assert.equal(status.dataset.fallbackAdapter, "true");
  assert.equal(status.dataset.adapterVendor, "test-vendor");
  assert.equal(status.dataset.adapterArchitecture, "test-arch");
  assert.equal(status.dataset.adapterDescription, "Test Adapter");
});

test("destroy cancels pending WebGPU initialization and clears live diagnostics", async () => {
  let markDeviceRequested;
  let releaseDevice;
  const deviceRequested = new Promise((resolve) => { markDeviceRequested = resolve; });
  const deviceReady = new Promise((resolve) => { releaseDevice = resolve; });
  const device = fakeDevice();
  device.destroy = () => { device.destroyed = true; };
  const adapter = fakeAdapter(device);
  adapter.requestDevice = async () => {
    markDeviceRequested();
    return deviceReady;
  };
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([adapter]) },
  });

  const pending = display.acquireWebGpuBackend();
  await deviceRequested;
  display.destroy();
  releaseDevice(device);

  assert.equal(await pending, undefined);
  assert.equal(device.destroyed, true);
  assert.equal(status.dataset.backend, "none");
  assert.equal(status.dataset.adapter, "unknown");
  assert.equal(status.textContent, "Waiting for guest display");
});

test("a configuration failure destroys its device and permits a clean retry", async () => {
  const first = fakeDevice();
  const second = fakeDevice();
  first.destroy = () => { first.destroyed = true; };
  const canvas = fakeCanvas({ webgpu: true });
  const configure = canvas.contextGpu.configure.bind(canvas.contextGpu);
  let attempts = 0;
  canvas.contextGpu.configure = (options) => {
    attempts += 1;
    if (attempts === 1) throw new Error("configuration rejected");
    configure(options);
  };
  const status = fakeStatus();
  const gpu = fakeGpu([fakeAdapter(first), fakeAdapter(second)]);
  const display = new GuestDisplay(canvas, status, { navigator: { gpu } });

  assert.equal(await display.acquireWebGpuBackend(), undefined);
  assert.equal(first.destroyed, true);
  assert.equal(status.dataset.backend, "unavailable");
  assert.match(status.dataset.lastError, /configuration rejected/);
  assert.equal((await display.acquireWebGpuBackend()).device, second);
  assert.equal(gpu.requestCount, 2);
  assert.equal(status.dataset.backend, "webgpu");
});

test("device loss recreates state and restores the shadow framebuffer", async () => {
  const first = fakeDevice();
  const second = fakeDevice();
  const gpu = fakeGpu([fakeAdapter(first), fakeAdapter(second)]);
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu } });
  await display.present(gpuPacket({ pixels: [11, 12, 13, 255] }));
  first.lose({ message: "test reset" });
  await Promise.resolve();
  await display.whenIdle();
  assert.equal(gpu.requestCount, 2);
  assert.equal(second.writes.length, 1);
  assert.deepEqual([...second.writes[0].data.subarray(0, 4)], [11, 12, 13, 255]);
  assert.equal(status.dataset.deviceGeneration, "2");
  assert.equal(status.dataset.uploads, "2");
  assert.match(status.dataset.lastError, /device lost: test reset/i);
});
