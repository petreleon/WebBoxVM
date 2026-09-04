import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglMatrixVertexColorPacket } from "./gpu-test-virgl-matrix.mjs?v=20260904-virgl-readback-pool-r1";

test("standard VirGL matrix vertex colors retain generic RGBA while WebGPU transforms position", async () => {
  const device = fakeDevice(); const status = fakeStatus(); const matrix = [0.5, 0, 0, 0.25, 0, 0.5, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu: fakeGpu([fakeAdapter(device)]) } });
  assert.deepEqual(await display.present3d(virglMatrixVertexColorPacket({ matrix, sequence: 94 })), { sequence: 94, success: true });
  assert.deepEqual([...new Float32Array(device.bufferWrites[0].data.buffer)], matrix.map(Math.fround));
  assert.deepEqual(device.pipelines[0].descriptor.vertex.buffers[0].attributes, [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 }]);
  assert.match(device.shaderModules[0].code, /output\.color = color/); assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-matrix-vertex-color");
});
