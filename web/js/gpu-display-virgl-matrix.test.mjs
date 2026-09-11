import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglMatrixPacket } from "./gpu-test-virgl-matrix.mjs?v=20260904-virgl-readback-pool-r1";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const RUST_TEST = "devices::virtio_gpu::tests::virgl_matrix_draw::vertex_dp4_matrix_transforms_standard_virgl_vertices";

test("standard VirGL matrix draws execute their DP4 rows through a WebGPU uniform", async () => {
  const device = fakeDevice(); const status = fakeStatus(); const matrix = [0.5, 0, 0, 0.25, 0, 0.5, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu: fakeGpu([fakeAdapter(device)]) } });
  assert.deepEqual(await display.present3d(virglMatrixPacket({ matrix, sequence: 92 })), { sequence: 92, success: true });
  const scene = new Float32Array(device.bufferWrites[0].data.buffer);
  assert.deepEqual([...scene.slice(0, 16)], matrix.map(Math.fround)); assert.deepEqual([...scene.slice(16)], [0, 1, 0, 0.25]);
  assert.equal(device.buffers[0].descriptor.size, 80); assert.match(device.shaderModules[0].code, /dot\(scene\.matrix\[0\]/);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-matrix");
});

test("Rust-emitted VirGL v15 packets parse and submit through the private WebGPU matrix path", async () => {
  const packet = rustMatrixPacket();
  assert.deepEqual([...packet.subarray(0, 4)], [0x56, 0x47, 0x44, 0x31]); assert.equal(packet.byteLength, 208);
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.version, 15); assert.equal(frame.protocol, "virgl-draw"); assert.equal(frame.vertexCount, 3);
  assert.equal(frame.acceleration, "webgpu-virgl-capset1-matrix"); assert.ok(frame.sequence);
  assert.deepEqual([...frame.matrix], [0.5, 0, 0, 0.25, 0, 0.5, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);
  assert.deepEqual([...frame.vertices], [0, 0.75, 0, 1, -0.75, -0.75, 0, 1, 0.75, -0.75, 0, 1]);
  const device = fakeDevice(); const status = fakeStatus();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, { navigator: { gpu: fakeGpu([fakeAdapter(device)]) } });
  const acknowledgement = await display.present3d(packet);
  assert.deepEqual(acknowledgement, { sequence: frame.sequence, success: true }); assert.equal(Object.hasOwn(acknowledgement, "readback"), false);
  assert.deepEqual(device.draw, [frame.vertexCount]); assert.equal(device.submits, 1); assert.equal(device.textureCopies.length, 0); assert.equal(device.renderPasses.length, 1);
  const scene = new Float32Array(device.bufferWrites[0].data.buffer);
  assert.deepEqual([...scene.slice(0, 16)], [...frame.matrix]); assert.deepEqual([...scene.slice(16)], [...frame.drawColor]);
  assert.equal(device.bufferWrites.length, 2); assert.match(device.shaderModules[0].code, /dot\(scene\.matrix\[0\]/);
  assert.deepEqual([...new Float32Array(device.bufferWrites[1].data.buffer)], [...frame.vertices]);
  assert.equal(status.dataset.threeDAcceleration, "webgpu-virgl-capset1-matrix");
});

function rustMatrixPacket() {
  const output = execFileSync("cargo", ["test", "-p", "emulator", "--lib", RUST_TEST, "--", "--exact", "--nocapture"], { cwd: ROOT, encoding: "utf8" });
  const match = output.match(/WEBBOXVM_VIRGL_MATRIX_V15_HEX:([0-9a-f]+)\b/);
  assert.ok(match, "the Rust v15 matrix regression must emit packet bytes");
  return new Uint8Array(Buffer.from(match[1], "hex"));
}
