import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglDepthTextureColorPacket } from "./gpu-test-virgl-depth.mjs?v=20260904-virgl-readback-pool-r1";

test("VirGL depth texture-color envelope keeps fixed attributes and DSA", () => {
  const packet = virglDepthTextureColorPacket({ depthCompare: 4, depthWriteEnabled: false, sampler: 0x3292, sequence: 94 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-depth-texture-color"); assert.equal(frame.version, 14);
  assert.equal(frame.depthCompare, "greater"); assert.equal(frame.depthWriteEnabled, false);
  assert.deepEqual([frame.texture.addressMode, frame.texture.filter], ["clamp-to-edge", "linear"]);
  const invalid = packet.slice(); new DataView(invalid.buffer).setUint32(invalid.byteLength - 4, 16, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth state/);
});

test("VirGL depth texture-color renderer preserves WebGPU depth and modulation layout", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu: fakeGpu([fakeAdapter(device)]) } });
  const packet = virglDepthTextureColorPacket({ depthCompare: 4, depthWriteEnabled: false, sampler: 0x3292, sequence: 95 });
  assert.deepEqual(await display.present3d(packet), { sequence: 95, success: true });
  assert.deepEqual(device.pipelines[0].descriptor.depthStencil, { depthCompare: "greater", depthWriteEnabled: false, format: "depth24plus" });
  assert.deepEqual(device.pipelines[0].descriptor.vertex.buffers[0], { arrayStride: 40, attributes: [
    { format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 },
    { format: "float32x2", offset: 32, shaderLocation: 2 },
  ] });
  assert.deepEqual(device.samplers[0], { addressModeU: "clamp-to-edge", addressModeV: "clamp-to-edge", magFilter: "linear", minFilter: "linear", mipmapFilter: "nearest" });
  assert.deepEqual(device.textures.find((texture) => texture.descriptor.label === "VirGL depth-texture-color depth").descriptor,
    { format: "depth24plus", label: "VirGL depth-texture-color depth", size: { depthOrArrayLayers: 1, height: 768, width: 1024 }, usage: 0x10 });
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-depth-texture-color");
});
