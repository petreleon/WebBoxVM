import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglMatrixTexturePacket } from "./gpu-test-virgl-matrix.mjs?v=20260904-virgl-readback-pool-r1";

test("standard VirGL matrix textures sample on WebGPU after GPU DP4 position transforms", async () => {
  const device = fakeDevice(); const status = fakeStatus(); const matrix = [0.5, 0, 0, 0.25, 0, 0.5, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu: fakeGpu([fakeAdapter(device)]) } });
  assert.deepEqual(await display.present3d(virglMatrixTexturePacket({ matrix, sequence: 96 })), { sequence: 96, success: true });
  assert.deepEqual([...new Float32Array(device.bufferWrites[0].data.buffer)], matrix.map(Math.fround));
  assert.deepEqual(device.pipelines[0].descriptor.vertex.buffers[0].attributes, [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x2", offset: 16, shaderLocation: 1 }]);
  assert.match(device.shaderModules[0].code, /dot\(scene\.matrix\[0\]/); assert.match(device.shaderModules[0].code, /textureSampleLevel/);
  assert.deepEqual([...device.writes[0].data.subarray(0, 4)], [10, 20, 30, 255]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-matrix-texture");
});
