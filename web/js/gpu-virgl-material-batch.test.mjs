import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglClearPacket } from "./gpu-test-packets.mjs?v=20260904-virgl-readback-pool-r1";
import { virglMaterialBatchPacket } from "./gpu-test-virgl-material-batch.mjs?v=20260904-virgl-readback-pool-r1";
import { virglResidentReadbackPacket } from "./gpu-test-virgl-resident-readback.mjs?v=20260904-virgl-readback-pool-r1";

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

test("VirGL material batches reuse byte-identical texture snapshots", async () => {
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu: fakeGpu([fakeAdapter(device)]) } });
  await display.present3d(virglMaterialBatchPacket({ sequence: 94 }));
  await display.present3d(virglMaterialBatchPacket({ sequence: 95 }));
  assert.equal(device.writes.length, 1); assert.equal(device.bufferWrites.length, 1); assert.equal(device.bindGroups.length, 1);
});

test("VirGL material batches elide unchanged pass state", async () => {
  const device = fakeDevice(); const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  const draw = textureColorDraw();
  await display.present3d(virglMaterialBatchPacket({ depth: false, draws: [draw, { ...draw, vertices: [...draw.vertices] }], sequence: 96 }));
  assert.equal(device.pipelineBinds.length, 1); assert.equal(device.bindGroupBinds.length, 1);
  assert.equal(device.viewports.length, 1); assert.equal(device.scissors.length, 1); assert.deepEqual(device.draw, [3, 3]);
});

test("VirGL material batches rebind after a pipeline transition", async () => {
  const device = fakeDevice(); const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  const texture = textureColorDraw();
  await display.present3d(virglMaterialBatchPacket({
    depth: false, draws: [texture, solidDraw(), { ...texture, vertices: [...texture.vertices] }], sequence: 97,
  }));
  assert.equal(device.pipelineBinds.length, 3); assert.equal(device.bindGroupBinds.length, 2); assert.deepEqual(device.draw, [3, 3, 3]);
});

test("VirGL material-batch parser rejects a noncanonical depth state", () => {
  const packet = virglMaterialBatchPacket(); new DataView(packet.buffer).setUint32(52, 2, true);
  assert.throws(() => parseGpu3dPacket(packet), /depth state/);
});

test("VirGL opaque material batches use replace pipelines", async () => {
  const frame = parseGpu3dPacket(virglMaterialBatchPacket({ sequence: 99, version: 4 }));
  assert.equal(frame.blend, "replace"); assert.equal(frame.residentCandidate, false);
  const device = fakeDevice(); const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.equal((await display.present3d(virglMaterialBatchPacket({ sequence: 99, version: 4 }))).success, true);
  assert.equal(device.pipelines.every((pipeline) => !("blend" in pipeline.descriptor.fragment.targets[0])), true);
});

test("VirGL opaque material depth batches use replace depth pipelines", async () => {
  const packet = virglMaterialBatchPacket({ sequence: 100, version: 5 }); const frame = parseGpu3dPacket(packet);
  assert.equal(frame.blend, "replace"); assert.equal(frame.depth, true);
  const device = fakeDevice(); const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.equal((await display.present3d(packet)).success, true);
  assert.equal(device.pipelines.every((pipeline) => !("blend" in pipeline.descriptor.fragment.targets[0])), true);
  assert.equal(device.pipelines.every((pipeline) => pipeline.descriptor.depthStencil.format === "depth24plus"), true);
});

test("non-depth singleton material batches retain and rekey one resident output", async () => {
  const fresh = parseGpu3dPacket(virglMaterialBatchPacket({ drawCount: 1, sequence: 94, version: 2 }));
  assert.equal(fresh.depth, false); assert.equal(fresh.residentCandidate, true);
  assert.throws(() => parseGpu3dPacket(virglMaterialBatchPacket({ drawCount: 1 })), /VGM1 framing/);
  const replacement = parseGpu3dPacket(virglMaterialBatchPacket({
    drawCount: 1, residentPreviousProducer: 94, sequence: 95, version: 3,
  }));
  assert.equal(replacement.residentPreviousProducer, 94);
  assert.throws(() => parseGpu3dPacket(virglMaterialBatchPacket({
    drawCount: 1, residentPreviousProducer: 95, sequence: 95, version: 3,
  })), /replacement producer/);
  const pixels = new Uint8Array(1024 * 768 * 4); pixels.set([3, 2, 1, 255]);
  const device = fakeDevice({ readbackBytes: pixels });
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglMaterialBatchPacket({ drawCount: 1, sequence: 94, version: 2 })), {
    resident: true, sequence: 94, success: true,
  });
  const outputs = device.textures.filter((texture) => texture.descriptor.label?.startsWith("VirGL resident output"));
  assert.equal(outputs.length, 1); assert.equal(device.textureTransfers.length, 1);
  assert.deepEqual(await display.present3d(virglMaterialBatchPacket({
    drawCount: 1, residentPreviousProducer: 94, sequence: 95, version: 3,
  })), { resident: true, sequence: 95, success: true });
  assert.equal(outputs.length, 1); assert.equal(device.textureTransfers.length, 2);
  assert.equal((await display.present3d(virglResidentReadbackPacket({
    producerSequence: 95, sequence: 96,
  }))).success, true);
  assert.equal(outputs[0].destroyed, true);
});

test("material pipeline setup preserves a shared resident clear target", async () => {
  const device = fakeDevice();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  await display.present3d(virglClearPacket({ sequence: 97, version: 2 }));
  assert.deepEqual(await display.present3d(virglMaterialBatchPacket({
    residentPreviousProducer: 97, sequence: 98, version: 3,
  })), { resident: true, sequence: 98, success: true });
  const outputs = device.textures.filter((texture) => texture.descriptor.label?.startsWith("VirGL resident output"));
  assert.equal(outputs.length, 1);
  assert.equal((await display.present3d(virglResidentReadbackPacket({
    producerSequence: 98, sequence: 99,
  }))).success, true);
  assert.equal(outputs[0].destroyed, true);
});

function textureColorDraw() {
  const texture = { addressMode: "clamp-to-edge", filter: "nearest", height: 2, pixels: new Uint8Array(16).fill(128), width: 2 };
  const vertices = [0, .75, 0, 1, 1, 1, 1, 1, .5, 0, -.75, -.75, 0, 1, 1, 1, 1, 1, 0, 1, .75, -.75, 0, 1, 1, 1, 1, 1, 1, 1];
  return { material: "texture-color", scissor: [0, 0, 1024, 768], texture, vertices, viewport: [512, 384, .5, 512, 384, .5] };
}

function solidDraw() {
  return { drawColor: [1, 0, 0, 1], material: "solid", scissor: [0, 0, 1024, 768], vertices: [0, .75, 0, 1, -.75, -.75, 0, 1, .75, -.75, 0, 1], viewport: [512, 384, .5, 512, 384, .5] };
}
