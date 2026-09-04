import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-depth-batch-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-depth-batch-r1";
import { virglSolidBatchPacket } from "./gpu-test-virgl-solid-batch.mjs?v=20260904-virgl-depth-batch-r1";

test("VirGL depth-batch envelope requires a clear-one ordered depth stream", () => {
  const packet = virglSolidBatchPacket({ draws: depthDraws(), sequence: 76, version: 2 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-depth-batch");
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-depth-batch");
  assert.equal(frame.depthClear, 1);
  assert.equal(frame.draws[0].vertices[2], -0.5);
  const invalid = packet.slice(); new DataView(invalid.buffer).setFloat32(44, 0, true);
  assert.throws(() => parseGpu3dPacket(invalid), /depth clear/);
});

test("VirGL depth-batch renderer clears one less-write depth attachment", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglSolidBatchPacket({ draws: depthDraws(), sequence: 77, version: 2 })), { sequence: 77, success: true });
  assert.deepEqual(device.pipelines[0].descriptor.depthStencil, {
    depthCompare: "less", depthWriteEnabled: true, format: "depth24plus",
  });
  assert.deepEqual(device.textures[0].descriptor, {
    format: "depth24plus", label: "VirGL capset 1 depth batch",
    size: { depthOrArrayLayers: 1, height: 768, width: 1024 }, usage: 0x10,
  });
  assert.equal(device.renderPasses.length, 1);
  assert.equal(device.renderPasses[0].depthStencilAttachment.depthClearValue, 1);
  assert.deepEqual(device.draw, [3, 3]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-depth-batch");
});

function depthDraws() {
  const viewport = [512, 384, 0.5, 512, 384, 0.5];
  const triangle = (z) => [0, 0.75, z, 1, -0.75, -0.75, z, 1, 0.75, -0.75, z, 1];
  return [
    { drawColor: [1, 0, 0, 0.5], scissor: [0, 0, 1024, 768], vertices: triangle(-0.5), viewport },
    { drawColor: [0, 1, 0, 0.5], scissor: [0, 0, 1024, 768], vertices: triangle(0.5), viewport },
  ];
}
