import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-depth-texture-color-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-depth-texture-color-r1";
import { virglTextureColorPacket } from "./gpu-test-virgl-texture-color.mjs?v=20260904-virgl-depth-texture-color-r1";

test("VirGL texture-color envelope snapshots one bounded texture and generic color varyings", () => {
  const packet = virglTextureColorPacket({ sequence: 68, sampler: 0x3292 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-texture-color");
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-texture-color");
  assert.equal(frame.version, 8);
  assert.equal(frame.vertices.length, 30);
  assert.equal(frame.texture.filter, "linear");
  assert.deepEqual([...frame.texture.pixels], new Array(4).fill([128, 128, 128, 255]).flat());
  const reserved = packet.slice();
  new DataView(reserved.buffer).setFloat32(40, 0.5, true);
  assert.throws(() => parseGpu3dPacket(reserved), /reserved color/);
  const uv = packet.slice();
  new DataView(uv.buffer).setFloat32(92, Number.NaN, true);
  assert.throws(() => parseGpu3dPacket(uv), /bounded and normalized/);
  const sampler = packet.slice();
  new DataView(sampler.buffer).setUint32(216, 0x1234, true);
  assert.throws(() => parseGpu3dPacket(sampler), /sampler or texture framing/);
  assert.throws(() => parseGpu3dPacket(packet.subarray(0, -1)), /invalid sampler or texture framing/);
});

test("VirGL texture-color renderer uses one sampler and a 40-byte interpolation layout", async () => {
  const device = fakeDevice();
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglTextureColorPacket({ sequence: 69 })), {
    sequence: 69, success: true,
  });
  assert.equal(device.pipelines.length, 1);
  assert.equal(device.buffers.length, 1);
  assert.equal(device.textures.length, 1);
  assert.equal(device.bindGroups[0].entries.length, 2);
  assert.deepEqual(device.samplers, [{
    addressModeU: "clamp-to-edge", addressModeV: "clamp-to-edge", magFilter: "nearest",
    minFilter: "nearest", mipmapFilter: "nearest",
  }]);
  assert.deepEqual(device.pipelines[0].descriptor.vertex.buffers[0], { arrayStride: 40, attributes: [
    { format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 },
    { format: "float32x2", offset: 32, shaderLocation: 2 },
  ] });
  assert.deepEqual(device.draw, [3]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-texture-color");
});
