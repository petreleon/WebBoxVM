import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-solid-batch-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-solid-batch-r1";
import { virglSolidBatchPacket } from "./gpu-test-virgl-solid-batch.mjs?v=20260904-virgl-solid-batch-r1";

test("VirGL solid-batch packet preserves bounded ordered draw records", () => {
  const packet = virglSolidBatchPacket({ sequence: 74 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-solid-batch");
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-solid-batch");
  assert.equal(frame.draws.length, 2);
  assert.equal(frame.draws[0].vertexCount, 3);
  const invalid = packet.slice();
  new DataView(invalid.buffer).setUint32(24, 1, true);
  assert.throws(() => parseGpu3dPacket(invalid), /VGB1 framing/);
});

test("VirGL solid-batch renderer submits both draws in one source-over pass", async () => {
  const device = fakeDevice();
  const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglSolidBatchPacket({ sequence: 75 })), {
    sequence: 75, success: true,
  });
  assert.deepEqual(device.draw, [3, 3]);
  assert.equal(device.bufferWrites[0].data.byteLength, 192);
  assert.deepEqual(device.pipelines[0].descriptor.vertex.buffers[0], {
    arrayStride: 32,
    attributes: [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 }],
  });
  assert.equal(device.renderPasses.length, 1);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-solid-batch");
});
