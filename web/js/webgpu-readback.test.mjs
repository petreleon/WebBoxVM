import assert from "node:assert/strict";
import test from "node:test";
import { fakeDevice } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import {
  READBACK_FORMAT_BGRA8,
  READBACK_FORMAT_RGBA8,
  canvasConfiguration,
  submitTextureReadback,
} from "./webgpu-readback.js?v=20260904-virgl-readback-pool-r1";

test("canvas readback configuration keeps rendering and copy usage", () => {
  const config = canvasConfiguration({ kind: "device" }, "bgra8unorm");
  assert.equal(config.usage, 0x13);
  assert.equal(config.alphaMode, "opaque");
});

test("texture readback waits for mapping and strips WebGPU row padding", async () => {
  let finish;
  const workDone = new Promise((resolve) => { finish = resolve; });
  const bytes = new Uint8Array(512);
  bytes.set([1, 2, 3, 4, 5, 6, 7, 8]); bytes.set([9, 10, 11, 12, 13, 14, 15, 16], 256);
  const device = fakeDevice({ readbackBytes: bytes, workDone });
  const encoder = device.createCommandEncoder();
  let settled = false;
  const result = submitTextureReadback(device, encoder, { kind: "target" }, 2, 2, "bgra8unorm")
    .then((value) => { settled = true; return value; });
  assert.equal(device.submits, 1); assert.equal(device.textureCopies.length, 1); assert.equal(settled, false);
  finish();
  assert.deepEqual(await result, {
    format: READBACK_FORMAT_BGRA8,
    pixels: new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
  });
});

test("RGBA canvas output retains its format tag while unsupported output falls back", async () => {
  const device = fakeDevice();
  const rgba = await submitTextureReadback(device, device.createCommandEncoder(), {}, 1, 1, "rgba8unorm");
  assert.equal(rgba.format, READBACK_FORMAT_RGBA8);
  const fallback = await submitTextureReadback(device, device.createCommandEncoder(), {}, 1, 1, "rgba16float");
  assert.equal(fallback, undefined); assert.equal(device.textureCopies.length, 1);
});

test("settled texture readback reuses an unmapped staging buffer", async () => {
  const device = fakeDevice();
  await submitTextureReadback(device, device.createCommandEncoder(), {}, 2, 2, "bgra8unorm");
  await submitTextureReadback(device, device.createCommandEncoder(), {}, 2, 2, "bgra8unorm");
  assert.equal(device.buffers.length, 1);
  assert.equal(device.buffers[0].destroyed, undefined);
  assert.equal(device.buffers[0].unmaps, 2);
});
