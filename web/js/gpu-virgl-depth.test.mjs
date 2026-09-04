import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-depth-batch-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-depth-batch-r1";
import { virglDepthPacket } from "./gpu-test-virgl-depth.mjs?v=20260904-virgl-depth-batch-r1";

test("VirGL depth envelope requires its canonical depth clear and viewport state", () => {
  const packet = virglDepthPacket({ sequence: 71 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-depth");
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-depth");
  assert.equal(frame.depthClear, 1);
  assert.equal(frame.vertices[2], -0.5);
  const invalid = packet.slice();
  new DataView(invalid.buffer).setFloat32(invalid.byteLength - 4, 0.5, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth clear/);
});

test("VirGL depth renderer creates a less-write WebGPU depth attachment", async () => {
  const device = fakeDevice();
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglDepthPacket({ sequence: 72 })), { sequence: 72, success: true });
  assert.deepEqual(device.pipelines[0].descriptor.depthStencil, {
    depthCompare: "less", depthWriteEnabled: true, format: "depth24plus",
  });
  assert.deepEqual(device.textures[0].descriptor, {
    format: "depth24plus", label: "VirGL capset 1 depth",
    size: { depthOrArrayLayers: 1, height: 768, width: 1024 }, usage: 0x10,
  });
  assert.deepEqual(device.renderPasses[0].depthStencilAttachment, {
    depthClearValue: 1, depthLoadOp: "clear", depthStoreOp: "store", view: { kind: "texture-view" },
  });
  assert.deepEqual(device.draw, [3]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-depth");
});
