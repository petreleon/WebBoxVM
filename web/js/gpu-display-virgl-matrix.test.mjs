import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglMatrixPacket } from "./gpu-test-virgl-matrix.mjs?v=20260904-virgl-readback-pool-r1";

test("standard VirGL matrix draws execute their DP4 rows through a WebGPU uniform", async () => {
  const device = fakeDevice(); const status = fakeStatus(); const matrix = [0.5, 0, 0, 0.25, 0, 0.5, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu: fakeGpu([fakeAdapter(device)]) } });
  assert.deepEqual(await display.present3d(virglMatrixPacket({ matrix, sequence: 92 })), { sequence: 92, success: true });
  const scene = new Float32Array(device.bufferWrites[0].data.buffer);
  assert.deepEqual([...scene.slice(0, 16)], matrix.map(Math.fround)); assert.deepEqual([...scene.slice(16)], [0, 1, 0, 0.25]);
  assert.equal(device.buffers[0].descriptor.size, 80); assert.match(device.shaderModules[0].code, /dot\(scene\.matrix\[0\]/);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-matrix");
});
