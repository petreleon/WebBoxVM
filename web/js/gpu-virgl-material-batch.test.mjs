import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-solid-gpu-readback-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus } from "./gpu-test-fakes.mjs?v=20260904-virgl-solid-gpu-readback-r1";
import { virglMaterialBatchPacket } from "./gpu-test-virgl-material-batch.mjs?v=20260904-virgl-solid-gpu-readback-r1";

test("VirGL material-batch parser preserves mixed draw order and depth state", () => {
  const frame = parseGpu3dPacket(virglMaterialBatchPacket());
  assert.equal(frame.protocol, "virgl-material-batch"); assert.equal(frame.depth, true);
  assert.deepEqual(frame.draws.map((draw) => draw.material), ["solid", "texture-color"]);
  assert.deepEqual(frame.draws.map((draw) => draw.depthCompare), ["less", "greater"]);
  assert.deepEqual(frame.draws.map((draw) => draw.depthWriteEnabled), [true, false]);
  assert.equal(frame.draws[1].texture.pixels.byteLength, 16);
});

test("VirGL material-batch renderer submits one depth pass for mixed materials", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu: fakeGpu([fakeAdapter(device)]) } });
  const result = await display.present3d(virglMaterialBatchPacket({ sequence: 93 }));
  assert.equal(result.sequence, 93); assert.equal(result.success, true); assert.equal(result.readback?.format, 1);
  assert.equal(device.renderPasses.length, 1); assert.deepEqual(device.draw, [3, 3]); assert.equal(device.submits, 1);
  assert.deepEqual(device.pipelines.map((pipeline) => pipeline.descriptor.depthStencil), [
    { depthCompare: "less", depthWriteEnabled: true, format: "depth24plus" },
    { depthCompare: "greater", depthWriteEnabled: false, format: "depth24plus" },
  ]);
  assert.equal(device.writes.length, 1); assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-depth-material-batch");
  assert.equal(device.textureCopies.length, 1); assert.equal(result.readback.pixels.byteLength, 1024 * 768 * 4);
});

test("VirGL material-batch parser rejects a noncanonical depth state", () => {
  const packet = virglMaterialBatchPacket(); new DataView(packet.buffer).setUint32(52, 2, true);
  assert.throws(() => parseGpu3dPacket(packet), /depth state/);
});
