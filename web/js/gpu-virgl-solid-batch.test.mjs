import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglSolidBatchPacket } from "./gpu-test-virgl-solid-batch.mjs?v=20260904-virgl-readback-pool-r1";

test("VirGL solid-batch packet preserves bounded ordered draw records", () => {
  const packet = virglSolidBatchPacket({ sequence: 74 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-solid-batch");
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-solid-batch");
  assert.equal(frame.draws.length, 2);
  assert.equal(frame.draws[0].vertexCount, 3);
  const resident = parseGpu3dPacket(virglSolidBatchPacket({ sequence: 76, version: 6 }));
  assert.equal(resident.residentCandidate, true);
  assert.equal(resident.depthClear, 0);
  const singleton = parseGpu3dPacket(virglSolidBatchPacket({ drawCount: 1, sequence: 78, version: 6 }));
  assert.equal(singleton.draws.length, 1);
  assert.throws(() => parseGpu3dPacket(virglSolidBatchPacket({ drawCount: 1, sequence: 79 })), /VGB1 framing/);
  const replacement = parseGpu3dPacket(virglSolidBatchPacket({
    residentPreviousProducer: 76, sequence: 77, version: 7,
  }));
  assert.equal(replacement.residentPreviousProducer, 76);
  assert.equal(replacement.residentCandidate, true);
  assert.equal(replacement.depthClear, 0);
  assert.throws(() => parseGpu3dPacket(virglSolidBatchPacket({
    residentPreviousProducer: 0, sequence: 77, version: 7,
  })), /replacement producer/);
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
  const result = await display.present3d(virglSolidBatchPacket({ sequence: 75 }));
  assert.equal(result.sequence, 75); assert.equal(result.success, true);
  assert.equal(result.readback?.format, 1); assert.equal(result.readback?.pixels.byteLength, 1024 * 768 * 4);
  assert.deepEqual(device.draw, [3, 3]);
  assert.equal(device.bufferWrites[0].data.byteLength, 192);
  assert.deepEqual(device.pipelines[0].descriptor.vertex.buffers[0], {
    arrayStride: 32,
    attributes: [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 }],
  });
  assert.equal(device.renderPasses.length, 1);
  assert.equal(device.textureCopies.length, 1);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-solid-batch");
});

test("VirGL singleton solid batches retain and rekey one resident output", async () => {
  const device = fakeDevice();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglSolidBatchPacket({
    drawCount: 1, sequence: 80, version: 6,
  })), { resident: true, sequence: 80, success: true });
  const outputs = device.textures.filter((texture) => texture.descriptor.label?.startsWith("VirGL resident output"));
  assert.equal(outputs.length, 1); assert.deepEqual(device.draw, [3]);
  assert.deepEqual(await display.present3d(virglSolidBatchPacket({
    drawCount: 1, residentPreviousProducer: 80, sequence: 81, version: 7,
  })), { resident: true, sequence: 81, success: true });
  assert.equal(outputs.length, 1); assert.deepEqual(device.draw, [3, 3]);
});
